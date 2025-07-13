use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use stepflow_core::types::UserId;

/// API 版本
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ApiVersion {
    V1,
    V2,
    Latest,
}

impl Default for ApiVersion {
    fn default() -> Self {
        ApiVersion::V1
    }
}

impl std::fmt::Display for ApiVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiVersion::V1 => write!(f, "v1"),
            ApiVersion::V2 => write!(f, "v2"),
            ApiVersion::Latest => write!(f, "latest"),
        }
    }
}

/// HTTP 方法
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpMethod::GET => write!(f, "GET"),
            HttpMethod::POST => write!(f, "POST"),
            HttpMethod::PUT => write!(f, "PUT"),
            HttpMethod::DELETE => write!(f, "DELETE"),
            HttpMethod::PATCH => write!(f, "PATCH"),
            HttpMethod::HEAD => write!(f, "HEAD"),
            HttpMethod::OPTIONS => write!(f, "OPTIONS"),
        }
    }
}

/// HTTP 请求
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub query_params: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
    pub user_id: Option<UserId>,
    pub tenant_id: Option<String>,
    pub request_id: String,
    pub remote_addr: Option<String>,
    pub user_agent: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl HttpRequest {
    pub fn new(method: HttpMethod, path: String) -> Self {
        Self {
            method,
            path,
            headers: HashMap::new(),
            query_params: HashMap::new(),
            body: None,
            user_id: None,
            tenant_id: None,
            request_id: uuid::Uuid::new_v4().to_string(),
            remote_addr: None,
            user_agent: None,
            timestamp: Utc::now(),
        }
    }
    
    pub fn with_header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }
    
    pub fn with_query_param(mut self, key: String, value: String) -> Self {
        self.query_params.insert(key, value);
        self
    }
    
    pub fn with_body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
        self
    }
    
    pub fn with_user_id(mut self, user_id: UserId) -> Self {
        self.user_id = Some(user_id);
        self
    }
    
    pub fn get_header(&self, key: &str) -> Option<&String> {
        self.headers.get(key)
    }
    
    pub fn get_query_param(&self, key: &str) -> Option<&String> {
        self.query_params.get(key)
    }
    
    pub fn content_type(&self) -> Option<&String> {
        self.get_header("content-type")
    }
    
    pub fn authorization(&self) -> Option<&String> {
        self.get_header("authorization")
    }
}

/// HTTP 响应
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub content_type: String,
    pub request_id: String,
    pub timestamp: DateTime<Utc>,
}

impl HttpResponse {
    pub fn new(status_code: u16, body: Vec<u8>, content_type: String) -> Self {
        Self {
            status_code,
            headers: HashMap::new(),
            body,
            content_type,
            request_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
        }
    }
    
    pub fn ok(body: Vec<u8>, content_type: String) -> Self {
        Self::new(200, body, content_type)
    }
    
    pub fn created(body: Vec<u8>, content_type: String) -> Self {
        Self::new(201, body, content_type)
    }
    
    pub fn no_content() -> Self {
        Self::new(204, Vec::new(), "application/json".to_string())
    }
    
    pub fn bad_request(message: String) -> Self {
        let body = serde_json::json!({
            "error": {
                "code": "BAD_REQUEST",
                "message": message,
                "timestamp": Utc::now().to_rfc3339(),
            }
        });
        Self::new(400, body.to_string().into_bytes(), "application/json".to_string())
    }
    
    pub fn unauthorized(message: String) -> Self {
        let body = serde_json::json!({
            "error": {
                "code": "UNAUTHORIZED",
                "message": message,
                "timestamp": Utc::now().to_rfc3339(),
            }
        });
        Self::new(401, body.to_string().into_bytes(), "application/json".to_string())
    }
    
    pub fn not_found(message: String) -> Self {
        let body = serde_json::json!({
            "error": {
                "code": "NOT_FOUND",
                "message": message,
                "timestamp": Utc::now().to_rfc3339(),
            }
        });
        Self::new(404, body.to_string().into_bytes(), "application/json".to_string())
    }
    
    pub fn internal_server_error(message: String) -> Self {
        let body = serde_json::json!({
            "error": {
                "code": "INTERNAL_SERVER_ERROR",
                "message": message,
                "timestamp": Utc::now().to_rfc3339(),
            }
        });
        Self::new(500, body.to_string().into_bytes(), "application/json".to_string())
    }
    
    pub fn with_header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }
    
    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = request_id;
        self
    }
}

/// 服务器配置
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
    pub max_connections: usize,
    pub request_timeout: std::time::Duration,
    pub enable_cors: bool,
    pub enable_compression: bool,
    pub enable_logging: bool,
    pub enable_metrics: bool,
    pub api_version: ApiVersion,
    pub cors_config: CorsConfig,
    pub auth_config: AuthConfig,
    pub rate_limit_config: RateLimitConfig,
    pub logging_config: LoggingConfig,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            workers: 4,
            max_connections: 1000,
            request_timeout: std::time::Duration::from_secs(30),
            enable_cors: true,
            enable_compression: true,
            enable_logging: true,
            enable_metrics: true,
            api_version: ApiVersion::V1,
            cors_config: CorsConfig::default(),
            auth_config: AuthConfig::default(),
            rate_limit_config: RateLimitConfig::default(),
            logging_config: LoggingConfig::default(),
        }
    }
}

/// CORS 配置
#[derive(Debug, Clone)]
pub struct CorsConfig {
    pub allow_origins: Vec<String>,
    pub allow_methods: Vec<HttpMethod>,
    pub allow_headers: Vec<String>,
    pub expose_headers: Vec<String>,
    pub allow_credentials: bool,
    pub max_age: Option<std::time::Duration>,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allow_origins: vec!["*".to_string()],
            allow_methods: vec![
                HttpMethod::GET,
                HttpMethod::POST,
                HttpMethod::PUT,
                HttpMethod::DELETE,
                HttpMethod::PATCH,
                HttpMethod::HEAD,
                HttpMethod::OPTIONS,
            ],
            allow_headers: vec![
                "Content-Type".to_string(),
                "Authorization".to_string(),
                "Accept".to_string(),
                "Origin".to_string(),
                "X-Requested-With".to_string(),
            ],
            expose_headers: vec![
                "X-Request-ID".to_string(),
                "X-Rate-Limit-Remaining".to_string(),
                "X-Rate-Limit-Reset".to_string(),
            ],
            allow_credentials: true,
            max_age: Some(std::time::Duration::from_secs(3600)),
        }
    }
}

/// 认证配置
#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub enable_auth: bool,
    pub jwt_secret: String,
    pub jwt_expiration: std::time::Duration,
    pub jwt_refresh_expiration: std::time::Duration,
    pub enable_api_keys: bool,
    pub enable_oauth: bool,
    pub oauth_providers: Vec<OAuthProvider>,
    pub password_policy: PasswordPolicy,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enable_auth: true,
            jwt_secret: "default-secret-key".to_string(),
            jwt_expiration: std::time::Duration::from_secs(3600),
            jwt_refresh_expiration: std::time::Duration::from_secs(86400 * 7),
            enable_api_keys: true,
            enable_oauth: false,
            oauth_providers: Vec::new(),
            password_policy: PasswordPolicy::default(),
        }
    }
}

/// OAuth 提供商
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthProvider {
    pub name: String,
    pub client_id: String,
    pub client_secret: String,
    pub auth_url: String,
    pub token_url: String,
    pub user_info_url: String,
    pub scopes: Vec<String>,
}

/// 密码策略
#[derive(Debug, Clone)]
pub struct PasswordPolicy {
    pub min_length: usize,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_numbers: bool,
    pub require_symbols: bool,
    pub max_age: Option<std::time::Duration>,
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: 8,
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: true,
            require_symbols: false,
            max_age: Some(std::time::Duration::from_secs(86400 * 90)),
        }
    }
}

/// 速率限制配置
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub enable_rate_limit: bool,
    pub requests_per_minute: usize,
    pub requests_per_hour: usize,
    pub requests_per_day: usize,
    pub burst_size: usize,
    pub storage_backend: RateLimitBackend,
    pub custom_limits: HashMap<String, CustomRateLimit>,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enable_rate_limit: true,
            requests_per_minute: 100,
            requests_per_hour: 1000,
            requests_per_day: 10000,
            burst_size: 10,
            storage_backend: RateLimitBackend::Memory,
            custom_limits: HashMap::new(),
        }
    }
}

/// 速率限制后端
#[derive(Debug, Clone)]
pub enum RateLimitBackend {
    Memory,
    Redis,
    Database,
}

/// 自定义速率限制
#[derive(Debug, Clone)]
pub struct CustomRateLimit {
    pub endpoint: String,
    pub method: HttpMethod,
    pub requests_per_minute: usize,
    pub requests_per_hour: usize,
    pub burst_size: usize,
}

/// 日志配置
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub enable_access_log: bool,
    pub enable_error_log: bool,
    pub enable_performance_log: bool,
    pub log_level: String,
    pub log_format: LogFormat,
    pub log_destination: LogDestination,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            enable_access_log: true,
            enable_error_log: true,
            enable_performance_log: true,
            log_level: "info".to_string(),
            log_format: LogFormat::Json,
            log_destination: LogDestination::Stdout,
        }
    }
}

/// 日志格式
#[derive(Debug, Clone)]
pub enum LogFormat {
    Json,
    Text,
    Combined,
}

/// 日志目标
#[derive(Debug, Clone)]
pub enum LogDestination {
    Stdout,
    File(String),
    Database,
    Remote(String),
}

/// 服务器状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStatus {
    pub status: String,
    pub version: String,
    pub uptime: std::time::Duration,
    pub active_connections: usize,
    pub total_requests: u64,
    pub requests_per_second: f64,
    pub memory_usage: u64,
    pub cpu_usage: f64,
    pub timestamp: DateTime<Utc>,
}

/// 健康检查状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: HealthStatusType,
    pub checks: HashMap<String, HealthCheck>,
    pub timestamp: DateTime<Utc>,
}

/// 健康检查类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatusType {
    Healthy,
    Degraded,
    Unhealthy,
}

/// 健康检查项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub status: HealthStatusType,
    pub message: String,
    pub duration: std::time::Duration,
    pub timestamp: DateTime<Utc>,
}

/// API 指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time: std::time::Duration,
    pub requests_per_second: f64,
    pub error_rate: f64,
    pub active_connections: usize,
    pub memory_usage: u64,
    pub cpu_usage: f64,
    pub timestamp: DateTime<Utc>,
}

/// 分页参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationParams {
    pub page: Option<usize>,
    pub page_size: Option<usize>,
    pub sort_by: Option<String>,
    pub sort_order: Option<SortOrder>,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: Some(1),
            page_size: Some(20),
            sort_by: None,
            sort_order: Some(SortOrder::Asc),
        }
    }
}

/// 排序顺序
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOrder {
    Asc,
    Desc,
}

/// 分页响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationInfo,
}

/// 分页信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationInfo {
    pub page: usize,
    pub page_size: usize,
    pub total_items: usize,
    pub total_pages: usize,
    pub has_next: bool,
    pub has_previous: bool,
}

/// 过滤参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterParams {
    pub filters: HashMap<String, String>,
    pub search: Option<String>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
}

/// 用户上下文
#[derive(Debug, Clone)]
pub struct UserContext {
    pub user_id: UserId,
    pub tenant_id: Option<String>,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub session_id: String,
    pub expires_at: DateTime<Utc>,
}

/// JWT 声明
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,
    pub iat: i64,
    pub exp: i64,
    pub aud: String,
    pub iss: String,
    pub user_id: String,
    pub tenant_id: Option<String>,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
}

/// API 密钥
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: String,
    pub name: String,
    pub key_hash: String,
    pub user_id: UserId,
    pub tenant_id: Option<String>,
    pub permissions: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 会话
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub user_id: UserId,
    pub tenant_id: Option<String>,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 请求 ID
pub type RequestId = String;

/// 租户 ID
pub type TenantId = String; 