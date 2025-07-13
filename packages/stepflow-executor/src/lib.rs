//! Stepflow Executor - Tool execution engine for Stepflow Tool System
//!
//! This crate provides the core execution engine for the Stepflow Tool System.
//! It handles tool scheduling, execution, monitoring, and result management.

// Module declarations
pub mod errors;
pub mod execution_context;
pub mod executor;
pub mod executor_impl;
pub mod scheduler;
pub mod worker_pool;
pub mod result_manager;
pub mod monitoring;

// Re-export core types from stepflow_core (avoiding conflicts)
pub use stepflow_core::{
    ToolId, ToolVersion, ToolType, ToolStatus, ToolInfo, ToolConfig, ToolRequest, ToolResponse,
    ExecutionId, ExecutionStatus, TenantId, UserId, UserRole, UserInfo, TenantInfo,
    Pagination, Filter, FilterOperator, Sort, SortDirection, Query,
};

// Re-export key types and traits
pub use errors::*;
pub use execution_context::{
    TaskId, WorkId, WorkerId, ExecutionRequest, ExecutionContext, ExecutionOptions,
    ExecutionOutput, ExecutionMetadata, ExecutionTiming, Priority, ResourceLimits,
    ResourceUsage, MetricEntry, Task, TaskStatus, Work, WorkStatus, QueueStatus,
    PoolStatus, ExecutionInfo,
};
pub use executor::*;
pub use executor_impl::ExecutorImpl;
pub use scheduler::{SchedulerImpl, SchedulerConfig};
pub use worker_pool::{WorkerPoolImpl, WorkerPoolConfig};
pub use result_manager::ResultManagerImpl;
pub use monitoring::MonitoringImpl;

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Create a new executor with all components
pub fn create_executor(
    db: std::sync::Arc<stepflow_database::SqliteDatabase>,
    registry: std::sync::Arc<stepflow_registry::RegistryImpl>,
    scheduler_config: Option<SchedulerConfig>,
    worker_pool_config: Option<WorkerPoolConfig>,
) -> ExecutorResult<ExecutorImpl> {
    let scheduler_config = scheduler_config.unwrap_or_default();
    let worker_pool_config = worker_pool_config.unwrap_or_default();
    
    // Create worker pool
    let worker_pool = std::sync::Arc::new(WorkerPoolImpl::new(
        registry.clone(),
        worker_pool_config,
    ));
    
    // Create scheduler
    let scheduler = std::sync::Arc::new(SchedulerImpl::new(
        db.clone(),
        worker_pool.clone(),
        scheduler_config,
    ));
    
    // Create result manager
    let result_manager = std::sync::Arc::new(ResultManagerImpl::new(db.clone()));
    
    // Create monitoring
    let monitoring = std::sync::Arc::new(MonitoringImpl::new(db.clone()));
    
    // Create executor
    let executor = ExecutorImpl::new(
        scheduler.clone(),
        worker_pool.clone(),
        result_manager,
        monitoring,
        registry,
        db,
    );
    
    // Start the worker pool and scheduler
    tokio::spawn(async move {
        if let Err(e) = worker_pool.start().await {
            tracing::error!("Failed to start worker pool: {}", e);
        }
    });
    
    tokio::spawn(async move {
        if let Err(e) = scheduler.start().await {
            tracing::error!("Failed to start scheduler: {}", e);
        }
    });
    
    Ok(executor)
}

/// Create a new executor with default configuration
pub fn create_default_executor(
    db: std::sync::Arc<stepflow_database::SqliteDatabase>,
    registry: std::sync::Arc<stepflow_registry::RegistryImpl>,
) -> ExecutorResult<ExecutorImpl> {
    create_executor(db, registry, None, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    
    async fn create_test_executor() -> ExecutorResult<ExecutorImpl> {
        let db = Arc::new(stepflow_database::SqliteDatabase::new(":memory:").await.unwrap());
        let registry = Arc::new(stepflow_registry::RegistryImpl::new(db.clone()).await.unwrap());
        create_default_executor(db, registry)
    }
    
    #[tokio::test]
    async fn test_create_executor() {
        let executor = create_test_executor().await;
        assert!(executor.is_ok());
    }
    
    #[tokio::test]
    async fn test_execution_request_validation() {
        let executor = create_test_executor().await.unwrap();
        
        let request = ExecutionRequest {
            tool_id: ToolId::from_string("test-tool".to_string()),
            version: None,
            parameters: std::collections::HashMap::new(),
            context: ExecutionContext {
                user_id: "test-user".to_string(),
                tenant_id: "test-tenant".to_string(),
                session_id: "test-session".to_string(),
                request_id: "test-request".to_string(),
                parent_execution_id: None,
                environment: std::collections::HashMap::new(),
            },
            options: ExecutionOptions::default(),
        };
        
        // This should fail because the tool doesn't exist
        let result = executor.execute_tool_async(request).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_execution_context_creation() {
        let context = ExecutionContext {
            user_id: "test-user".to_string(),
            tenant_id: "test-tenant".to_string(),
            session_id: "test-session".to_string(),
            request_id: "test-request".to_string(),
            parent_execution_id: None,
            environment: std::collections::HashMap::new(),
        };
        
        assert_eq!(context.user_id, "test-user");
        assert_eq!(context.tenant_id, "test-tenant");
    }
    
    #[tokio::test]
    async fn test_execution_options_defaults() {
        let options = ExecutionOptions::default();
        assert_eq!(options.priority, Priority::Normal);
        assert_eq!(options.retry_count, 3);
    }
    
    #[tokio::test]
    async fn test_priority_ordering() {
        assert!(Priority::Critical > Priority::High);
        assert!(Priority::High > Priority::Normal);
        assert!(Priority::Normal > Priority::Low);
    }
    
    #[tokio::test]
    async fn test_resource_limits_default() {
        let limits = ResourceLimits::default();
        assert!(limits.memory_limit.is_none());
        assert!(limits.cpu_limit.is_none());
    }
    
    #[tokio::test]
    async fn test_metric_entry() {
        let metric = MetricEntry {
            name: "test_metric".to_string(),
            value: 42.0,
            unit: "count".to_string(),
            timestamp: chrono::Utc::now(),
            labels: std::collections::HashMap::new(),
        };
        
        assert_eq!(metric.name, "test_metric");
        assert_eq!(metric.value, 42.0);
    }
} 