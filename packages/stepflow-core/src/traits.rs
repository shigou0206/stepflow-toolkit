//! Core traits for Stepflow Tool System

use async_trait::async_trait;
use std::collections::HashMap;
use crate::types::*;

/// Tool registry trait
#[async_trait]
pub trait ToolRegistry: Send + Sync {
    /// Register a tool
    async fn register_tool(&self, tool: Box<dyn Tool>) -> Result<ToolId, crate::StepflowError>;

    /// Unregister a tool
    async fn unregister_tool(&self, tool_id: &ToolId) -> Result<(), crate::StepflowError>;

    /// Get a tool by ID
    async fn get_tool(&self, tool_id: &ToolId) -> Result<Box<dyn Tool>, crate::StepflowError>;

    /// List all tools
    async fn list_tools(&self, filter: Option<ToolFilter>) -> Result<Vec<ToolInfo>, crate::StepflowError>;

    /// Search tools
    async fn search_tools(&self, query: &str) -> Result<Vec<ToolInfo>, crate::StepflowError>;

    /// Get tool statistics
    async fn get_tool_stats(&self, tool_id: &ToolId) -> Result<ToolStats, crate::StepflowError>;
}

/// Tool filter
#[derive(Debug, Clone)]
pub struct ToolFilter {
    pub tool_type: Option<ToolType>,
    pub status: Option<ToolStatus>,
    pub tags: Vec<String>,
    pub author: Option<String>,
    pub created_after: Option<chrono::DateTime<chrono::Utc>>,
    pub created_before: Option<chrono::DateTime<chrono::Utc>>,
}

/// Tool statistics
#[derive(Debug, Clone)]
pub struct ToolStats {
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub average_execution_time: f64,
    pub last_execution: Option<chrono::DateTime<chrono::Utc>>,
    pub popularity_score: f64,
}

/// Tool executor trait
#[async_trait]
pub trait ToolExecutor: Send + Sync {
    /// Execute a tool
    async fn execute_tool(&self, request: ToolRequest) -> Result<ToolResponse, crate::StepflowError>;

    /// Execute a tool with timeout
    async fn execute_tool_with_timeout(
        &self,
        request: ToolRequest,
        timeout: std::time::Duration,
    ) -> Result<ToolResponse, crate::StepflowError>;

    /// Execute multiple tools
    async fn execute_tools(
        &self,
        requests: Vec<ToolRequest>,
    ) -> Result<Vec<ToolResponse>, crate::StepflowError>;

    /// Cancel execution
    async fn cancel_execution(&self, execution_id: &ExecutionId) -> Result<(), crate::StepflowError>;

    /// Get execution status
    async fn get_execution_status(&self, execution_id: &ExecutionId) -> Result<ExecutionStatus, crate::StepflowError>;

    /// Get execution result
    async fn get_execution_result(&self, execution_id: &ExecutionId) -> Result<ExecutionResult, crate::StepflowError>;
}

/// Execution manager trait
#[async_trait]
pub trait ExecutionManager: Send + Sync {
    /// Create execution
    async fn create_execution(&self, request: ToolRequest) -> Result<Execution, crate::StepflowError>;

    /// Start execution
    async fn start_execution(&self, execution_id: &ExecutionId) -> Result<(), crate::StepflowError>;

    /// Complete execution
    async fn complete_execution(&self, execution_id: &ExecutionId, result: ExecutionResult) -> Result<(), crate::StepflowError>;

    /// Fail execution
    async fn fail_execution(&self, execution_id: &ExecutionId, error: String) -> Result<(), crate::StepflowError>;

    /// Get execution
    async fn get_execution(&self, execution_id: &ExecutionId) -> Result<Execution, crate::StepflowError>;

    /// List executions
    async fn list_executions(&self, filter: Option<ExecutionFilter>) -> Result<Vec<Execution>, crate::StepflowError>;

    /// Clean up old executions
    async fn cleanup_executions(&self, older_than: chrono::DateTime<chrono::Utc>) -> Result<u64, crate::StepflowError>;
}

/// Execution filter
#[derive(Debug, Clone)]
pub struct ExecutionFilter {
    pub tool_id: Option<ToolId>,
    pub tenant_id: Option<TenantId>,
    pub user_id: Option<UserId>,
    pub status: Option<ExecutionStatus>,
    pub started_after: Option<chrono::DateTime<chrono::Utc>>,
    pub started_before: Option<chrono::DateTime<chrono::Utc>>,
}

/// Tenant manager trait
#[async_trait]
pub trait TenantManager: Send + Sync {
    /// Create tenant
    async fn create_tenant(&self, info: TenantInfo) -> Result<TenantId, crate::StepflowError>;

    /// Get tenant
    async fn get_tenant(&self, tenant_id: &TenantId) -> Result<TenantInfo, crate::StepflowError>;

    /// Update tenant
    async fn update_tenant(&self, tenant_id: &TenantId, info: TenantInfo) -> Result<(), crate::StepflowError>;

    /// Delete tenant
    async fn delete_tenant(&self, tenant_id: &TenantId) -> Result<(), crate::StepflowError>;

    /// List tenants
    async fn list_tenants(&self, filter: Option<TenantFilter>) -> Result<Vec<TenantInfo>, crate::StepflowError>;
}

/// Tenant filter
#[derive(Debug, Clone)]
pub struct TenantFilter {
    pub domain: Option<String>,
    pub created_after: Option<chrono::DateTime<chrono::Utc>>,
    pub created_before: Option<chrono::DateTime<chrono::Utc>>,
}

/// User manager trait
#[async_trait]
pub trait UserManager: Send + Sync {
    /// Create user
    async fn create_user(&self, info: UserInfo) -> Result<UserId, crate::StepflowError>;

    /// Get user
    async fn get_user(&self, user_id: &UserId) -> Result<UserInfo, crate::StepflowError>;

    /// Update user
    async fn update_user(&self, user_id: &UserId, info: UserInfo) -> Result<(), crate::StepflowError>;

    /// Delete user
    async fn delete_user(&self, user_id: &UserId) -> Result<(), crate::StepflowError>;

    /// List users
    async fn list_users(&self, filter: Option<UserFilter>) -> Result<Vec<UserInfo>, crate::StepflowError>;

    /// Authenticate user
    async fn authenticate_user(&self, username: &str, password: &str) -> Result<UserInfo, crate::StepflowError>;

    /// Change password
    async fn change_password(&self, user_id: &UserId, old_password: &str, new_password: &str) -> Result<(), crate::StepflowError>;
}

/// User filter
#[derive(Debug, Clone)]
pub struct UserFilter {
    pub tenant_id: Option<TenantId>,
    pub role: Option<UserRole>,
    pub created_after: Option<chrono::DateTime<chrono::Utc>>,
    pub created_before: Option<chrono::DateTime<chrono::Utc>>,
}

/// Authentication trait
#[async_trait]
pub trait Authenticator: Send + Sync {
    /// Authenticate user
    async fn authenticate(&self, credentials: &Credentials) -> Result<AuthResult, crate::StepflowError>;

    /// Validate token
    async fn validate_token(&self, token: &str) -> Result<AuthResult, crate::StepflowError>;

    /// Refresh token
    async fn refresh_token(&self, token: &str) -> Result<AuthResult, crate::StepflowError>;

    /// Revoke token
    async fn revoke_token(&self, token: &str) -> Result<(), crate::StepflowError>;
}

/// Credentials
#[derive(Debug, Clone)]
pub struct Credentials {
    pub username: String,
    pub password: String,
    pub tenant_id: Option<TenantId>,
}

/// Authentication result
#[derive(Debug, Clone)]
pub struct AuthResult {
    pub user_id: UserId,
    pub tenant_id: TenantId,
    pub token: String,
    pub refresh_token: Option<String>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub permissions: Vec<String>,
}

/// Authorization trait
#[async_trait]
pub trait Authorizer: Send + Sync {
    /// Check permission
    async fn check_permission(&self, user_id: &UserId, resource: &str, action: &str) -> Result<bool, crate::StepflowError>;

    /// Grant permission
    async fn grant_permission(&self, user_id: &UserId, resource: &str, action: &str) -> Result<(), crate::StepflowError>;

    /// Revoke permission
    async fn revoke_permission(&self, user_id: &UserId, resource: &str, action: &str) -> Result<(), crate::StepflowError>;

    /// Get user permissions
    async fn get_user_permissions(&self, user_id: &UserId) -> Result<Vec<Permission>, crate::StepflowError>;
}

/// Permission
#[derive(Debug, Clone)]
pub struct Permission {
    pub resource: String,
    pub action: String,
    pub granted_at: chrono::DateTime<chrono::Utc>,
    pub granted_by: UserId,
}

/// Monitoring trait
#[async_trait]
pub trait Monitor: Send + Sync {
    /// Record metric
    async fn record_metric(&self, name: &str, value: f64, labels: &HashMap<String, String>) -> Result<(), crate::StepflowError>;

    /// Record event
    async fn record_event(&self, event: &Event) -> Result<(), crate::StepflowError>;

    /// Get metrics
    async fn get_metrics(&self, filter: Option<MetricFilter>) -> Result<Vec<Metric>, crate::StepflowError>;

    /// Get events
    async fn get_events(&self, filter: Option<EventFilter>) -> Result<Vec<Event>, crate::StepflowError>;
}

/// Event
#[derive(Debug, Clone)]
pub struct Event {
    pub name: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub user_id: Option<UserId>,
    pub tenant_id: Option<TenantId>,
    pub data: HashMap<String, serde_json::Value>,
}

/// Metric filter
#[derive(Debug, Clone)]
pub struct MetricFilter {
    pub name: Option<String>,
    pub labels: HashMap<String, String>,
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
}

/// Event filter
#[derive(Debug, Clone)]
pub struct EventFilter {
    pub name: Option<String>,
    pub user_id: Option<UserId>,
    pub tenant_id: Option<TenantId>,
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
}

/// Cache trait
#[async_trait]
pub trait Cache: Send + Sync {
    /// Get value
    async fn get<T>(&self, key: &str) -> Result<Option<T>, crate::StepflowError>
    where
        T: serde::de::DeserializeOwned + Send + Sync;

    /// Set value
    async fn set<T>(&self, key: &str, value: &T, ttl: Option<std::time::Duration>) -> Result<(), crate::StepflowError>
    where
        T: serde::Serialize + Send + Sync;

    /// Delete value
    async fn delete(&self, key: &str) -> Result<(), crate::StepflowError>;

    /// Clear cache
    async fn clear(&self) -> Result<(), crate::StepflowError>;

    /// Get cache statistics
    async fn get_stats(&self) -> Result<CacheStats, crate::StepflowError>;
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub size: usize,
    pub max_size: usize,
}

/// Database trait
#[async_trait]
pub trait Database: Send + Sync {
    /// Execute query
    async fn execute(&self, query: &str, params: &[serde_json::Value]) -> Result<QueryResult, crate::StepflowError>;

    /// Migrate database
    async fn migrate(&self, migrations: &[Migration]) -> Result<(), crate::StepflowError>;

    /// Get database statistics
    async fn get_stats(&self) -> Result<DatabaseStats, crate::StepflowError>;
}

/// Query result
#[derive(Debug, Clone)]
pub struct QueryResult {
    pub rows_affected: u64,
    pub last_insert_id: Option<i64>,
    pub rows: Vec<HashMap<String, serde_json::Value>>,
}

/// Migration
#[derive(Debug, Clone)]
pub struct Migration {
    pub version: u32,
    pub name: String,
    pub sql: String,
}

/// Database statistics
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    pub connections: u32,
    pub active_connections: u32,
    pub idle_connections: u32,
    pub total_queries: u64,
    pub slow_queries: u64,
    pub errors: u64,
} 