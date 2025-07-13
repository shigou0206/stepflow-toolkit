//! Database connection management

use sqlx::{Sqlite, SqlitePool, Row, Column, TypeInfo};
use stepflow_core::{Database, QueryResult, Migration, DatabaseStats, StepflowError, StepflowResult};
use tracing::{debug, info, error};
use std::time::{Duration, Instant};
use base64::Engine;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// SQLite database connection configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connection_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "sqlite::memory:".to_string(),
            max_connections: 10,
            min_connections: 2,
            connection_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600),
            max_lifetime: Duration::from_secs(1800),
        }
    }
}

/// SQLite database connection manager
#[derive(Clone)]
pub struct SqliteDatabase {
    pool: SqlitePool,
    config: DatabaseConfig,
    stats: Arc<DatabaseStatsTracker>,
}

/// Database statistics tracker
#[derive(Debug)]
pub struct DatabaseStatsTracker {
    pub total_queries: AtomicU64,
    pub slow_queries: AtomicU64,
    pub errors: AtomicU64,
    pub slow_query_threshold: Duration,
}

impl SqliteDatabase {
    /// Create a new SQLite database connection with default configuration
    pub async fn new(database_url: &str) -> Result<Self, StepflowError> {
        let config = DatabaseConfig {
            url: database_url.to_string(),
            ..Default::default()
        };
        Self::with_config(config).await
    }

    /// Create a new SQLite database connection with custom configuration
    pub async fn with_config(config: DatabaseConfig) -> Result<Self, StepflowError> {
        let pool = SqlitePool::connect(&config.url)
            .await
            .map_err(|e| StepflowError::DatabaseError(stepflow_core::DatabaseError::ConnectionFailed(
                format!("Failed to connect to database: {}", e)
            )))?;

        let stats = Arc::new(DatabaseStatsTracker {
            total_queries: AtomicU64::new(0),
            slow_queries: AtomicU64::new(0),
            errors: AtomicU64::new(0),
            slow_query_threshold: Duration::from_millis(1000), // 1 second threshold
        });

        info!("Connected to SQLite database with {} max connections", config.max_connections);
        Ok(Self { pool, config, stats })
    }

    /// Get the connection pool
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Get the database configuration
    pub fn config(&self) -> &DatabaseConfig {
        &self.config
    }

    /// Check if the database connection is healthy
    pub async fn health_check(&self) -> Result<bool, StepflowError> {
        match sqlx::query("SELECT 1").execute(&self.pool).await {
            Ok(_) => {
                debug!("Database health check passed");
                Ok(true)
            }
            Err(e) => {
                error!("Database health check failed: {}", e);
                Err(StepflowError::DatabaseError(stepflow_core::DatabaseError::ConnectionFailed(
                    format!("Health check failed: {}", e)
                )))
            }
        }
    }

    /// Get connection pool statistics
    pub async fn get_pool_stats(&self) -> Result<PoolStats, StepflowError> {
        // Note: sqlx doesn't provide direct pool statistics, so we'll return basic info
        Ok(PoolStats {
            size: self.config.max_connections,
            idle: 0, // Not available in sqlx
            active: 0, // Not available in sqlx
        })
    }

    /// Close the database connection
    pub async fn close(&self) -> Result<(), StepflowError> {
        info!("Closing database connection pool");
        self.pool.close().await;
        Ok(())
    }
}

/// Connection pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub size: u32,
    pub idle: u32,
    pub active: u32,
}

#[async_trait::async_trait]
impl Database for SqliteDatabase {
    async fn execute(&self, query: &str, params: &[serde_json::Value]) -> StepflowResult<QueryResult> {
        debug!("Executing query: {}", query);
        
        let start_time = Instant::now();
        self.stats.total_queries.fetch_add(1, Ordering::Relaxed);

        // 检查查询类型
        let query_type = query.trim().to_lowercase();
        let is_select = query_type.starts_with("select");
        
        let result = if is_select {
            // 处理SELECT查询
            self.execute_select_query(query, params).await
        } else {
            // 处理INSERT/UPDATE/DELETE查询
            self.execute_modify_query(query, params).await
        };

        let duration = start_time.elapsed();
        
        // 跟踪慢查询
        if duration > self.stats.slow_query_threshold {
            self.stats.slow_queries.fetch_add(1, Ordering::Relaxed);
            debug!("Slow query detected: {} (took {:?})", query, duration);
        }

        // 跟踪错误
        if result.is_err() {
            self.stats.errors.fetch_add(1, Ordering::Relaxed);
        }

        result
    }

    async fn migrate(&self, migrations: &[Migration]) -> StepflowResult<()> {
        for migration in migrations {
            debug!("Running migration: {}", migration.name);
            
            sqlx::query(&migration.sql)
                .execute(&self.pool)
                .await
                .map_err(|e| StepflowError::DatabaseError(stepflow_core::DatabaseError::MigrationFailed(
                    format!("Migration {} failed: {}", migration.name, e)
                )))?;
        }

        Ok(())
    }

    async fn get_stats(&self) -> StepflowResult<DatabaseStats> {
        let pool_stats = self.get_pool_stats().await?;
        
        Ok(DatabaseStats {
            connections: pool_stats.size,
            active_connections: pool_stats.active,
            idle_connections: pool_stats.idle,
            total_queries: self.stats.total_queries.load(Ordering::Relaxed),
            slow_queries: self.stats.slow_queries.load(Ordering::Relaxed),
            errors: self.stats.errors.load(Ordering::Relaxed),
        })
    }
}

impl SqliteDatabase {
    /// Execute a SELECT query and return rows
    async fn execute_select_query(&self, query: &str, params: &[serde_json::Value]) -> StepflowResult<QueryResult> {
        let mut query_builder = sqlx::query(query);
        
        // 绑定参数 - 直接使用值而不是序列化
        for param in params {
            match param {
                serde_json::Value::String(s) => query_builder = query_builder.bind(s),
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        query_builder = query_builder.bind(i);
                    } else if let Some(f) = n.as_f64() {
                        query_builder = query_builder.bind(f);
                    } else {
                        query_builder = query_builder.bind(n.to_string());
                    }
                }
                serde_json::Value::Bool(b) => query_builder = query_builder.bind(b),
                serde_json::Value::Null => query_builder = query_builder.bind(None::<String>),
                _ => query_builder = query_builder.bind(serde_json::to_string(param)?),
            }
        }

        let rows = query_builder.fetch_all(&self.pool).await
            .map_err(|e| StepflowError::DatabaseError(stepflow_core::DatabaseError::QueryFailed(
                format!("Query execution failed: {}", e)
            )))?;

        // 将行转换为HashMap<String, serde_json::Value>
        let mut result_rows = Vec::new();
        for row in rows {
            let mut row_map = std::collections::HashMap::new();
            
            // 获取列信息
            let columns = row.columns();
            for (i, column) in columns.iter().enumerate() {
                let column_name = column.name().to_string();
                
                // 根据列类型获取值
                let value = match column.type_info().name() {
                    "TEXT" => {
                        let val: Option<String> = row.try_get(i).ok();
                        val.map(serde_json::Value::String).unwrap_or(serde_json::Value::Null)
                    }
                    "INTEGER" => {
                        let val: Option<i64> = row.try_get(i).ok();
                        val.map(|v| serde_json::Value::Number(serde_json::Number::from(v)))
                            .unwrap_or(serde_json::Value::Null)
                    }
                    "REAL" => {
                        let val: Option<f64> = row.try_get(i).ok();
                        val.and_then(|v| serde_json::Number::from_f64(v))
                            .map(serde_json::Value::Number)
                            .unwrap_or(serde_json::Value::Null)
                    }
                    "BLOB" => {
                        let val: Option<Vec<u8>> = row.try_get(i).ok();
                        val.map(|v| serde_json::Value::String(base64::engine::general_purpose::STANDARD.encode(v)))
                            .unwrap_or(serde_json::Value::Null)
                    }
                    _ => {
                        // 尝试作为字符串获取
                        let val: Option<String> = row.try_get(i).ok();
                        val.map(serde_json::Value::String).unwrap_or(serde_json::Value::Null)
                    }
                };
                
                row_map.insert(column_name, value);
            }
            
            result_rows.push(row_map);
        }

        Ok(QueryResult {
            rows_affected: result_rows.len() as u64,
            last_insert_id: None,
            rows: result_rows,
        })
    }

    /// Execute a modify query (INSERT/UPDATE/DELETE)
    async fn execute_modify_query(&self, query: &str, params: &[serde_json::Value]) -> StepflowResult<QueryResult> {
        let mut query_builder = sqlx::query(query);
        
        // 绑定参数 - 直接使用值而不是序列化
        for param in params {
            match param {
                serde_json::Value::String(s) => query_builder = query_builder.bind(s),
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        query_builder = query_builder.bind(i);
                    } else if let Some(f) = n.as_f64() {
                        query_builder = query_builder.bind(f);
                    } else {
                        query_builder = query_builder.bind(n.to_string());
                    }
                }
                serde_json::Value::Bool(b) => query_builder = query_builder.bind(b),
                serde_json::Value::Null => query_builder = query_builder.bind(None::<String>),
                _ => query_builder = query_builder.bind(serde_json::to_string(param)?),
            }
        }

        let result = query_builder.execute(&self.pool).await
            .map_err(|e| StepflowError::DatabaseError(stepflow_core::DatabaseError::QueryFailed(
                format!("Query execution failed: {}", e)
            )))?;

        Ok(QueryResult {
            rows_affected: result.rows_affected(),
            last_insert_id: Some(result.last_insert_rowid()),
            rows: Vec::new(),
        })
    }

    /// Execute a transaction with a callback
    pub async fn execute_transaction<F, Fut>(&self, callback: F) -> StepflowResult<()>
    where
        F: FnOnce(&mut sqlx::Transaction<'_, Sqlite>) -> Fut + Send,
        Fut: std::future::Future<Output = StepflowResult<()>> + Send,
    {
        let mut transaction = self.pool.begin().await
            .map_err(|e| StepflowError::DatabaseError(stepflow_core::DatabaseError::TransactionFailed(
                format!("Failed to begin transaction: {}", e)
            )))?;
        
        let result = callback(&mut transaction).await?;
        
        transaction.commit().await
            .map_err(|e| StepflowError::DatabaseError(stepflow_core::DatabaseError::TransactionFailed(
                format!("Failed to commit transaction: {}", e)
            )))?;
        
        Ok(result)
    }

    /// Execute a read-only transaction
    pub async fn execute_read_transaction<F, Fut, T>(&self, callback: F) -> StepflowResult<T>
    where
        F: FnOnce(&mut sqlx::Transaction<'_, Sqlite>) -> Fut + Send,
        Fut: std::future::Future<Output = StepflowResult<T>> + Send,
        T: Send,
    {
        let mut transaction = self.pool.begin().await
            .map_err(|e| StepflowError::DatabaseError(stepflow_core::DatabaseError::TransactionFailed(
                format!("Failed to begin read transaction: {}", e)
            )))?;
        
        let result = callback(&mut transaction).await?;
        
        transaction.rollback().await
            .map_err(|e| StepflowError::DatabaseError(stepflow_core::DatabaseError::TransactionFailed(
                format!("Failed to rollback read transaction: {}", e)
            )))?;
        
        Ok(result)
    }
} 