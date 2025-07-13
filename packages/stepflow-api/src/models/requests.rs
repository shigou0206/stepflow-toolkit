use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use stepflow_core::{ToolId, UserId};
use crate::types::{FilterParams, PaginationParams};
use std::collections::HashMap;

/// 注册工具请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterToolRequest {
    pub name: String,
    pub description: String,
    pub tool_type: String,
    pub version: String,
    pub author: String,
    pub license: String,
    pub repository: Option<String>,
    pub documentation: Option<String>,
    pub tags: Vec<String>,
    pub configuration: serde_json::Value,
    pub requirements: HashMap<String, String>,
    pub metadata: HashMap<String, String>,
}

/// 更新工具请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateToolRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,
    pub author: Option<String>,
    pub license: Option<String>,
    pub repository: Option<String>,
    pub documentation: Option<String>,
    pub tags: Option<Vec<String>>,
    pub configuration: Option<serde_json::Value>,
    pub requirements: Option<HashMap<String, String>>,
    pub metadata: Option<HashMap<String, String>>,
}

/// 列出工具请求参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListToolsParams {
    #[serde(flatten)]
    pub pagination: PaginationParams,
    #[serde(flatten)]
    pub filter: FilterParams,
    pub tool_type: Option<String>,
    pub author: Option<String>,
    pub tags: Option<Vec<String>>,
    pub status: Option<String>,
}

/// 执行工具请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteToolRequest {
    pub tool_id: ToolId,
    pub version: Option<String>,
    pub inputs: HashMap<String, serde_json::Value>,
    pub configuration: Option<serde_json::Value>,
    pub sandbox_config: Option<SandboxConfigRequest>,
    pub timeout: Option<u64>,
    pub priority: Option<u32>,
    pub metadata: Option<HashMap<String, String>>,
}

/// 沙箱配置请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfigRequest {
    pub isolation_type: String,
    pub resource_limits: Option<ResourceLimitsRequest>,
    pub network_config: Option<NetworkConfigRequest>,
    pub storage_config: Option<StorageConfigRequest>,
    pub security_policy: Option<SecurityPolicyRequest>,
}

/// 资源限制请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimitsRequest {
    pub memory_limit: Option<u64>,
    pub cpu_limit: Option<f64>,
    pub disk_limit: Option<u64>,
    pub network_limit: Option<u64>,
    pub process_limit: Option<u32>,
    pub file_descriptor_limit: Option<u32>,
    pub execution_timeout: Option<u64>,
}

/// 网络配置请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfigRequest {
    pub enable_network: bool,
    pub allowed_hosts: Option<Vec<String>>,
    pub blocked_hosts: Option<Vec<String>>,
    pub allowed_ports: Option<Vec<u16>>,
    pub blocked_ports: Option<Vec<u16>>,
    pub dns_servers: Option<Vec<String>>,
}

/// 存储配置请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfigRequest {
    pub read_only: bool,
    pub temp_directory: Option<String>,
    pub allowed_paths: Option<Vec<String>>,
    pub blocked_paths: Option<Vec<String>>,
    pub max_file_size: Option<u64>,
    pub max_total_size: Option<u64>,
}

/// 安全策略请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicyRequest {
    pub allow_system_calls: bool,
    pub allowed_syscalls: Option<Vec<String>>,
    pub blocked_syscalls: Option<Vec<String>>,
    pub allow_network_access: bool,
    pub allow_file_access: bool,
    pub allow_process_spawn: bool,
    pub capabilities: Option<Vec<String>>,
}

/// 列出执行请求参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListExecutionsParams {
    #[serde(flatten)]
    pub pagination: PaginationParams,
    #[serde(flatten)]
    pub filter: FilterParams,
    pub tool_id: Option<ToolId>,
    pub user_id: Option<UserId>,
    pub status: Option<String>,
    pub priority: Option<u32>,
}

/// 搜索工具请求参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchToolsParams {
    pub query: String,
    pub tool_type: Option<String>,
    pub author: Option<String>,
    pub tags: Option<Vec<String>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// 创建工具版本请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateToolVersionRequest {
    pub version: String,
    pub description: Option<String>,
    pub changelog: Option<String>,
    pub configuration: Option<serde_json::Value>,
    pub requirements: Option<HashMap<String, String>>,
    pub metadata: Option<HashMap<String, String>>,
}

/// 用户登录请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub remember_me: Option<bool>,
}

/// 用户注册请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub organization: Option<String>,
}

/// 刷新令牌请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

/// 重置密码请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResetPasswordRequest {
    pub email: String,
}

/// 更改密码请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

/// 创建 API 密钥请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// 更新 API 密钥请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateApiKeyRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub permissions: Option<Vec<String>>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// 批量操作请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOperationRequest<T> {
    pub operations: Vec<BatchOperation<T>>,
}

/// 批量操作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOperation<T> {
    pub operation: BatchOperationType,
    pub data: T,
}

/// 批量操作类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BatchOperationType {
    Create,
    Update,
    Delete,
}

/// 导出请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportRequest {
    pub format: ExportFormat,
    pub filters: Option<FilterParams>,
    pub fields: Option<Vec<String>>,
}

/// 导出格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Csv,
    Xml,
    Yaml,
}

/// 导入请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportRequest {
    pub format: ImportFormat,
    pub data: String,
    pub options: Option<ImportOptions>,
}

/// 导入格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImportFormat {
    Json,
    Csv,
    Xml,
    Yaml,
}

/// 导入选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportOptions {
    pub skip_errors: bool,
    pub update_existing: bool,
    pub validate_only: bool,
}

/// 配置更新请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfigRequest {
    pub config: serde_json::Value,
}

/// 系统维护请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceRequest {
    pub action: MaintenanceAction,
    pub message: Option<String>,
    pub scheduled_at: Option<DateTime<Utc>>,
}

/// 维护操作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MaintenanceAction {
    Enable,
    Disable,
    Schedule,
}

/// 备份请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupRequest {
    pub include_data: bool,
    pub include_config: bool,
    pub include_logs: bool,
    pub compression: Option<CompressionType>,
}

/// 压缩类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionType {
    Gzip,
    Bzip2,
    Xz,
}

/// 恢复请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreRequest {
    pub backup_id: String,
    pub restore_data: bool,
    pub restore_config: bool,
    pub restore_logs: bool,
}

/// 通知请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationRequest {
    pub title: String,
    pub message: String,
    pub notification_type: NotificationType,
    pub recipients: Vec<String>,
    pub channels: Vec<NotificationChannel>,
}

/// 通知类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationType {
    Info,
    Warning,
    Error,
    Success,
}

/// 通知渠道
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationChannel {
    Email,
    Sms,
    Push,
    Webhook,
}

/// 审计日志查询请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogQueryRequest {
    #[serde(flatten)]
    pub pagination: PaginationParams,
    #[serde(flatten)]
    pub filter: FilterParams,
    pub action: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub user_id: Option<UserId>,
}

/// 指标查询请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsQueryRequest {
    pub metric_names: Vec<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub interval: Option<String>,
    pub aggregation: Option<AggregationType>,
    pub tags: Option<HashMap<String, String>>,
}

/// 聚合类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregationType {
    Sum,
    Average,
    Min,
    Max,
    Count,
}

/// Webhook 请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookRequest {
    pub url: String,
    pub method: String,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<serde_json::Value>,
    pub timeout: Option<u64>,
    pub retry_count: Option<u32>,
}

/// 缓存操作请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheOperationRequest {
    pub operation: CacheOperationType,
    pub key: Option<String>,
    pub value: Option<serde_json::Value>,
    pub ttl: Option<u64>,
}

/// 缓存操作类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheOperationType {
    Get,
    Set,
    Delete,
    Clear,
    Stats,
} 