use crate::errors::{ApiError, ApiResult};
use crate::types::{
    ApiMetrics, HealthStatus, HttpRequest, HttpResponse, ServerConfig, ServerStatus, UserContext,
};
use async_trait::async_trait;
use std::sync::Arc;
use stepflow_database::SqliteDatabase;
use stepflow_executor::Executor;
use stepflow_registry::Registry;
use stepflow_sandbox::Sandbox;

/// API 服务器特征
#[async_trait]
pub trait ApiServer: Send + Sync {
    /// 启动 API 服务器
    async fn start(&self, config: ServerConfig) -> ApiResult<()>;
    
    /// 停止 API 服务器
    async fn stop(&self) -> ApiResult<()>;
    
    /// 获取服务器状态
    async fn get_status(&self) -> ApiResult<ServerStatus>;
    
    /// 健康检查
    async fn health_check(&self) -> ApiResult<HealthStatus>;
    
    /// 获取 API 指标
    async fn get_metrics(&self) -> ApiResult<ApiMetrics>;
    
    /// 重新加载配置
    async fn reload_config(&self, config: ServerConfig) -> ApiResult<()>;
    
    /// 优雅关闭
    async fn graceful_shutdown(&self) -> ApiResult<()>;
}

/// 路由处理器特征
#[async_trait]
pub trait RouteHandler: Send + Sync {
    /// 处理 GET 请求
    async fn handle_get(&self, request: HttpRequest) -> ApiResult<HttpResponse>;
    
    /// 处理 POST 请求
    async fn handle_post(&self, request: HttpRequest) -> ApiResult<HttpResponse>;
    
    /// 处理 PUT 请求
    async fn handle_put(&self, request: HttpRequest) -> ApiResult<HttpResponse>;
    
    /// 处理 DELETE 请求
    async fn handle_delete(&self, request: HttpRequest) -> ApiResult<HttpResponse>;
    
    /// 处理 PATCH 请求
    async fn handle_patch(&self, request: HttpRequest) -> ApiResult<HttpResponse>;
    
    /// 处理 HEAD 请求
    async fn handle_head(&self, request: HttpRequest) -> ApiResult<HttpResponse>;
    
    /// 处理 OPTIONS 请求
    async fn handle_options(&self, request: HttpRequest) -> ApiResult<HttpResponse>;
    
    /// 获取支持的方法
    fn supported_methods(&self) -> Vec<crate::types::HttpMethod>;
    
    /// 获取路由路径
    fn route_path(&self) -> &str;
}

/// 中间件特征
#[async_trait]
pub trait Middleware: Send + Sync {
    /// 处理请求
    async fn process_request(&self, request: &mut HttpRequest) -> ApiResult<()>;
    
    /// 处理响应
    async fn process_response(&self, response: &mut HttpResponse) -> ApiResult<()>;
    
    /// 获取中间件名称
    fn name(&self) -> &str;
    
    /// 获取中间件优先级
    fn priority(&self) -> u32;
    
    /// 是否启用
    fn is_enabled(&self) -> bool {
        true
    }
}

/// 认证服务特征
#[async_trait]
pub trait AuthService: Send + Sync {
    /// 验证 JWT 令牌
    async fn validate_jwt_token(&self, token: &str) -> ApiResult<UserContext>;
    
    /// 验证 API 密钥
    async fn validate_api_key(&self, key: &str) -> ApiResult<UserContext>;
    
    /// 生成 JWT 令牌
    async fn generate_jwt_token(&self, user_context: &UserContext) -> ApiResult<String>;
    
    /// 刷新 JWT 令牌
    async fn refresh_jwt_token(&self, refresh_token: &str) -> ApiResult<String>;
    
    /// 撤销令牌
    async fn revoke_token(&self, token: &str) -> ApiResult<()>;
    
    /// 验证用户权限
    async fn check_permission(&self, user_context: &UserContext, permission: &str) -> ApiResult<bool>;
    
    /// 获取用户角色
    async fn get_user_roles(&self, user_id: &str) -> ApiResult<Vec<String>>;
    
    /// 获取用户权限
    async fn get_user_permissions(&self, user_id: &str) -> ApiResult<Vec<String>>;
}

/// 速率限制服务特征
#[async_trait]
pub trait RateLimitService: Send + Sync {
    /// 检查速率限制
    async fn check_rate_limit(&self, key: &str, limit: usize, window: std::time::Duration) -> ApiResult<bool>;
    
    /// 获取剩余请求数
    async fn get_remaining_requests(&self, key: &str, limit: usize, window: std::time::Duration) -> ApiResult<usize>;
    
    /// 获取重置时间
    async fn get_reset_time(&self, key: &str, window: std::time::Duration) -> ApiResult<std::time::SystemTime>;
    
    /// 重置速率限制
    async fn reset_rate_limit(&self, key: &str) -> ApiResult<()>;
    
    /// 获取当前请求数
    async fn get_current_requests(&self, key: &str, window: std::time::Duration) -> ApiResult<usize>;
}

/// 监控服务特征
#[async_trait]
pub trait MonitoringService: Send + Sync {
    /// 记录请求
    async fn record_request(&self, request: &HttpRequest, response: &HttpResponse, duration: std::time::Duration) -> ApiResult<()>;
    
    /// 记录错误
    async fn record_error(&self, request: &HttpRequest, error: &ApiError) -> ApiResult<()>;
    
    /// 获取指标
    async fn get_metrics(&self) -> ApiResult<ApiMetrics>;
    
    /// 获取健康状态
    async fn get_health_status(&self) -> ApiResult<HealthStatus>;
    
    /// 记录自定义指标
    async fn record_custom_metric(&self, name: &str, value: f64, tags: Option<std::collections::HashMap<String, String>>) -> ApiResult<()>;
    
    /// 增加计数器
    async fn increment_counter(&self, name: &str, tags: Option<std::collections::HashMap<String, String>>) -> ApiResult<()>;
    
    /// 记录直方图
    async fn record_histogram(&self, name: &str, value: f64, tags: Option<std::collections::HashMap<String, String>>) -> ApiResult<()>;
}

/// 验证服务特征
#[async_trait]
pub trait ValidationService: Send + Sync {
    /// 验证请求
    async fn validate_request(&self, request: &HttpRequest) -> ApiResult<()>;
    
    /// 验证响应
    async fn validate_response(&self, response: &HttpResponse) -> ApiResult<()>;
    
    /// 验证 JSON 数据
    async fn validate_json(&self, data: &serde_json::Value, schema: &str) -> ApiResult<()>;
    
    /// 验证字段
    async fn validate_field(&self, field_name: &str, value: &str, rules: &[&str]) -> ApiResult<()>;
    
    /// 验证工具名称
    async fn validate_tool_name(&self, name: &str) -> ApiResult<()>;
    
    /// 验证工具类型
    async fn validate_tool_type(&self, tool_type: &str) -> ApiResult<()>;
    
    /// 验证版本格式
    async fn validate_version(&self, version: &str) -> ApiResult<()>;
    
    /// 验证用户 ID
    async fn validate_user_id(&self, user_id: &str) -> ApiResult<()>;
    
    /// 验证执行 ID
    async fn validate_execution_id(&self, execution_id: &str) -> ApiResult<()>;
}

/// 缓存服务特征
#[async_trait]
pub trait CacheService: Send + Sync {
    /// 获取缓存值
    async fn get(&self, key: &str) -> ApiResult<Option<Vec<u8>>>;
    
    /// 设置缓存值
    async fn set(&self, key: &str, value: Vec<u8>, ttl: Option<std::time::Duration>) -> ApiResult<()>;
    
    /// 删除缓存值
    async fn delete(&self, key: &str) -> ApiResult<()>;
    
    /// 检查键是否存在
    async fn exists(&self, key: &str) -> ApiResult<bool>;
    
    /// 设置过期时间
    async fn expire(&self, key: &str, ttl: std::time::Duration) -> ApiResult<()>;
    
    /// 获取 TTL
    async fn ttl(&self, key: &str) -> ApiResult<Option<std::time::Duration>>;
    
    /// 清空缓存
    async fn clear(&self) -> ApiResult<()>;
    
    /// 获取缓存统计
    async fn stats(&self) -> ApiResult<std::collections::HashMap<String, String>>;
}

/// 请求上下文
#[derive(Debug, Clone)]
pub struct RequestContext {
    pub request_id: String,
    pub user_context: Option<UserContext>,
    pub start_time: std::time::Instant,
    pub metadata: std::collections::HashMap<String, String>,
}

impl RequestContext {
    pub fn new(request_id: String) -> Self {
        Self {
            request_id,
            user_context: None,
            start_time: std::time::Instant::now(),
            metadata: std::collections::HashMap::new(),
        }
    }
    
    pub fn with_user_context(mut self, user_context: UserContext) -> Self {
        self.user_context = Some(user_context);
        self
    }
    
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
    
    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }
}

/// 应用状态
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<SqliteDatabase>,
    pub registry: Arc<dyn Registry>,
    pub executor: Arc<dyn Executor>,
    pub sandbox: Arc<dyn Sandbox>,
    pub auth_service: Arc<dyn AuthService>,
    pub rate_limit_service: Arc<dyn RateLimitService>,
    pub monitoring_service: Arc<dyn MonitoringService>,
    pub validation_service: Arc<dyn ValidationService>,
    pub cache_service: Arc<dyn CacheService>,
    pub config: ServerConfig,
}

impl AppState {
    pub fn new(
        db: Arc<SqliteDatabase>,
        registry: Arc<dyn Registry>,
        executor: Arc<dyn Executor>,
        sandbox: Arc<dyn Sandbox>,
        auth_service: Arc<dyn AuthService>,
        rate_limit_service: Arc<dyn RateLimitService>,
        monitoring_service: Arc<dyn MonitoringService>,
        validation_service: Arc<dyn ValidationService>,
        cache_service: Arc<dyn CacheService>,
        config: ServerConfig,
    ) -> Self {
        Self {
            db,
            registry,
            executor,
            sandbox,
            auth_service,
            rate_limit_service,
            monitoring_service,
            validation_service,
            cache_service,
            config,
        }
    }
}

/// 自定义处理器特征
#[async_trait]
pub trait CustomHandler: Send + Sync {
    /// 处理请求
    async fn handle_request(&self, request: HttpRequest) -> ApiResult<HttpResponse>;
    
    /// 获取路由
    fn get_route(&self) -> &str;
    
    /// 获取方法
    fn get_methods(&self) -> Vec<crate::types::HttpMethod>;
    
    /// 获取处理器名称
    fn name(&self) -> &str;
    
    /// 获取处理器描述
    fn description(&self) -> &str;
    
    /// 是否需要认证
    fn requires_auth(&self) -> bool {
        true
    }
    
    /// 所需权限
    fn required_permissions(&self) -> Vec<String> {
        Vec::new()
    }
}

/// 自定义中间件特征
#[async_trait]
pub trait CustomMiddleware: Send + Sync {
    /// 处理请求
    async fn process_request(&self, request: &mut HttpRequest) -> ApiResult<()>;
    
    /// 处理响应
    async fn process_response(&self, response: &mut HttpResponse) -> ApiResult<()>;
    
    /// 获取中间件名称
    fn name(&self) -> &str;
    
    /// 获取中间件优先级
    fn priority(&self) -> u32;
    
    /// 是否启用
    fn is_enabled(&self) -> bool {
        true
    }
    
    /// 获取中间件描述
    fn description(&self) -> &str;
    
    /// 获取中间件版本
    fn version(&self) -> &str;
} 