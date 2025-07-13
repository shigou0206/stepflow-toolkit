//! Core executor traits and interfaces

use async_trait::async_trait;
use stepflow_core::*;
use crate::errors::*;
use crate::execution_context::*;

/// Core executor trait
#[async_trait]
pub trait Executor: Send + Sync {
    /// Execute a tool synchronously
    async fn execute_tool(&self, request: ExecutionRequest) -> ExecutorResult<ExecutionResult>;
    
    /// Execute a tool asynchronously
    async fn execute_tool_async(&self, request: ExecutionRequest) -> ExecutorResult<ExecutionId>;
    
    /// Get execution status
    async fn get_execution_status(&self, execution_id: &ExecutionId) -> ExecutorResult<ExecutionStatus>;
    
    /// Cancel execution
    async fn cancel_execution(&self, execution_id: &ExecutionId) -> ExecutorResult<()>;
    
    /// Get execution result
    async fn get_execution_result(&self, execution_id: &ExecutionId) -> ExecutorResult<ExecutionResult>;
    
    /// List executions
    async fn list_executions(&self, filter: Option<ExecutionFilter>) -> ExecutorResult<Vec<ExecutionInfo>>;
    
    /// Get execution metrics
    async fn get_execution_metrics(&self, execution_id: &ExecutionId) -> ExecutorResult<Vec<Metric>>;
    
    /// Health check for the executor
    async fn health_check(&self) -> ExecutorResult<bool>;
}

/// Task scheduler trait
#[async_trait]
pub trait Scheduler: Send + Sync {
    /// Schedule a task
    async fn schedule_task(&self, task: Task) -> SchedulerResult<TaskId>;
    
    /// Get task status
    async fn get_task_status(&self, task_id: &TaskId) -> SchedulerResult<TaskStatus>;
    
    /// Cancel task
    async fn cancel_task(&self, task_id: &TaskId) -> SchedulerResult<()>;
    
    /// Get queue status
    async fn get_queue_status(&self) -> SchedulerResult<QueueStatus>;
    
    /// List tasks
    async fn list_tasks(&self, filter: Option<TaskFilter>) -> SchedulerResult<Vec<TaskInfo>>;
}

/// Worker pool trait
#[async_trait]
pub trait WorkerPool: Send + Sync {
    /// Submit work to the pool
    async fn submit_work(&self, work: Work) -> WorkerPoolResult<WorkId>;
    
    /// Get work status
    async fn get_work_status(&self, work_id: &WorkId) -> WorkerPoolResult<WorkStatus>;
    
    /// Get pool status
    async fn get_pool_status(&self) -> WorkerPoolResult<PoolStatus>;
    
    /// Scale the pool
    async fn scale_pool(&self, target_size: usize) -> WorkerPoolResult<()>;
    
    /// Shutdown the pool
    async fn shutdown(&self) -> WorkerPoolResult<()>;
}

/// Result manager trait
#[async_trait]
pub trait ResultManager: Send + Sync {
    /// Store execution result
    async fn store_result(&self, result: ExecutionResult) -> ExecutorResult<()>;
    
    /// Get execution result
    async fn get_result(&self, execution_id: &ExecutionId) -> ExecutorResult<ExecutionResult>;
    
    /// Delete execution result
    async fn delete_result(&self, execution_id: &ExecutionId) -> ExecutorResult<()>;
    
    /// List results
    async fn list_results(&self, filter: Option<ExecutionFilter>) -> ExecutorResult<Vec<ExecutionResult>>;
    
    /// Clean up old results
    async fn cleanup_results(&self, older_than: chrono::DateTime<chrono::Utc>) -> ExecutorResult<u64>;
}

/// Monitoring trait
#[async_trait]
pub trait Monitoring: Send + Sync {
    /// Record execution start
    async fn record_execution_start(&self, execution_id: &ExecutionId) -> MonitoringResult<()>;
    
    /// Record execution end
    async fn record_execution_end(&self, execution_id: &ExecutionId, result: &ExecutionResult) -> MonitoringResult<()>;
    
    /// Record metric
    async fn record_metric(&self, execution_id: &ExecutionId, metric: Metric) -> MonitoringResult<()>;
    
    /// Get metrics
    async fn get_metrics(&self, filter: Option<MetricFilter>) -> MonitoringResult<Vec<Metric>>;
    
    /// Get execution metrics
    async fn get_execution_metrics(&self, execution_id: &ExecutionId) -> MonitoringResult<Vec<Metric>>;
}

/// Task filter for listing
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskFilter {
    pub status: Option<TaskStatus>,
    pub priority: Option<Priority>,
    pub created_after: Option<chrono::DateTime<chrono::Utc>>,
    pub created_before: Option<chrono::DateTime<chrono::Utc>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// Task info for listing
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskInfo {
    pub id: TaskId,
    pub tool_id: ToolId,
    pub status: TaskStatus,
    pub priority: Priority,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub scheduled_at: Option<chrono::DateTime<chrono::Utc>>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
} 