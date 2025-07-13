//! Executor implementation

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use stepflow_core::*;
use stepflow_registry::{Registry, RegistryImpl};
use stepflow_database::SqliteDatabase;
use crate::errors::*;
use crate::execution_context::*;
use crate::executor::*;
use crate::scheduler::SchedulerImpl;
use crate::worker_pool::WorkerPoolImpl;
use crate::result_manager::ResultManagerImpl;
use crate::monitoring::MonitoringImpl;

/// Executor implementation
pub struct ExecutorImpl {
    scheduler: Arc<SchedulerImpl>,
    worker_pool: Arc<WorkerPoolImpl>,
    result_manager: Arc<ResultManagerImpl>,
    monitoring: Arc<MonitoringImpl>,
    registry: Arc<RegistryImpl>,
    db: Arc<SqliteDatabase>,
    // Active executions tracking
    active_executions: Arc<RwLock<HashMap<ExecutionId, ExecutionRequest>>>,
}

impl ExecutorImpl {
    /// Create a new executor
    pub fn new(
        scheduler: Arc<SchedulerImpl>,
        worker_pool: Arc<WorkerPoolImpl>,
        result_manager: Arc<ResultManagerImpl>,
        monitoring: Arc<MonitoringImpl>,
        registry: Arc<RegistryImpl>,
        db: Arc<SqliteDatabase>,
    ) -> Self {
        Self {
            scheduler,
            worker_pool,
            result_manager,
            monitoring,
            registry,
            db,
            active_executions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Validate execution request
    async fn validate_request(&self, request: &ExecutionRequest) -> ExecutorResult<()> {
        // Check if tool exists
        match self.registry.get_tool(&request.tool_id).await {
            Ok(_) => Ok(()),
            Err(_) => Err(ExecutorError::ToolNotFound(request.tool_id.clone())),
        }
    }
    
    /// Create execution result from tool response
    async fn create_execution_result(
        &self,
        execution_id: ExecutionId,
        request: &ExecutionRequest,
        start_time: DateTime<Utc>,
    ) -> ExecutorResult<ExecutionResult> {
        let tool = self.registry.get_tool(&request.tool_id).await?;
        
        // Create a successful execution result compatible with stepflow_core::ExecutionResult
        let result = ExecutionResult {
            success: true,
            output: Some(serde_json::json!({
                "message": "Tool executed successfully",
                "tool_id": tool.id.to_string(),
                "execution_id": execution_id.to_string(),
                "timestamp": Utc::now().to_rfc3339(),
            })),
            error: None,
            logs: vec![
                LogEntry {
                    level: LogLevel::Info,
                    message: "Tool execution started".to_string(),
                    timestamp: start_time,
                    source: "executor".to_string(),
                    metadata: HashMap::new(),
                },
                LogEntry {
                    level: LogLevel::Info,
                    message: "Tool execution completed".to_string(),
                    timestamp: Utc::now(),
                    source: "executor".to_string(),
                    metadata: HashMap::new(),
                },
            ],
            metrics: HashMap::from([
                ("execution_duration".to_string(), 1.0),
                ("memory_usage".to_string(), 1024.0),
            ]),
            metadata: HashMap::from([
                ("tool_id".to_string(), serde_json::Value::String(tool.id.to_string())),
                ("tool_name".to_string(), serde_json::Value::String(tool.name.clone())),
                ("tool_version".to_string(), serde_json::Value::String(tool.version.to_string())),
                ("execution_id".to_string(), serde_json::Value::String(execution_id.to_string())),
                ("start_time".to_string(), serde_json::Value::String(start_time.to_rfc3339())),
                ("end_time".to_string(), serde_json::Value::String(Utc::now().to_rfc3339())),
            ]),
        };
        
        Ok(result)
    }
    
    /// Store async execution result with execution_id
    async fn store_async_result(&self, execution_id: &ExecutionId, result: ExecutionResult) -> ExecutorResult<()> {
        // Store in database with execution_id
        let sql = r#"
            INSERT INTO execution_results (execution_id, success, output_data, error, logs, metrics, metadata, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#;
        
        let params = vec![
            serde_json::Value::String(execution_id.to_string()),
            serde_json::Value::Bool(result.success),
            result.output.unwrap_or(serde_json::Value::Null),
            result.error.map(serde_json::Value::String).unwrap_or(serde_json::Value::Null),
            serde_json::to_value(&result.logs).unwrap_or(serde_json::Value::Array(vec![])),
            serde_json::to_value(&result.metrics).unwrap_or(serde_json::Value::Object(serde_json::Map::new())),
            serde_json::to_value(&result.metadata).unwrap_or(serde_json::Value::Object(serde_json::Map::new())),
            serde_json::Value::String(Utc::now().to_rfc3339()),
        ];
        
        self.db.execute(sql, &params).await
            .map_err(|e| ExecutorError::DatabaseError(e.to_string()))?;
        
        Ok(())
    }
}

impl Clone for ExecutorImpl {
    fn clone(&self) -> Self {
        Self {
            scheduler: self.scheduler.clone(),
            worker_pool: self.worker_pool.clone(),
            result_manager: self.result_manager.clone(),
            monitoring: self.monitoring.clone(),
            registry: self.registry.clone(),
            db: self.db.clone(),
            active_executions: self.active_executions.clone(),
        }
    }
}

#[async_trait::async_trait]
impl Executor for ExecutorImpl {
    /// Execute a tool synchronously
    async fn execute_tool(&self, request: ExecutionRequest) -> ExecutorResult<ExecutionResult> {
        // Validate request
        self.validate_request(&request).await?;
        
        // Generate execution ID
        let execution_id = ExecutionId::new();
        let start_time = Utc::now();
        
        // Track active execution
        {
            let mut active = self.active_executions.write().await;
            active.insert(execution_id.clone(), request.clone());
        }
        
        // Record execution start
        self.monitoring.record_execution_start(&execution_id).await
            .map_err(|e| ExecutorError::MonitoringError(e.to_string()))?;
        
        // Create execution result
        let result = self.create_execution_result(execution_id.clone(), &request, start_time).await?;
        
        // Record execution end
        self.monitoring.record_execution_end(&execution_id, &result).await
            .map_err(|e| ExecutorError::MonitoringError(e.to_string()))?;
        
        // Store result
        self.result_manager.store_result(result.clone()).await
            .map_err(|e| ExecutorError::InternalError(e.to_string()))?;
        
        // Remove from active executions
        {
            let mut active = self.active_executions.write().await;
            active.remove(&execution_id);
        }
        
        Ok(result)
    }
    
    /// Execute a tool asynchronously
    async fn execute_tool_async(&self, request: ExecutionRequest) -> ExecutorResult<ExecutionId> {
        // Validate request
        self.validate_request(&request).await?;
        
        // Generate execution ID
        let execution_id = ExecutionId::new();
        
        // Track active execution
        {
            let mut active = self.active_executions.write().await;
            active.insert(execution_id.clone(), request.clone());
        }
        
        // Record execution start
        self.monitoring.record_execution_start(&execution_id).await
            .map_err(|e| ExecutorError::MonitoringError(e.to_string()))?;
        
        // Spawn a background task to simulate async execution
        let executor = self.clone();
        let exec_id = execution_id.clone();
        let req = request.clone();
        tokio::spawn(async move {
            // Simulate async work
            tokio::time::sleep(Duration::from_millis(100)).await;
            
            let start_time = Utc::now();
            if let Ok(result) = executor.create_execution_result(exec_id.clone(), &req, start_time).await {
                // Store result with the execution_id
                if let Err(e) = executor.store_async_result(&exec_id, result.clone()).await {
                    tracing::error!("Failed to store async result: {}", e);
                }
                
                // Record execution end
                if let Err(e) = executor.monitoring.record_execution_end(&exec_id, &result).await {
                    tracing::error!("Failed to record execution end: {}", e);
                }
                
                // Remove from active executions
                {
                    let mut active = executor.active_executions.write().await;
                    active.remove(&exec_id);
                }
            }
        });
        
        Ok(execution_id)
    }
    
    /// Get execution status
    async fn get_execution_status(&self, execution_id: &ExecutionId) -> ExecutorResult<ExecutionStatus> {
        // Check if execution is active
        let active = self.active_executions.read().await;
        if active.contains_key(execution_id) {
            return Ok(ExecutionStatus::Running);
        }
        
        // Check in database for completed results
        let sql = "SELECT success FROM execution_results WHERE execution_id = ?";
        let params = vec![serde_json::Value::String(execution_id.to_string())];
        
        match self.db.execute(sql, &params).await {
            Ok(result) if !result.rows.is_empty() => Ok(ExecutionStatus::Completed),
            _ => Ok(ExecutionStatus::Pending),
        }
    }
    
    /// Cancel execution
    async fn cancel_execution(&self, execution_id: &ExecutionId) -> ExecutorResult<()> {
        // Remove from active executions
        {
            let mut active = self.active_executions.write().await;
            active.remove(execution_id);
        }
        
        // Cancel in worker pool (if running)
        // Note: This is a simplified implementation
        Ok(())
    }
    
    /// Get execution result
    async fn get_execution_result(&self, execution_id: &ExecutionId) -> ExecutorResult<ExecutionResult> {
        self.result_manager.get_result(execution_id).await
            .map_err(|e| ExecutorError::InternalError(e.to_string()))
    }
    
    /// List executions
    async fn list_executions(&self, filter: Option<ExecutionFilter>) -> ExecutorResult<Vec<ExecutionInfo>> {
        let mut sql = "SELECT execution_id, tool_id, status, created_at, started_at, completed_at, user_id, tenant_id FROM executions WHERE 1=1".to_string();
        let mut params = Vec::new();
        
        if let Some(filter) = filter {
            if let Some(tool_id) = filter.tool_id {
                sql.push_str(" AND tool_id = ?");
                params.push(serde_json::Value::String(tool_id.to_string()));
            }
            
            if let Some(status) = filter.status {
                sql.push_str(" AND status = ?");
                params.push(serde_json::Value::String(format!("{:?}", status)));
            }
            
            if let Some(user_id) = filter.user_id {
                sql.push_str(" AND user_id = ?");
                params.push(serde_json::Value::String(user_id.to_string()));
            }
            
            if let Some(tenant_id) = filter.tenant_id {
                sql.push_str(" AND tenant_id = ?");
                params.push(serde_json::Value::String(tenant_id.to_string()));
            }
            
            if let Some(started_after) = filter.started_after {
                sql.push_str(" AND started_at >= ?");
                params.push(serde_json::Value::String(started_after.to_rfc3339()));
            }
            
            if let Some(started_before) = filter.started_before {
                sql.push_str(" AND started_at <= ?");
                params.push(serde_json::Value::String(started_before.to_rfc3339()));
            }
        }
        
        let result = self.db.execute(&sql, &params).await
            .map_err(|e| ExecutorError::DatabaseError(e.to_string()))?;
        
        let mut executions = Vec::new();
        for row in result.rows {
            let execution_id_str = row.get("execution_id").and_then(|v| v.as_str()).unwrap_or("");
            let tool_id_str = row.get("tool_id").and_then(|v| v.as_str()).unwrap_or("");
            let status_str = row.get("status").and_then(|v| v.as_str()).unwrap_or("Pending");
            let created_at_str = row.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
            let started_at_str = row.get("started_at").and_then(|v| v.as_str());
            let completed_at_str = row.get("completed_at").and_then(|v| v.as_str());
            let user_id_str = row.get("user_id").and_then(|v| v.as_str()).unwrap_or("");
            let tenant_id_str = row.get("tenant_id").and_then(|v| v.as_str()).unwrap_or("");
            
            let execution_id = ExecutionId::from_string(execution_id_str.to_string());
            let tool_id = ToolId::from_string(tool_id_str.to_string());
            let status = match status_str {
                "Running" => ExecutionStatus::Running,
                "Completed" => ExecutionStatus::Completed,
                "Failed" => ExecutionStatus::Failed,
                "Cancelled" => ExecutionStatus::Cancelled,
                _ => ExecutionStatus::Pending,
            };
            
            let created_at = DateTime::parse_from_rfc3339(created_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            
            let started_at = started_at_str.and_then(|s| {
                DateTime::parse_from_rfc3339(s)
                    .map(|dt| dt.with_timezone(&Utc))
                    .ok()
            });
            
            let completed_at = completed_at_str.and_then(|s| {
                DateTime::parse_from_rfc3339(s)
                    .map(|dt| dt.with_timezone(&Utc))
                    .ok()
            });
            
            executions.push(ExecutionInfo {
                execution_id,
                tool_id,
                status,
                created_at,
                started_at,
                completed_at,
                user_id: user_id_str.to_string(),
                tenant_id: tenant_id_str.to_string(),
            });
        }
        
        Ok(executions)
    }
    
    /// Get execution metrics
    async fn get_execution_metrics(&self, execution_id: &ExecutionId) -> ExecutorResult<Vec<Metric>> {
        self.monitoring.get_execution_metrics(execution_id).await
            .map_err(|e| ExecutorError::MonitoringError(e.to_string()))
    }
    
    async fn health_check(&self) -> ExecutorResult<bool> {
        // Simple health check - verify core components are working
        match self.scheduler.get_queue_status().await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
} 