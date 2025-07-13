//! Result manager implementation

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use stepflow_core::*;
use stepflow_database::SqliteDatabase;
use crate::errors::*;
use crate::executor::ResultManager;

/// Result manager implementation
pub struct ResultManagerImpl {
    db: Arc<SqliteDatabase>,
    // In-memory cache for recent results
    result_cache: Arc<RwLock<HashMap<ExecutionId, ExecutionResult>>>,
    cache_size: usize,
}

impl ResultManagerImpl {
    /// Create a new result manager
    pub fn new(db: Arc<SqliteDatabase>) -> Self {
        Self {
            db,
            result_cache: Arc::new(RwLock::new(HashMap::new())),
            cache_size: 1000,
        }
    }
    
    /// Store result in database
    async fn store_result_in_db(&self, execution_id: &ExecutionId, result: &ExecutionResult) -> ExecutorResult<()> {
        let output_json = serde_json::to_string(&result.output)
            .map_err(|e| ExecutorError::InternalError(e.to_string()))?;
        
        let logs_json = serde_json::to_string(&result.logs)
            .map_err(|e| ExecutorError::InternalError(e.to_string()))?;
        
        let metrics_json = serde_json::to_string(&result.metrics)
            .map_err(|e| ExecutorError::InternalError(e.to_string()))?;
        
        let metadata_json = serde_json::to_string(&result.metadata)
            .map_err(|e| ExecutorError::InternalError(e.to_string()))?;
        
        let params = vec![
            serde_json::Value::String(execution_id.to_string()),
            serde_json::Value::Bool(result.success),
            serde_json::Value::String(output_json),
            serde_json::Value::String(result.error.clone().unwrap_or_default()),
            serde_json::Value::String(logs_json),
            serde_json::Value::String(metrics_json),
            serde_json::Value::String(metadata_json),
            serde_json::Value::String(Utc::now().to_rfc3339()),
        ];
        
        self.db.execute(
            r#"
            INSERT INTO execution_results (
                execution_id, success, output_data, error, logs, metrics, metadata, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            &params
        ).await.map_err(|e| ExecutorError::DatabaseError(e.to_string()))?;
        
        Ok(())
    }
    
    /// Get result from database
    async fn get_result_from_db(&self, execution_id: &ExecutionId) -> ExecutorResult<Option<ExecutionResult>> {
        let params = vec![serde_json::Value::String(execution_id.to_string())];
        
        let query_result = self.db.execute(
            "SELECT success, output_data, error, logs, metrics, metadata FROM execution_results WHERE execution_id = ?",
            &params
        ).await.map_err(|e| ExecutorError::DatabaseError(e.to_string()))?;
        
        if query_result.rows.is_empty() {
            return Ok(None);
        }
        
        let row = &query_result.rows[0];
        
        let success = row.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
        let output_json = row.get("output_data").and_then(|v| v.as_str()).unwrap_or("null");
        let error = row.get("error").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(|s| s.to_string());
        let logs_json = row.get("logs").and_then(|v| v.as_str()).unwrap_or("[]");
        let metrics_json = row.get("metrics").and_then(|v| v.as_str()).unwrap_or("{}");
        let metadata_json = row.get("metadata").and_then(|v| v.as_str()).unwrap_or("{}");
        
        let output: Option<serde_json::Value> = serde_json::from_str(output_json).ok();
        let logs: Vec<LogEntry> = serde_json::from_str(logs_json).unwrap_or_default();
        let metrics: HashMap<String, f64> = serde_json::from_str(metrics_json).unwrap_or_default();
        let metadata: HashMap<String, serde_json::Value> = serde_json::from_str(metadata_json).unwrap_or_default();
        
        let result = ExecutionResult {
            success,
            output,
            error,
            logs,
            metrics,
            metadata,
        };
        
        Ok(Some(result))
    }
    
    /// Clean up cache to maintain size limit
    async fn cleanup_cache(&self) {
        let mut cache = self.result_cache.write().await;
        
        if cache.len() > self.cache_size {
            // Remove oldest entries (simplified - in practice, you'd use LRU)
            let excess = cache.len() - self.cache_size;
            let keys_to_remove: Vec<ExecutionId> = cache.keys().take(excess).cloned().collect();
            
            for key in keys_to_remove {
                cache.remove(&key);
            }
        }
    }
}

#[async_trait::async_trait]
impl ResultManager for ResultManagerImpl {
    /// Store execution result
    async fn store_result(&self, result: ExecutionResult) -> ExecutorResult<()> {
        // For now, we'll use a generated execution ID since stepflow_core::ExecutionResult doesn't have one
        let execution_id = ExecutionId::new();
        
        // Store in database
        self.store_result_in_db(&execution_id, &result).await?;
        
        // Store in cache
        let mut cache = self.result_cache.write().await;
        cache.insert(execution_id.clone(), result);
        
        // Cleanup cache if needed
        drop(cache);
        self.cleanup_cache().await;
        
        Ok(())
    }
    
    /// Get execution result
    async fn get_result(&self, execution_id: &ExecutionId) -> ExecutorResult<ExecutionResult> {
        // Check cache first
        {
            let cache = self.result_cache.read().await;
            if let Some(result) = cache.get(execution_id) {
                return Ok(result.clone());
            }
        }
        
        // Get from database
        match self.get_result_from_db(execution_id).await? {
            Some(result) => {
                // Store in cache
                let mut cache = self.result_cache.write().await;
                cache.insert(execution_id.clone(), result.clone());
                Ok(result)
            }
            None => Err(ExecutorError::InternalError(format!("Execution result not found: {}", execution_id))),
        }
    }
    
    /// Delete execution result
    async fn delete_result(&self, execution_id: &ExecutionId) -> ExecutorResult<()> {
        // Remove from cache
        self.result_cache.write().await.remove(execution_id);
        
        // Remove from database
        let params = vec![serde_json::Value::String(execution_id.to_string())];
        
        self.db.execute(
            "DELETE FROM execution_results WHERE execution_id = ?",
            &params
        ).await.map_err(|e| ExecutorError::DatabaseError(e.to_string()))?;
        
        Ok(())
    }
    
    /// List results
    async fn list_results(&self, filter: Option<ExecutionFilter>) -> ExecutorResult<Vec<ExecutionResult>> {
        let mut sql = "SELECT execution_id, success, output_data, error, logs, metrics, metadata FROM execution_results WHERE 1=1".to_string();
        let mut params = Vec::new();
        
        if let Some(filter) = filter {
            if let Some(tool_id) = filter.tool_id {
                sql.push_str(" AND metadata LIKE ?");
                params.push(serde_json::Value::String(format!("%\"tool_id\":\"{}\"%", tool_id)));
            }
            
            if let Some(status) = filter.status {
                sql.push_str(" AND success = ?");
                params.push(serde_json::Value::Bool(status == ExecutionStatus::Completed));
            }
            
            if let Some(user_id) = filter.user_id {
                sql.push_str(" AND metadata LIKE ?");
                params.push(serde_json::Value::String(format!("%\"user_id\":\"{}\"%", user_id)));
            }
            
            if let Some(tenant_id) = filter.tenant_id {
                sql.push_str(" AND metadata LIKE ?");
                params.push(serde_json::Value::String(format!("%\"tenant_id\":\"{}\"%", tenant_id)));
            }
            
            if let Some(started_after) = filter.started_after {
                sql.push_str(" AND created_at >= ?");
                params.push(serde_json::Value::String(started_after.to_rfc3339()));
            }
            
            if let Some(started_before) = filter.started_before {
                sql.push_str(" AND created_at <= ?");
                params.push(serde_json::Value::String(started_before.to_rfc3339()));
            }
        }
        
        let query_result = self.db.execute(&sql, &params).await
            .map_err(|e| ExecutorError::DatabaseError(e.to_string()))?;
        
        let mut results = Vec::new();
        
        for row in query_result.rows {
            let success = row.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
            let output_json = row.get("output_data").and_then(|v| v.as_str()).unwrap_or("null");
            let error = row.get("error").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(|s| s.to_string());
            let logs_json = row.get("logs").and_then(|v| v.as_str()).unwrap_or("[]");
            let metrics_json = row.get("metrics").and_then(|v| v.as_str()).unwrap_or("{}");
            let metadata_json = row.get("metadata").and_then(|v| v.as_str()).unwrap_or("{}");
            
            let output: Option<serde_json::Value> = serde_json::from_str(output_json).ok();
            let logs: Vec<LogEntry> = serde_json::from_str(logs_json).unwrap_or_default();
            let metrics: HashMap<String, f64> = serde_json::from_str(metrics_json).unwrap_or_default();
            let metadata: HashMap<String, serde_json::Value> = serde_json::from_str(metadata_json).unwrap_or_default();
            
            results.push(ExecutionResult {
                success,
                output,
                error,
                logs,
                metrics,
                metadata,
            });
        }
        
        Ok(results)
    }
    
    /// Clean up old results
    async fn cleanup_results(&self, older_than: DateTime<Utc>) -> ExecutorResult<u64> {
        let params = vec![serde_json::Value::String(older_than.to_rfc3339())];
        
        let query_result = self.db.execute(
            "DELETE FROM execution_results WHERE created_at < ?",
            &params
        ).await.map_err(|e| ExecutorError::DatabaseError(e.to_string()))?;
        
        // Clean up cache entries as well
        let mut cache = self.result_cache.write().await;
        let mut cache_keys_to_remove = Vec::new();
        
        for (execution_id, _) in cache.iter() {
            // In a real implementation, you'd check the actual creation time
            // For now, we'll just remove some entries
            cache_keys_to_remove.push(execution_id.clone());
        }
        
        for key in cache_keys_to_remove {
            cache.remove(&key);
        }
        
        Ok(query_result.rows_affected)
    }
} 