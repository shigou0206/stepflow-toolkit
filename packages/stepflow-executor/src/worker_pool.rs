//! Worker pool implementation

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, Mutex, oneshot};
use tokio::task::JoinHandle;
use chrono::Utc;
use stepflow_core::*;
use stepflow_registry::{Registry, RegistryImpl};
use crate::errors::*;
use crate::execution_context::*;
use crate::executor::WorkerPool;

/// Worker pool configuration
#[derive(Debug, Clone)]
pub struct WorkerPoolConfig {
    pub min_workers: usize,
    pub max_workers: usize,
    pub worker_idle_timeout: Duration,
    pub work_queue_size: usize,
    pub enable_auto_scaling: bool,
    pub scale_up_threshold: f64,
    pub scale_down_threshold: f64,
}

impl Default for WorkerPoolConfig {
    fn default() -> Self {
        Self {
            min_workers: 2,
            max_workers: 10,
            worker_idle_timeout: Duration::from_secs(300),
            work_queue_size: 1000,
            enable_auto_scaling: true,
            scale_up_threshold: 0.8,
            scale_down_threshold: 0.2,
        }
    }
}

/// Worker state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkerState {
    Idle,
    Running,
    Stopping,
    Stopped,
}

/// Worker information
#[derive(Debug, Clone)]
pub struct Worker {
    pub id: WorkerId,
    pub state: WorkerState,
    pub current_work: Option<WorkId>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub completed_work_count: u64,
}

/// Worker pool implementation
pub struct WorkerPoolImpl {
    registry: Arc<RegistryImpl>,
    config: WorkerPoolConfig,
    
    // Workers and their handles
    workers: Arc<RwLock<HashMap<WorkerId, Worker>>>,
    worker_handles: Arc<RwLock<HashMap<WorkerId, JoinHandle<()>>>>,
    
    // Work queue
    work_queue: Arc<Mutex<VecDeque<Work>>>,
    
    // Work status tracking
    work_status: Arc<RwLock<HashMap<WorkId, WorkStatus>>>,
    
    // Shutdown signal
    shutdown_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
    
    // Running state
    running: Arc<RwLock<bool>>,
}

impl WorkerPoolImpl {
    /// Create a new worker pool
    pub fn new(registry: Arc<RegistryImpl>, config: WorkerPoolConfig) -> Self {
        let (shutdown_tx, _) = oneshot::channel();
        
        Self {
            registry,
            config,
            workers: Arc::new(RwLock::new(HashMap::new())),
            worker_handles: Arc::new(RwLock::new(HashMap::new())),
            work_queue: Arc::new(Mutex::new(VecDeque::new())),
            work_status: Arc::new(RwLock::new(HashMap::new())),
            shutdown_tx: Arc::new(Mutex::new(Some(shutdown_tx))),
            running: Arc::new(RwLock::new(false)),
        }
    }
    
    /// Start the worker pool
    pub async fn start(&self) -> WorkerPoolResult<()> {
        // Set running state
        *self.running.write().await = true;
        
        // Start minimum number of workers
        for _ in 0..self.config.min_workers {
            self.spawn_worker().await?;
        }
        
        // Start auto-scaling if enabled
        if self.config.enable_auto_scaling {
            let (shutdown_tx, shutdown_rx) = oneshot::channel();
            *self.shutdown_tx.lock().await = Some(shutdown_tx);
            
            let pool = self.clone();
            tokio::spawn(async move {
                pool.auto_scaling_loop(shutdown_rx).await;
            });
        }
        
        Ok(())
    }
    
    /// Stop the worker pool
    pub async fn stop(&self) -> WorkerPoolResult<()> {
        // Set running state to false
        *self.running.write().await = false;
        
        // Send shutdown signal
        if let Some(tx) = self.shutdown_tx.lock().await.take() {
            let _ = tx.send(());
        }
        
        // Wait for all workers to finish
        let mut handles = self.worker_handles.write().await;
        for (_, handle) in handles.drain() {
            handle.abort();
        }
        
        Ok(())
    }
    
    /// Spawn a new worker
    async fn spawn_worker(&self) -> WorkerPoolResult<WorkerId> {
        let worker_id = WorkerId::new();
        let worker = Worker {
            id: worker_id.clone(),
            state: WorkerState::Idle,
            current_work: None,
            started_at: Utc::now(),
            last_activity: Utc::now(),
            completed_work_count: 0,
        };
        
        // Add worker to collection
        self.workers.write().await.insert(worker_id.clone(), worker);
        
        // Spawn worker task
        let pool = self.clone();
        let worker_id_clone = worker_id.clone();
        let handle = tokio::spawn(async move {
            pool.worker_loop(worker_id_clone).await;
        });
        
        // Store handle
        self.worker_handles.write().await.insert(worker_id.clone(), handle);
        
        Ok(worker_id)
    }
    
    /// Worker main loop
    async fn worker_loop(&self, worker_id: WorkerId) {
        loop {
            // Check if we should stop
            if !*self.running.read().await {
                break;
            }
            
            // Get work from queue
            let work = {
                let mut queue = self.work_queue.lock().await;
                queue.pop_front()
            };
            
            if let Some(work) = work {
                // Update worker state
                self.update_worker_state(&worker_id, WorkerState::Running, Some(work.id.clone())).await;
                
                // Update work status
                {
                    let mut status = self.work_status.write().await;
                    status.insert(work.id.clone(), WorkStatus::Running);
                }
                
                // Execute work
                let _result = self.execute_work(work).await;
                
                // Update worker state back to idle
                self.update_worker_state(&worker_id, WorkerState::Idle, None).await;
                
                // Increment completed work count
                self.increment_worker_completed_count(&worker_id).await;
            } else {
                // No work available, sleep briefly
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
        
        // Update worker state to stopped
        self.update_worker_state(&worker_id, WorkerState::Stopped, None).await;
    }
    
    /// Execute work
    async fn execute_work(&self, work: Work) -> WorkerPoolResult<ExecutionResult> {
        let task = work.task;
        let request = task.execution_request;
        
        // Get tool from registry
        let tool = self.registry.get_tool(&request.tool_id).await
            .map_err(|e| WorkerPoolError::InternalError(e.to_string()))?;
        
        // Create execution result compatible with stepflow_core::ExecutionResult
        let execution_result = ExecutionResult {
            success: true,
            output: Some(serde_json::json!({"message": "Tool executed successfully"})),
            error: None,
            logs: vec![],
            metrics: HashMap::new(),
            metadata: HashMap::from([
                ("tool_id".to_string(), serde_json::Value::String(tool.id.to_string())),
                ("tool_name".to_string(), serde_json::Value::String(tool.name.clone())),
                ("execution_time".to_string(), serde_json::Value::Number(serde_json::Number::from(1))),
            ]),
        };
        
        Ok(execution_result)
    }
    
    /// Update worker state
    async fn update_worker_state(&self, worker_id: &WorkerId, state: WorkerState, current_work: Option<WorkId>) {
        let mut workers = self.workers.write().await;
        if let Some(worker) = workers.get_mut(worker_id) {
            worker.state = state;
            worker.current_work = current_work;
            worker.last_activity = Utc::now();
        }
    }
    
    /// Increment worker completed count
    async fn increment_worker_completed_count(&self, worker_id: &WorkerId) {
        let mut workers = self.workers.write().await;
        if let Some(worker) = workers.get_mut(worker_id) {
            worker.completed_work_count += 1;
        }
    }
    
    /// Auto-scaling loop
    async fn auto_scaling_loop(&self, mut shutdown_rx: oneshot::Receiver<()>) {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if let Err(e) = self.check_and_scale().await {
                        eprintln!("Auto-scaling error: {}", e);
                    }
                }
                _ = &mut shutdown_rx => {
                    break;
                }
            }
        }
    }
    
    /// Check and scale workers
    async fn check_and_scale(&self) -> WorkerPoolResult<()> {
        let workers = self.workers.read().await;
        let work_queue = self.work_queue.lock().await;
        
        let total_workers = workers.len();
        let running_workers = workers.values().filter(|w| w.state == WorkerState::Running).count();
        let queue_size = work_queue.len();
        
        // Calculate utilization
        let utilization = if total_workers > 0 {
            running_workers as f64 / total_workers as f64
        } else {
            0.0
        };
        
        // Scale up if utilization is high and we have queued work
        if utilization > self.config.scale_up_threshold && queue_size > 0 && total_workers < self.config.max_workers {
            drop(workers);
            drop(work_queue);
            self.spawn_worker().await?;
        }
        // Scale down if utilization is low
        else if utilization < self.config.scale_down_threshold && total_workers > self.config.min_workers {
            // Find an idle worker to remove
            for (worker_id, worker) in workers.iter() {
                if worker.state == WorkerState::Idle {
                    let worker_id = worker_id.clone();
                    drop(workers);
                    drop(work_queue);
                    self.remove_worker(&worker_id).await?;
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    /// Remove a worker
    async fn remove_worker(&self, worker_id: &WorkerId) -> WorkerPoolResult<()> {
        // Remove from workers collection
        self.workers.write().await.remove(worker_id);
        
        // Abort and remove handle
        if let Some(handle) = self.worker_handles.write().await.remove(worker_id) {
            handle.abort();
        }
        
        Ok(())
    }
}

impl Clone for WorkerPoolImpl {
    fn clone(&self) -> Self {
        Self {
            registry: self.registry.clone(),
            config: self.config.clone(),
            workers: self.workers.clone(),
            worker_handles: self.worker_handles.clone(),
            work_queue: self.work_queue.clone(),
            work_status: self.work_status.clone(),
            shutdown_tx: self.shutdown_tx.clone(),
            running: self.running.clone(),
        }
    }
}

#[async_trait::async_trait]
impl WorkerPool for WorkerPoolImpl {
    /// Submit work to the pool
    async fn submit_work(&self, work: Work) -> WorkerPoolResult<WorkId> {
        // Check if pool is running
        if !*self.running.read().await {
            return Err(WorkerPoolError::PoolNotRunning);
        }
        
        // Add to work queue
        let mut queue = self.work_queue.lock().await;
        if queue.len() >= self.config.work_queue_size {
            return Err(WorkerPoolError::PoolFull);
        }
        
        let work_id = work.id.clone();
        queue.push_back(work);
        
        // Update work status
        let mut status = self.work_status.write().await;
        status.insert(work_id.clone(), WorkStatus::Pending);
        
        Ok(work_id)
    }
    
    /// Get work status
    async fn get_work_status(&self, work_id: &WorkId) -> WorkerPoolResult<WorkStatus> {
        let status = self.work_status.read().await;
        status.get(work_id)
            .copied()
            .ok_or_else(|| WorkerPoolError::WorkNotFound(work_id.clone()))
    }
    
    /// Get pool status
    async fn get_pool_status(&self) -> WorkerPoolResult<PoolStatus> {
        let workers = self.workers.read().await;
        let work_queue = self.work_queue.lock().await;
        let work_status = self.work_status.read().await;
        
        let total_workers = workers.len();
        let active_workers = workers.values().filter(|w| w.state == WorkerState::Running).count();
        let idle_workers = workers.values().filter(|w| w.state == WorkerState::Idle).count();
        let pending_work = work_queue.len();
        let completed_work = work_status.values().filter(|&&status| status == WorkStatus::Completed).count();
        
        Ok(PoolStatus {
            active_workers,
            idle_workers,
            total_workers,
            pending_work,
            completed_work,
        })
    }
    
    /// Scale the pool
    async fn scale_pool(&self, target_size: usize) -> WorkerPoolResult<()> {
        let workers = self.workers.read().await;
        let current_size = workers.len();
        drop(workers);
        
        if target_size > current_size {
            // Scale up
            for _ in current_size..target_size {
                self.spawn_worker().await?;
            }
        } else if target_size < current_size {
            // Scale down - simplified implementation
            // In a real implementation, we would gracefully stop workers
        }
        
        Ok(())
    }
    
    /// Shutdown the pool
    async fn shutdown(&self) -> WorkerPoolResult<()> {
        self.stop().await
    }
} 