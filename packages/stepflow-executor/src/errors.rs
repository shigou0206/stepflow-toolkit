//! Error types for the executor

use thiserror::Error;
use stepflow_core::*;
use crate::execution_context::{TaskId, WorkId};

/// Executor error type
#[derive(Debug, Error)]
pub enum ExecutorError {
    #[error("Tool not found: {0}")]
    ToolNotFound(ToolId),
    
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    
    #[error("Timeout exceeded")]
    TimeoutExceeded,
    
    #[error("Resource limit exceeded")]
    ResourceLimitExceeded,
    
    #[error("Permission denied")]
    PermissionDenied,
    
    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Registry error: {0}")]
    RegistryError(String),
    
    #[error("Monitoring error: {0}")]
    MonitoringError(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Scheduler error type
#[derive(Debug, Error)]
pub enum SchedulerError {
    #[error("Task not found: {0}")]
    TaskNotFound(TaskId),
    
    #[error("Queue full")]
    QueueFull,
    
    #[error("Scheduler not running")]
    SchedulerNotRunning,
    
    #[error("Invalid task: {0}")]
    InvalidTask(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Worker pool error type
#[derive(Debug, Error)]
pub enum WorkerPoolError {
    #[error("Work not found: {0}")]
    WorkNotFound(WorkId),
    
    #[error("Worker pool full")]
    PoolFull,
    
    #[error("Worker pool not running")]
    PoolNotRunning,
    
    #[error("Invalid work: {0}")]
    InvalidWork(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

// 使用stepflow_core中的MonitoringError
pub use stepflow_core::MonitoringError;

/// Result types
pub type ExecutorResult<T> = Result<T, ExecutorError>;
pub type SchedulerResult<T> = Result<T, SchedulerError>;
pub type WorkerPoolResult<T> = Result<T, WorkerPoolError>;
pub type MonitoringResult<T> = Result<T, MonitoringError>;

/// Error conversions
impl From<stepflow_core::StepflowError> for ExecutorError {
    fn from(error: stepflow_core::StepflowError) -> Self {
        ExecutorError::InternalError(error.to_string())
    }
}

impl From<stepflow_registry::RegistryError> for ExecutorError {
    fn from(error: stepflow_registry::RegistryError) -> Self {
        ExecutorError::RegistryError(error.to_string())
    }
}

impl From<serde_json::Error> for ExecutorError {
    fn from(error: serde_json::Error) -> Self {
        ExecutorError::InternalError(format!("JSON error: {}", error))
    }
}

impl From<tokio::time::error::Elapsed> for ExecutorError {
    fn from(_: tokio::time::error::Elapsed) -> Self {
        ExecutorError::TimeoutExceeded
    }
} 