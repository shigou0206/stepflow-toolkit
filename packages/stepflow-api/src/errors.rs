use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use stepflow_core::errors::{StepflowError, DatabaseError, MonitoringError};
use stepflow_executor::errors::ExecutorError;
use stepflow_registry::errors::RegistryError;
use stepflow_sandbox::errors::SandboxError;
use thiserror::Error;

/// API 错误类型
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    
    #[error("Forbidden: {0}")]
    Forbidden(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Method not allowed: {0}")]
    MethodNotAllowed(String),
    
    #[error("Conflict: {0}")]
    Conflict(String),
    
    #[error("Unprocessable entity: {0}")]
    UnprocessableEntity(String),
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Internal server error: {0}")]
    InternalServerError(String),
    
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
    
    #[error("Gateway timeout")]
    GatewayTimeout,
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] DatabaseError),
    
    #[error("Registry error: {0}")]
    RegistryError(#[from] RegistryError),
    
    #[error("Executor error: {0}")]
    ExecutorError(#[from] ExecutorError),
    
    #[error("Sandbox error: {0}")]
    SandboxError(#[from] SandboxError),
    
    #[error("Monitoring error: {0}")]
    MonitoringError(#[from] MonitoringError),
    
    #[error("Stepflow error: {0}")]
    StepflowError(#[from] StepflowError),
    
    #[error("JWT error: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),
    
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

impl ApiError {
    /// 获取 HTTP 状态码
    pub fn status_code(&self) -> StatusCode {
        match self {
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            ApiError::Forbidden(_) => StatusCode::FORBIDDEN,
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::MethodNotAllowed(_) => StatusCode::METHOD_NOT_ALLOWED,
            ApiError::Conflict(_) => StatusCode::CONFLICT,
            ApiError::UnprocessableEntity(_) => StatusCode::UNPROCESSABLE_ENTITY,
            ApiError::RateLimitExceeded => StatusCode::TOO_MANY_REQUESTS,
            ApiError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            ApiError::GatewayTimeout => StatusCode::GATEWAY_TIMEOUT,
            ApiError::ValidationError(_) => StatusCode::BAD_REQUEST,
            ApiError::SerializationError(_) => StatusCode::BAD_REQUEST,
            ApiError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::RegistryError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::ExecutorError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::SandboxError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::MonitoringError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::StepflowError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::JwtError(_) => StatusCode::UNAUTHORIZED,
            ApiError::JsonError(_) => StatusCode::BAD_REQUEST,
        }
    }
    
    /// 获取错误代码
    pub fn error_code(&self) -> &'static str {
        match self {
            ApiError::BadRequest(_) => "BAD_REQUEST",
            ApiError::Unauthorized(_) => "UNAUTHORIZED",
            ApiError::Forbidden(_) => "FORBIDDEN",
            ApiError::NotFound(_) => "NOT_FOUND",
            ApiError::MethodNotAllowed(_) => "METHOD_NOT_ALLOWED",
            ApiError::Conflict(_) => "CONFLICT",
            ApiError::UnprocessableEntity(_) => "UNPROCESSABLE_ENTITY",
            ApiError::RateLimitExceeded => "RATE_LIMIT_EXCEEDED",
            ApiError::InternalServerError(_) => "INTERNAL_SERVER_ERROR",
            ApiError::ServiceUnavailable(_) => "SERVICE_UNAVAILABLE",
            ApiError::GatewayTimeout => "GATEWAY_TIMEOUT",
            ApiError::ValidationError(_) => "VALIDATION_ERROR",
            ApiError::SerializationError(_) => "SERIALIZATION_ERROR",
            ApiError::DatabaseError(_) => "DATABASE_ERROR",
            ApiError::RegistryError(_) => "REGISTRY_ERROR",
            ApiError::ExecutorError(_) => "EXECUTOR_ERROR",
            ApiError::SandboxError(_) => "SANDBOX_ERROR",
            ApiError::MonitoringError(_) => "MONITORING_ERROR",
            ApiError::StepflowError(_) => "STEPFLOW_ERROR",
            ApiError::JwtError(_) => "JWT_ERROR",
            ApiError::JsonError(_) => "JSON_ERROR",
        }
    }
    
    /// 是否应该记录错误
    pub fn should_log(&self) -> bool {
        match self {
            ApiError::BadRequest(_) => false,
            ApiError::Unauthorized(_) => false,
            ApiError::Forbidden(_) => false,
            ApiError::NotFound(_) => false,
            ApiError::MethodNotAllowed(_) => false,
            ApiError::Conflict(_) => false,
            ApiError::UnprocessableEntity(_) => false,
            ApiError::RateLimitExceeded => false,
            ApiError::ValidationError(_) => false,
            ApiError::SerializationError(_) => false,
            ApiError::JwtError(_) => false,
            ApiError::JsonError(_) => false,
            _ => true,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status_code = self.status_code();
        let error_code = self.error_code();
        let message = self.to_string();
        
        let body = json!({
            "error": {
                "code": error_code,
                "message": message,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }
        });
        
        (status_code, Json(body)).into_response()
    }
}

/// 中间件错误类型
#[derive(Debug, Error)]
pub enum MiddlewareError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("Authorization failed: {0}")]
    AuthorizationFailed(String),
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Request validation failed: {0}")]
    RequestValidationFailed(String),
    
    #[error("Response validation failed: {0}")]
    ResponseValidationFailed(String),
    
    #[error("Middleware error: {0}")]
    MiddlewareError(String),
}

impl From<MiddlewareError> for ApiError {
    fn from(error: MiddlewareError) -> Self {
        match error {
            MiddlewareError::AuthenticationFailed(msg) => ApiError::Unauthorized(msg),
            MiddlewareError::AuthorizationFailed(msg) => ApiError::Forbidden(msg),
            MiddlewareError::RateLimitExceeded => ApiError::RateLimitExceeded,
            MiddlewareError::RequestValidationFailed(msg) => ApiError::ValidationError(msg),
            MiddlewareError::ResponseValidationFailed(msg) => ApiError::ValidationError(msg),
            MiddlewareError::MiddlewareError(msg) => ApiError::InternalServerError(msg),
        }
    }
}

/// 验证错误类型
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Invalid tool name: {0}")]
    InvalidToolName(String),
    
    #[error("Invalid tool type: {0}")]
    InvalidToolType(String),
    
    #[error("Invalid version format: {0}")]
    InvalidVersionFormat(String),
    
    #[error("Invalid execution ID: {0}")]
    InvalidExecutionId(String),
    
    #[error("Invalid user ID: {0}")]
    InvalidUserId(String),
    
    #[error("Invalid tenant ID: {0}")]
    InvalidTenantId(String),
    
    #[error("Missing required field: {0}")]
    MissingRequiredField(String),
    
    #[error("Invalid field value: {field} = {value}")]
    InvalidFieldValue { field: String, value: String },
    
    #[error("Field too long: {field} (max: {max_length})")]
    FieldTooLong { field: String, max_length: usize },
    
    #[error("Field too short: {field} (min: {min_length})")]
    FieldTooShort { field: String, min_length: usize },
    
    #[error("Invalid JSON: {0}")]
    InvalidJson(String),
    
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    
    #[error("Invalid email: {0}")]
    InvalidEmail(String),
    
    #[error("Invalid date format: {0}")]
    InvalidDateFormat(String),
}

impl From<ValidationError> for ApiError {
    fn from(error: ValidationError) -> Self {
        ApiError::ValidationError(error.to_string())
    }
}

/// 认证错误类型
#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Missing token")]
    MissingToken,
    
    #[error("Invalid token")]
    InvalidToken,
    
    #[error("Expired token")]
    ExpiredToken,
    
    #[error("Invalid credentials")]
    InvalidCredentials,
    
    #[error("Insufficient permissions")]
    InsufficientPermissions,
    
    #[error("Account locked")]
    AccountLocked,
    
    #[error("Account disabled")]
    AccountDisabled,
    
    #[error("Authentication service unavailable")]
    ServiceUnavailable,
}

impl From<AuthError> for ApiError {
    fn from(error: AuthError) -> Self {
        match error {
            AuthError::MissingToken => ApiError::Unauthorized("Missing authentication token".to_string()),
            AuthError::InvalidToken => ApiError::Unauthorized("Invalid authentication token".to_string()),
            AuthError::ExpiredToken => ApiError::Unauthorized("Authentication token has expired".to_string()),
            AuthError::InvalidCredentials => ApiError::Unauthorized("Invalid credentials".to_string()),
            AuthError::InsufficientPermissions => ApiError::Forbidden("Insufficient permissions".to_string()),
            AuthError::AccountLocked => ApiError::Forbidden("Account is locked".to_string()),
            AuthError::AccountDisabled => ApiError::Forbidden("Account is disabled".to_string()),
            AuthError::ServiceUnavailable => ApiError::ServiceUnavailable("Authentication service unavailable".to_string()),
        }
    }
}

/// 速率限制错误类型
#[derive(Debug, Error)]
pub enum RateLimitError {
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Quota exceeded")]
    QuotaExceeded,
    
    #[error("Rate limit configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Rate limit storage error: {0}")]
    StorageError(String),
}

impl From<RateLimitError> for ApiError {
    fn from(error: RateLimitError) -> Self {
        match error {
            RateLimitError::RateLimitExceeded => ApiError::RateLimitExceeded,
            RateLimitError::QuotaExceeded => ApiError::RateLimitExceeded,
            RateLimitError::ConfigurationError(msg) => ApiError::InternalServerError(msg),
            RateLimitError::StorageError(msg) => ApiError::InternalServerError(msg),
        }
    }
}

/// API 结果类型
pub type ApiResult<T> = Result<T, ApiError>;
pub type MiddlewareResult<T> = Result<T, MiddlewareError>;
pub type ValidationResult<T> = Result<T, ValidationError>;
pub type AuthResult<T> = Result<T, AuthError>;
pub type RateLimitResult<T> = Result<T, RateLimitError>; 