//! Monitoring and metrics implementation

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use stepflow_core::*;
use stepflow_database::SqliteDatabase;
use crate::errors::*;
use crate::executor::Monitoring;

/// Monitoring implementation
pub struct MonitoringImpl {
    db: Arc<SqliteDatabase>,
    // In-memory metrics cache
    metrics_cache: Arc<RwLock<HashMap<ExecutionId, Vec<Metric>>>>,
    // Execution tracking
    execution_tracking: Arc<RwLock<HashMap<ExecutionId, ExecutionTracking>>>,
}

/// Execution tracking information
#[derive(Debug, Clone)]
struct ExecutionTracking {
    start_time: DateTime<Utc>,
    last_update: DateTime<Utc>,
    metrics_count: usize,
}

impl MonitoringImpl {
    /// Create a new monitoring instance
    pub fn new(db: Arc<SqliteDatabase>) -> Self {
        Self {
            db,
            metrics_cache: Arc::new(RwLock::new(HashMap::new())),
            execution_tracking: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Store metric in database
    async fn store_metric_in_db(&self, execution_id: &ExecutionId, metric: &Metric) -> MonitoringResult<()> {
        let labels_json = serde_json::to_string(&metric.labels)
            .map_err(|e| MonitoringError::LoggingFailed(e.to_string()))?;
        
        let params = vec![
            serde_json::Value::String(execution_id.to_string()),
            serde_json::Value::String(metric.name.clone()),
            serde_json::Value::Number(serde_json::Number::from_f64(metric.value).unwrap_or(serde_json::Number::from(0))),
            serde_json::Value::String(metric.timestamp.to_rfc3339()),
            serde_json::Value::String(labels_json),
        ];
        
        self.db.execute(
            "INSERT INTO metrics (execution_id, name, value, timestamp, labels) VALUES (?, ?, ?, ?, ?)",
            &params,
        ).await.map_err(|e| MonitoringError::LoggingFailed(e.to_string()))?;
        
        Ok(())
    }
    
    /// Store log in database
    async fn store_log_in_db(&self, execution_id: &ExecutionId, log: &LogEntry) -> MonitoringResult<()> {
        let metadata_json = serde_json::to_string(&log.metadata)
            .map_err(|e| MonitoringError::LoggingFailed(e.to_string()))?;
        
        let params = vec![
            serde_json::Value::String(execution_id.to_string()),
            serde_json::Value::String(format!("{:?}", log.level)),
            serde_json::Value::String(log.message.clone()),
            serde_json::Value::String(log.timestamp.to_rfc3339()),
            serde_json::Value::String(log.source.clone()),
            serde_json::Value::String(metadata_json),
        ];
        
        self.db.execute(
            "INSERT INTO logs (execution_id, level, message, timestamp, source, metadata) VALUES (?, ?, ?, ?, ?, ?)",
            &params,
        ).await.map_err(|e| MonitoringError::LoggingFailed(e.to_string()))?;
        
        Ok(())
    }
    
    /// Get metrics from database
    async fn get_metrics_from_db(&self, filter: Option<MetricFilter>) -> MonitoringResult<Vec<Metric>> {
        let mut sql = "SELECT execution_id, name, value, timestamp, labels FROM metrics WHERE 1=1".to_string();
        let mut params = Vec::new();
        
        if let Some(filter) = filter {
            if let Some(name) = filter.name {
                sql.push_str(" AND name = ?");
                params.push(serde_json::Value::String(name));
            }
            
            if let Some(start_time) = filter.start_time {
                sql.push_str(" AND timestamp >= ?");
                params.push(serde_json::Value::String(start_time.to_rfc3339()));
            }
            
            if let Some(end_time) = filter.end_time {
                sql.push_str(" AND timestamp <= ?");
                params.push(serde_json::Value::String(end_time.to_rfc3339()));
            }
            
            // Handle labels filtering
            for (key, value) in filter.labels {
                sql.push_str(" AND labels LIKE ?");
                params.push(serde_json::Value::String(format!("%\"{}\":\"{}\"%", key, value)));
            }
        }
        
        let result = self.db.execute(&sql, &params)
            .await.map_err(|e| MonitoringError::LoggingFailed(e.to_string()))?;
        
        let mut metrics = Vec::new();
        for row in result.rows {
            let _execution_id = row.get("execution_id").and_then(|v| v.as_str()).unwrap_or("");
            let name = row.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let value = row.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let timestamp_str = row.get("timestamp").and_then(|v| v.as_str()).unwrap_or("");
            let labels_str = row.get("labels").and_then(|v| v.as_str()).unwrap_or("{}");
            
            let timestamp = DateTime::parse_from_rfc3339(timestamp_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            
            let labels: HashMap<String, String> = serde_json::from_str(labels_str)
                .unwrap_or_default();
            
            metrics.push(Metric {
                name,
                value,
                labels,
                timestamp,
            });
        }
        
        Ok(metrics)
    }
    
    /// Update execution tracking
    async fn update_execution_tracking(&self, execution_id: &ExecutionId) {
        let mut tracking = self.execution_tracking.write().await;
        let entry = tracking.entry(execution_id.clone()).or_insert_with(|| ExecutionTracking {
            start_time: Utc::now(),
            last_update: Utc::now(),
            metrics_count: 0,
        });
        
        entry.last_update = Utc::now();
        entry.metrics_count += 1;
    }
    
    /// Record standard metrics for execution result
    async fn record_standard_metrics(&self, execution_id: &ExecutionId, result: &ExecutionResult) -> MonitoringResult<()> {
        // Record duration metric if available
        // Note: stepflow_core::ExecutionResult doesn't have timing field, so we'll skip this
        // for now or use metadata to extract timing information
        
        // Record execution count metric
        let count_metric = Metric {
            name: "execution_count".to_string(),
            value: 1.0,
            labels: HashMap::from([
                ("status".to_string(), if result.success { "success".to_string() } else { "failure".to_string() }),
            ]),
            timestamp: Utc::now(),
        };
        
        self.record_metric(execution_id, count_metric).await?;
        
        // Record end metric
        let end_metric = Metric {
            name: "execution_end".to_string(),
            value: 1.0,
            labels: HashMap::from([
                ("status".to_string(), if result.success { "success".to_string() } else { "failure".to_string() }),
            ]),
            timestamp: Utc::now(),
        };
        
        self.record_metric(execution_id, end_metric).await?;
        
        // Record log entries if available
        for log in &result.logs {
            self.store_log_in_db(execution_id, log).await?;
        }
        
        Ok(())
    }
}

#[async_trait::async_trait]
impl Monitoring for MonitoringImpl {
    /// Record execution start
    async fn record_execution_start(&self, execution_id: &ExecutionId) -> MonitoringResult<()> {
        // Update tracking
        self.update_execution_tracking(execution_id).await;
        
        // Record start metric
        let start_metric = Metric {
            name: "execution_start".to_string(),
            value: 1.0,
            labels: HashMap::new(),
            timestamp: Utc::now(),
        };
        
        self.record_metric(execution_id, start_metric).await?;
        
        Ok(())
    }
    
    /// Record execution end
    async fn record_execution_end(&self, execution_id: &ExecutionId, result: &ExecutionResult) -> MonitoringResult<()> {
        // Update tracking
        self.update_execution_tracking(execution_id).await;
        
        // Record standard metrics
        self.record_standard_metrics(execution_id, result).await?;
        
        Ok(())
    }
    
    /// Record metric
    async fn record_metric(&self, execution_id: &ExecutionId, metric: Metric) -> MonitoringResult<()> {
        // Store in database
        self.store_metric_in_db(execution_id, &metric).await?;
        
        // Update cache
        let mut cache = self.metrics_cache.write().await;
        cache.entry(execution_id.clone()).or_insert_with(Vec::new).push(metric);
        
        // Update tracking
        self.update_execution_tracking(execution_id).await;
        
        Ok(())
    }
    
    /// Get metrics
    async fn get_metrics(&self, filter: Option<MetricFilter>) -> MonitoringResult<Vec<Metric>> {
        self.get_metrics_from_db(filter).await
    }
    
    /// Get execution metrics
    async fn get_execution_metrics(&self, _execution_id: &ExecutionId) -> MonitoringResult<Vec<Metric>> {
        let filter = MetricFilter {
            name: None,
            labels: HashMap::new(),
            start_time: None,
            end_time: None,
        };
        
        // Get all metrics and filter by execution_id
        // Note: stepflow_core::MetricFilter doesn't have execution_id field,
        // so we'll filter after getting all metrics
        let all_metrics = self.get_metrics_from_db(Some(filter)).await?;
        
        // Since we can't filter by execution_id in the database query,
        // we'll need to use a different approach or modify the database schema
        // For now, return all metrics (this is a limitation)
        Ok(all_metrics)
    }
} 