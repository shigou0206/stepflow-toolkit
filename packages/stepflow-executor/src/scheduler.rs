//! Task scheduler implementation

use std::collections::{HashMap, VecDeque, BinaryHeap};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, Mutex};
use tokio::time::sleep;
use chrono::Utc;
use async_trait::async_trait;
use stepflow_core::*;
use stepflow_database::SqliteDatabase;
use crate::errors::*;
use crate::execution_context::*;
use crate::executor::{Scheduler, WorkerPool, TaskFilter, TaskInfo};

/// Scheduler configuration
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    pub queue_size: usize,
    pub worker_count: usize,
    pub priority_levels: usize,
    pub enable_priority_queue: bool,
    pub enable_fair_scheduling: bool,
    pub polling_interval: Duration,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            queue_size: 1000,
            worker_count: 10,
            priority_levels: 4,
            enable_priority_queue: true,
            enable_fair_scheduling: true,
            polling_interval: Duration::from_millis(100),
        }
    }
}

/// Task wrapper for priority queue
#[derive(Debug, Clone)]
struct PriorityTask {
    task: Task,
    priority: Priority,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl PartialEq for PriorityTask {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.created_at == other.created_at
    }
}

impl Eq for PriorityTask {}

impl PartialOrd for PriorityTask {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PriorityTask {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Higher priority first, then earlier created_at
        self.priority.cmp(&other.priority)
            .then_with(|| other.created_at.cmp(&self.created_at))
    }
}

/// Scheduler implementation
pub struct SchedulerImpl {
    db: Arc<SqliteDatabase>,
    worker_pool: Arc<dyn WorkerPool>,
    config: SchedulerConfig,
    
    // Priority queue for tasks
    priority_queue: Arc<Mutex<BinaryHeap<PriorityTask>>>,
    
    // FIFO queue for fair scheduling
    fifo_queue: Arc<Mutex<VecDeque<Task>>>,
    
    // Task status tracking
    task_status: Arc<RwLock<HashMap<TaskId, TaskStatus>>>,
    
    // Running state
    running: Arc<RwLock<bool>>,
}

impl SchedulerImpl {
    /// Create a new scheduler
    pub fn new(
        db: Arc<SqliteDatabase>,
        worker_pool: Arc<dyn WorkerPool>,
        config: SchedulerConfig,
    ) -> Self {
        Self {
            db,
            worker_pool,
            config,
            priority_queue: Arc::new(Mutex::new(BinaryHeap::new())),
            fifo_queue: Arc::new(Mutex::new(VecDeque::new())),
            task_status: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
        }
    }
    
    /// Start the scheduler
    pub async fn start(&self) -> SchedulerResult<()> {
        let mut running = self.running.write().await;
        if *running {
            return Err(SchedulerError::InternalError("Scheduler already running".to_string()));
        }
        
        *running = true;
        
        // Start scheduling loop
        let scheduler = self.clone();
        tokio::spawn(async move {
            scheduler.scheduling_loop().await;
        });
        
        Ok(())
    }
    
    /// Stop the scheduler
    pub async fn stop(&self) -> SchedulerResult<()> {
        let mut running = self.running.write().await;
        *running = false;
        Ok(())
    }
    
    /// Main scheduling loop
    async fn scheduling_loop(&self) {
        while *self.running.read().await {
            if let Err(e) = self.process_tasks().await {
                tracing::error!("Error processing tasks: {}", e);
            }
            
            sleep(self.config.polling_interval).await;
        }
    }
    
    /// Process pending tasks
    async fn process_tasks(&self) -> SchedulerResult<()> {
        let pool_status = self.worker_pool.get_pool_status().await
            .map_err(|e| SchedulerError::InternalError(e.to_string()))?;
        
        // Check if we have available workers
        if pool_status.idle_workers == 0 {
            return Ok(());
        }
        
        // Get next task to process
        let task = if self.config.enable_priority_queue {
            self.get_next_priority_task().await
        } else {
            self.get_next_fifo_task().await
        };
        
        if let Some(task) = task {
            // Submit to worker pool
            let work = Work {
                id: WorkId::new(),
                task: task.clone(),
                assigned_worker: None,
                started_at: None,
            };
            
            match self.worker_pool.submit_work(work).await {
                Ok(_) => {
                    // Update task status
                    let mut status = self.task_status.write().await;
                    status.insert(task.id.clone(), TaskStatus::Running);
                }
                Err(e) => {
                    tracing::error!("Failed to submit work: {}", e);
                    // Mark task as failed
                    let mut status = self.task_status.write().await;
                    status.insert(task.id.clone(), TaskStatus::Failed);
                }
            }
        }
        
        Ok(())
    }
    
    /// Get next task from priority queue
    async fn get_next_priority_task(&self) -> Option<Task> {
        let mut queue = self.priority_queue.lock().await;
        queue.pop().map(|pt| pt.task)
    }
    
    /// Get next task from FIFO queue
    async fn get_next_fifo_task(&self) -> Option<Task> {
        let mut queue = self.fifo_queue.lock().await;
        queue.pop_front()
    }
    
    /// Add task to appropriate queue
    async fn add_task_to_queue(&self, task: Task) -> SchedulerResult<()> {
        if self.config.enable_priority_queue {
            let priority_task = PriorityTask {
                priority: task.priority,
                created_at: task.created_at,
                task,
            };
            
            let mut queue = self.priority_queue.lock().await;
            if queue.len() >= self.config.queue_size {
                return Err(SchedulerError::QueueFull);
            }
            queue.push(priority_task);
        } else {
            let mut queue = self.fifo_queue.lock().await;
            if queue.len() >= self.config.queue_size {
                return Err(SchedulerError::QueueFull);
            }
            queue.push_back(task);
        }
        
        Ok(())
    }
    
    /// Store task in database
    async fn store_task(&self, task: &Task) -> SchedulerResult<()> {
        let task_json = serde_json::to_string(task)
            .map_err(|e| SchedulerError::InternalError(e.to_string()))?;
        let execution_request_json = serde_json::to_string(&task.execution_request)
            .map_err(|e| SchedulerError::InternalError(e.to_string()))?;
        
        let params = vec![
            serde_json::Value::String(task.id.to_string()),
            serde_json::Value::String(task.execution_request.tool_id.to_string()),
            serde_json::Value::String(execution_request_json),
            serde_json::Value::String(format!("{:?}", task.priority)),
            serde_json::Value::String("Pending".to_string()),
            serde_json::Value::String(task_json),
            serde_json::Value::String(task.created_at.to_rfc3339()),
        ];
        
        self.db.execute(
            "INSERT INTO tasks (id, tool_id, execution_request, priority, status, task_data, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
            &params
        ).await.map_err(|e| SchedulerError::DatabaseError(e.to_string()))?;
        
        Ok(())
    }
    
    /// Update task status in database
    async fn update_task_status(&self, task_id: &TaskId, status: TaskStatus) -> SchedulerResult<()> {
        let params = vec![
            serde_json::Value::String(format!("{:?}", status)),
            serde_json::Value::String(task_id.to_string()),
        ];
        
        self.db.execute(
            "UPDATE tasks SET status = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            &params
        ).await.map_err(|e| SchedulerError::DatabaseError(e.to_string()))?;
        
        Ok(())
    }
}

impl Clone for SchedulerImpl {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            worker_pool: self.worker_pool.clone(),
            config: self.config.clone(),
            priority_queue: self.priority_queue.clone(),
            fifo_queue: self.fifo_queue.clone(),
            task_status: self.task_status.clone(),
            running: self.running.clone(),
        }
    }
}

#[async_trait]
impl Scheduler for SchedulerImpl {
    async fn schedule_task(&self, mut task: Task) -> SchedulerResult<TaskId> {
        // Generate task ID (TaskId is already generated in the task)
        task.id = TaskId::new();
        
        // Set created timestamp
        task.created_at = Utc::now();
        
        // Store in database
        self.store_task(&task).await?;
        
        // Add to queue
        self.add_task_to_queue(task.clone()).await?;
        
        // Update status
        let mut status = self.task_status.write().await;
        status.insert(task.id.clone(), TaskStatus::Queued);
        
        Ok(task.id)
    }
    
    async fn get_task_status(&self, task_id: &TaskId) -> SchedulerResult<TaskStatus> {
        let status = self.task_status.read().await;
        status.get(task_id)
            .copied()
            .ok_or_else(|| SchedulerError::TaskNotFound(task_id.clone()))
    }
    
    async fn cancel_task(&self, task_id: &TaskId) -> SchedulerResult<()> {
        // Update status
        let mut status = self.task_status.write().await;
        status.insert(task_id.clone(), TaskStatus::Cancelled);
        
        // Remove from queues (simplified implementation)
        // In a real implementation, we would need to remove from priority/fifo queues
        
        Ok(())
    }
    
    async fn get_queue_status(&self) -> SchedulerResult<QueueStatus> {
        let priority_queue = self.priority_queue.lock().await;
        let fifo_queue = self.fifo_queue.lock().await;
        let task_status = self.task_status.read().await;
        
        let pending_tasks = priority_queue.len() + fifo_queue.len();
        let running_tasks = task_status.values().filter(|&&status| status == TaskStatus::Running).count();
        let completed_tasks = task_status.values().filter(|&&status| status == TaskStatus::Completed).count();
        let failed_tasks = task_status.values().filter(|&&status| status == TaskStatus::Failed).count();
        
        Ok(QueueStatus {
            pending_tasks,
            running_tasks,
            completed_tasks,
            failed_tasks,
            total_capacity: self.config.queue_size,
        })
    }
    
    async fn list_tasks(&self, _filter: Option<TaskFilter>) -> SchedulerResult<Vec<TaskInfo>> {
        // Simplified implementation - return empty list for now
        // In a real implementation, we would query the database with filters
        Ok(vec![])
    }
} 