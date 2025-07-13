use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use stepflow_core::{ToolId, ExecutionId, UserId};
use std::collections::HashMap;
use crate::types::PaginationInfo;

/// 标准 API 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiErrorResponse>,
    pub meta: Option<ResponseMeta>,
}

/// API 错误响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub timestamp: DateTime<Utc>,
}

/// 响应元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMeta {
    pub request_id: String,
    pub timestamp: DateTime<Utc>,
    pub version: String,
    pub execution_time: u64,
}

/// 工具响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResponse {
    pub id: ToolId,
    pub name: String,
    pub description: String,
    pub tool_type: String,
    pub version: String,
    pub author: String,
    pub license: String,
    pub repository: Option<String>,
    pub documentation: Option<String>,
    pub tags: Vec<String>,
    pub status: String,
    pub configuration: serde_json::Value,
    pub requirements: HashMap<String, String>,
    pub metadata: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 注册工具响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterToolResponse {
    pub tool: ToolResponse,
    pub message: String,
}

/// 更新工具响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateToolResponse {
    pub tool: ToolResponse,
    pub message: String,
}

/// 删除工具响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteToolResponse {
    pub message: String,
    pub deleted_at: DateTime<Utc>,
}

/// 列出工具响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListToolsResponse {
    pub tools: Vec<ToolResponse>,
    pub pagination: PaginationInfo,
}

/// 获取工具响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetToolResponse {
    pub tool: ToolResponse,
}

/// 执行响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResponse {
    pub id: ExecutionId,
    pub tool_id: ToolId,
    pub tool_version: String,
    pub user_id: UserId,
    pub status: String,
    pub inputs: HashMap<String, serde_json::Value>,
    pub outputs: Option<HashMap<String, serde_json::Value>>,
    pub error: Option<String>,
    pub configuration: Option<serde_json::Value>,
    pub sandbox_config: Option<SandboxConfigResponse>,
    pub metrics: Option<ExecutionMetrics>,
    pub logs: Option<Vec<LogEntry>>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 沙箱配置响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfigResponse {
    pub isolation_type: String,
    pub resource_limits: Option<ResourceLimitsResponse>,
    pub network_config: Option<NetworkConfigResponse>,
    pub storage_config: Option<StorageConfigResponse>,
    pub security_policy: Option<SecurityPolicyResponse>,
}

/// 资源限制响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimitsResponse {
    pub memory_limit: Option<u64>,
    pub cpu_limit: Option<f64>,
    pub disk_limit: Option<u64>,
    pub network_limit: Option<u64>,
    pub process_limit: Option<u32>,
    pub file_descriptor_limit: Option<u32>,
    pub execution_timeout: Option<u64>,
}

/// 网络配置响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfigResponse {
    pub enable_network: bool,
    pub allowed_hosts: Option<Vec<String>>,
    pub blocked_hosts: Option<Vec<String>>,
    pub allowed_ports: Option<Vec<u16>>,
    pub blocked_ports: Option<Vec<u16>>,
    pub dns_servers: Option<Vec<String>>,
}

/// 存储配置响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfigResponse {
    pub read_only: bool,
    pub temp_directory: Option<String>,
    pub allowed_paths: Option<Vec<String>>,
    pub blocked_paths: Option<Vec<String>>,
    pub max_file_size: Option<u64>,
    pub max_total_size: Option<u64>,
}

/// 安全策略响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicyResponse {
    pub allow_system_calls: bool,
    pub allowed_syscalls: Option<Vec<String>>,
    pub blocked_syscalls: Option<Vec<String>>,
    pub allow_network_access: bool,
    pub allow_file_access: bool,
    pub allow_process_spawn: bool,
    pub capabilities: Option<Vec<String>>,
}

/// 执行指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    pub execution_time: u64,
    pub memory_usage: u64,
    pub cpu_usage: f64,
    pub disk_usage: u64,
    pub network_usage: u64,
    pub exit_code: Option<i32>,
}

/// 日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub level: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: Option<HashMap<String, String>>,
}

/// 执行工具响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteToolResponse {
    pub execution: ExecutionResponse,
    pub message: String,
}

/// 获取执行响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetExecutionResponse {
    pub execution: ExecutionResponse,
}

/// 列出执行响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListExecutionsResponse {
    pub executions: Vec<ExecutionResponse>,
    pub pagination: PaginationInfo,
}

/// 取消执行响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelExecutionResponse {
    pub message: String,
    pub cancelled_at: DateTime<Utc>,
}

/// 搜索工具响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchToolsResponse {
    pub tools: Vec<ToolResponse>,
    pub total_count: usize,
    pub query: String,
}

/// 工具版本响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolVersionResponse {
    pub version: String,
    pub description: Option<String>,
    pub changelog: Option<String>,
    pub configuration: Option<serde_json::Value>,
    pub requirements: HashMap<String, String>,
    pub metadata: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 获取工具版本响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetToolVersionsResponse {
    pub versions: Vec<ToolVersionResponse>,
    pub total_count: usize,
}

/// 创建工具版本响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateToolVersionResponse {
    pub version: ToolVersionResponse,
    pub message: String,
}

/// 用户响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: UserId,
    pub username: String,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub organization: Option<String>,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub is_active: bool,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 登录响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub user: UserResponse,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: DateTime<Utc>,
    pub token_type: String,
}

/// 注册用户响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterUserResponse {
    pub user: UserResponse,
    pub message: String,
}

/// 刷新令牌响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: DateTime<Utc>,
    pub token_type: String,
}

/// API 密钥响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub key_preview: String,
    pub permissions: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 创建 API 密钥响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApiKeyResponse {
    pub api_key: ApiKeyResponse,
    pub secret_key: String,
    pub message: String,
}

/// 服务器状态响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStatusResponse {
    pub status: String,
    pub version: String,
    pub uptime: u64,
    pub active_connections: usize,
    pub total_requests: u64,
    pub requests_per_second: f64,
    pub memory_usage: u64,
    pub cpu_usage: f64,
    pub timestamp: DateTime<Utc>,
}

/// 健康检查响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

/// 详细健康检查响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedHealthResponse {
    pub status: String,
    pub services: std::collections::HashMap<String, ServiceHealth>,
    pub timestamp: DateTime<Utc>,
}

/// 服务健康状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    pub status: String,
    pub message: String,
    pub duration: u64,
    pub timestamp: DateTime<Utc>,
}

/// 应用程序状态
pub struct AppState {
    pub registry: std::sync::Arc<dyn stepflow_registry::Registry>,
    pub executor: std::sync::Arc<dyn stepflow_executor::Executor>,
    pub sandbox: std::sync::Arc<dyn stepflow_sandbox::Sandbox>,
    pub database: std::sync::Arc<stepflow_database::SqliteDatabase>,
}

/// 指标响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsResponse {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time: u64,
    pub requests_per_second: f64,
    pub error_rate: f64,
    pub active_connections: usize,
    pub memory_usage: u64,
    pub cpu_usage: f64,
    pub timestamp: DateTime<Utc>,
}

/// 批量操作响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOperationResponse<T> {
    pub results: Vec<BatchOperationResult<T>>,
    pub total_count: usize,
    pub success_count: usize,
    pub error_count: usize,
}

/// 批量操作结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOperationResult<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

/// 导出响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResponse {
    pub format: String,
    pub data: String,
    pub filename: String,
    pub size: usize,
    pub timestamp: DateTime<Utc>,
}

/// 导入响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResponse {
    pub total_records: usize,
    pub imported_records: usize,
    pub failed_records: usize,
    pub errors: Vec<ImportError>,
    pub timestamp: DateTime<Utc>,
}

/// 导入错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportError {
    pub line: usize,
    pub error: String,
    pub data: Option<serde_json::Value>,
}

/// 配置响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigResponse {
    pub config: serde_json::Value,
    pub version: String,
    pub timestamp: DateTime<Utc>,
}

/// 审计日志响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogResponse {
    pub id: String,
    pub action: String,
    pub resource_type: String,
    pub resource_id: String,
    pub user_id: UserId,
    pub changes: Option<serde_json::Value>,
    pub metadata: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
}

/// 审计日志列表响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogListResponse {
    pub logs: Vec<AuditLogResponse>,
    pub pagination: PaginationInfo,
}

/// 指标数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDataPoint {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
    pub tags: Option<HashMap<String, String>>,
}

/// 指标查询响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsQueryResponse {
    pub metrics: HashMap<String, Vec<MetricDataPoint>>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub interval: String,
}

/// 通知响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationResponse {
    pub id: String,
    pub status: String,
    pub sent_at: DateTime<Utc>,
    pub delivery_status: HashMap<String, String>,
}

/// 备份响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupResponse {
    pub id: String,
    pub filename: String,
    pub size: u64,
    pub checksum: String,
    pub created_at: DateTime<Utc>,
}

/// 恢复响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreResponse {
    pub backup_id: String,
    pub status: String,
    pub restored_at: DateTime<Utc>,
    pub details: HashMap<String, String>,
}

/// Webhook 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookResponse {
    pub id: String,
    pub status: String,
    pub response_code: Option<u16>,
    pub response_body: Option<String>,
    pub executed_at: DateTime<Utc>,
    pub duration: u64,
}

/// 缓存操作响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheOperationResponse {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub ttl: Option<u64>,
    pub stats: Option<HashMap<String, String>>,
}

/// 统计响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticsResponse {
    pub total_tools: usize,
    pub total_executions: usize,
    pub total_users: usize,
    pub active_users: usize,
    pub success_rate: f64,
    pub average_execution_time: u64,
    pub popular_tools: Vec<PopularTool>,
    pub recent_activity: Vec<ActivityItem>,
}

/// 热门工具
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopularTool {
    pub tool_id: ToolId,
    pub name: String,
    pub execution_count: usize,
    pub success_rate: f64,
}

/// 活动项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityItem {
    pub action: String,
    pub resource_type: String,
    pub resource_id: String,
    pub user_id: UserId,
    pub timestamp: DateTime<Utc>,
}

/// 简单成功响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessResponse {
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

/// 简单错误响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub timestamp: DateTime<Utc>,
} 