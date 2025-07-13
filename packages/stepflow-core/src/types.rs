//! Core type definitions for Stepflow Tool System

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Unique identifier for a tool
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ToolId(String);

impl ToolId {
    /// Create a new tool ID
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    /// Create from string
    pub fn from_string(s: String) -> Self {
        Self(s)
    }

    /// Get as string reference
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ToolId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for ToolId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Tool version information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub pre_release: Option<String>,
    pub build: Option<String>,
}

impl ToolVersion {
    /// Create a new version
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            pre_release: None,
            build: None,
        }
    }

    /// Create with pre-release
    pub fn with_pre_release(mut self, pre_release: String) -> Self {
        self.pre_release = Some(pre_release);
        self
    }

    /// Create with build
    pub fn with_build(mut self, build: String) -> Self {
        self.build = Some(build);
        self
    }
}

impl std::fmt::Display for ToolVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(pre) = &self.pre_release {
            write!(f, "-{}", pre)?;
        }
        if let Some(build) = &self.build {
            write!(f, "+{}", build)?;
        }
        Ok(())
    }
}

/// Tool type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolType {
    OpenAPI,
    AsyncAPI,
    Python,
    Shell,
    AI,
    System,
    Custom(String),
}

impl std::fmt::Display for ToolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolType::OpenAPI => write!(f, "openapi"),
            ToolType::AsyncAPI => write!(f, "asyncapi"),
            ToolType::Python => write!(f, "python"),
            ToolType::Shell => write!(f, "shell"),
            ToolType::AI => write!(f, "ai"),
            ToolType::System => write!(f, "system"),
            ToolType::Custom(s) => write!(f, "custom:{}", s),
        }
    }
}

/// Tool status enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolStatus {
    Active,
    Inactive,
    Deprecated,
    Error,
}

impl std::fmt::Display for ToolStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolStatus::Active => write!(f, "active"),
            ToolStatus::Inactive => write!(f, "inactive"),
            ToolStatus::Deprecated => write!(f, "deprecated"),
            ToolStatus::Error => write!(f, "error"),
        }
    }
}

/// Tool information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub id: ToolId,
    pub name: String,
    pub description: String,
    pub version: ToolVersion,
    pub tool_type: ToolType,
    pub status: ToolStatus,
    pub author: String,
    pub repository: Option<String>,
    pub documentation: Option<String>,
    pub tags: Vec<String>,
    pub capabilities: Vec<String>,
    pub configuration_schema: Option<serde_json::Value>,
    pub examples: Vec<ToolExample>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Tool example
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExample {
    pub name: String,
    pub description: String,
    pub input: serde_json::Value,
    pub output: serde_json::Value,
}

/// Tool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    pub tool_id: ToolId,
    pub configuration: HashMap<String, serde_json::Value>,
    pub environment: HashMap<String, String>,
    pub secrets: HashMap<String, String>,
    pub timeout: Option<u64>,
    pub retries: Option<u32>,
    pub enabled: bool,
}

/// Tool execution request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRequest {
    pub tool_id: ToolId,
    pub input: serde_json::Value,
    pub configuration: Option<HashMap<String, serde_json::Value>>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Tool execution response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResponse {
    pub success: bool,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
    pub execution_time: u64,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Tool trait - the core interface for all tools
#[async_trait::async_trait]
pub trait Tool: Send + Sync {
    /// Get tool information
    async fn get_info(&self) -> Result<ToolInfo, crate::StepflowError>;

    /// Execute the tool
    async fn execute(&self, request: ToolRequest) -> Result<ToolResponse, crate::StepflowError>;

    /// Validate input
    async fn validate_input(&self, input: &serde_json::Value) -> Result<bool, crate::StepflowError>;

    /// Get configuration schema
    async fn get_configuration_schema(&self) -> Result<Option<serde_json::Value>, crate::StepflowError>;

    /// Test the tool
    async fn test(&self) -> Result<bool, crate::StepflowError>;
}

/// Unique identifier for an execution
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ExecutionId(String);

impl ExecutionId {
    /// Create a new execution ID
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    /// Create from string
    pub fn from_string(s: String) -> Self {
        Self(s)
    }

    /// Get as string reference
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ExecutionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Execution status enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    Timeout,
}

impl std::fmt::Display for ExecutionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionStatus::Pending => write!(f, "pending"),
            ExecutionStatus::Running => write!(f, "running"),
            ExecutionStatus::Completed => write!(f, "completed"),
            ExecutionStatus::Failed => write!(f, "failed"),
            ExecutionStatus::Cancelled => write!(f, "cancelled"),
            ExecutionStatus::Timeout => write!(f, "timeout"),
        }
    }
}

/// Execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
    pub logs: Vec<LogEntry>,
    pub metrics: HashMap<String, f64>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub source: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Log level enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "trace"),
            LogLevel::Debug => write!(f, "debug"),
            LogLevel::Info => write!(f, "info"),
            LogLevel::Warn => write!(f, "warn"),
            LogLevel::Error => write!(f, "error"),
            LogLevel::Fatal => write!(f, "fatal"),
        }
    }
}

/// Execution information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Execution {
    pub id: ExecutionId,
    pub tool_id: ToolId,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub status: ExecutionStatus,
    pub request: ToolRequest,
    pub result: Option<ExecutionResult>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Unique identifier for a tenant
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TenantId(String);

impl TenantId {
    /// Create a new tenant ID
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    /// Create from string
    pub fn from_string(s: String) -> Self {
        Self(s)
    }

    /// Get as string reference
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for TenantId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Tenant information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantInfo {
    pub id: TenantId,
    pub name: String,
    pub description: String,
    pub domain: Option<String>,
    pub settings: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Unique identifier for a user
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(String);

impl UserId {
    /// Create a new user ID
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    /// Create from string
    pub fn from_string(s: String) -> Self {
        Self(s)
    }

    /// Get as string reference
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// User role enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserRole {
    Admin,
    User,
    Guest,
    Custom(String),
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::Admin => write!(f, "admin"),
            UserRole::User => write!(f, "user"),
            UserRole::Guest => write!(f, "guest"),
            UserRole::Custom(s) => write!(f, "custom:{}", s),
        }
    }
}

/// User information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: UserId,
    pub username: String,
    pub email: String,
    pub role: UserRole,
    pub tenant_id: TenantId,
    pub settings: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Pagination parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub page: u32,
    pub size: u32,
    pub total: Option<u64>,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            page: 1,
            size: 20,
            total: None,
        }
    }
}

/// Filter parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filter {
    pub field: String,
    pub operator: FilterOperator,
    pub value: serde_json::Value,
}

/// Filter operator enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterOperator {
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Contains,
    NotContains,
    In,
    NotIn,
    IsNull,
    IsNotNull,
}

impl std::fmt::Display for FilterOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FilterOperator::Equal => write!(f, "eq"),
            FilterOperator::NotEqual => write!(f, "ne"),
            FilterOperator::GreaterThan => write!(f, "gt"),
            FilterOperator::LessThan => write!(f, "lt"),
            FilterOperator::GreaterThanOrEqual => write!(f, "gte"),
            FilterOperator::LessThanOrEqual => write!(f, "lte"),
            FilterOperator::Contains => write!(f, "contains"),
            FilterOperator::NotContains => write!(f, "not_contains"),
            FilterOperator::In => write!(f, "in"),
            FilterOperator::NotIn => write!(f, "not_in"),
            FilterOperator::IsNull => write!(f, "is_null"),
            FilterOperator::IsNotNull => write!(f, "is_not_null"),
        }
    }
}

/// Sort parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sort {
    pub field: String,
    pub direction: SortDirection,
}

/// Sort direction enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortDirection {
    Asc,
    Desc,
}

impl std::fmt::Display for SortDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SortDirection::Asc => write!(f, "asc"),
            SortDirection::Desc => write!(f, "desc"),
        }
    }
}

/// Query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Query {
    pub filters: Vec<Filter>,
    pub sorts: Vec<Sort>,
    pub pagination: Pagination,
}

impl Default for Query {
    fn default() -> Self {
        Self {
            filters: Vec::new(),
            sorts: Vec::new(),
            pagination: Pagination::default(),
        }
    }
}

/// Tool statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolStats {
    pub total_tools: u64,
    pub active_tools: u64,
    pub deprecated_tools: u64,
    pub error_tools: u64,
}

/// Metric information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub value: f64,
    pub labels: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
} 