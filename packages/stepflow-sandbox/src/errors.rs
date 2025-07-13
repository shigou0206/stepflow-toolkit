use thiserror::Error;
use stepflow_core::DatabaseError;

/// Sandbox 相关错误
#[derive(Debug, Error)]
pub enum SandboxError {
    #[error("Sandbox not found: {0}")]
    SandboxNotFound(String),
    
    #[error("Sandbox creation failed: {0}")]
    SandboxCreationFailed(String),
    
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    
    #[error("Security violation: {0}")]
    SecurityViolation(String),
    
    #[error("Resource limit exceeded")]
    ResourceLimitExceeded,
    
    #[error("Container error: {0}")]
    ContainerError(#[from] ContainerError),
    
    #[error("Isolation error: {0}")]
    IsolationError(#[from] IsolationError),
    
    #[error("Permission denied")]
    PermissionDenied,
    
    #[error("Internal error: {0}")]
    InternalError(String),
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] DatabaseError),
    
    #[error("Monitoring error: {0}")]
    MonitoringError(#[from] MonitoringError),
}

/// Container 相关错误
#[derive(Debug, Error)]
pub enum ContainerError {
    #[error("Container not found: {0}")]
    ContainerNotFound(String),
    
    #[error("Container creation failed: {0}")]
    ContainerCreationFailed(String),
    
    #[error("Container start failed: {0}")]
    ContainerStartFailed(String),
    
    #[error("Container stop failed: {0}")]
    ContainerStopFailed(String),
    
    #[error("Container delete failed: {0}")]
    ContainerDeleteFailed(String),
    
    #[error("Docker error: {0}")]
    DockerError(String),
    
    #[error("Image pull failed: {0}")]
    ImagePullFailed(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Volume mount error: {0}")]
    VolumeMountError(String),
}

/// 隔离相关错误
#[derive(Debug, Error)]
pub enum IsolationError {
    #[error("Namespace creation failed: {0}")]
    NamespaceCreationFailed(String),
    
    #[error("Security policy application failed: {0}")]
    SecurityPolicyFailed(String),
    
    #[error("Resource limit setting failed: {0}")]
    ResourceLimitFailed(String),
    
    #[error("Seccomp profile error: {0}")]
    SeccompError(String),
    
    #[error("Capability setting error: {0}")]
    CapabilityError(String),
    
    #[error("Isolation not supported: {0}")]
    IsolationNotSupported(String),
}

/// 安全相关错误
#[derive(Debug, Error)]
pub enum SecurityError {
    #[error("Security policy validation failed: {0}")]
    PolicyValidationFailed(String),
    
    #[error("Security audit failed: {0}")]
    AuditFailed(String),
    
    #[error("Vulnerability detected: {0}")]
    VulnerabilityDetected(String),
    
    #[error("Access denied: {0}")]
    AccessDenied(String),
    
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("Authorization failed: {0}")]
    AuthorizationFailed(String),
}

/// 监控相关错误
#[derive(Debug, Error)]
pub enum MonitoringError {
    #[error("Metric collection failed: {0}")]
    MetricCollectionFailed(String),
    
    #[error("Metric storage failed: {0}")]
    MetricStorageFailed(String),
    
    #[error("Monitoring initialization failed: {0}")]
    InitializationFailed(String),
    
    #[error("Monitoring query failed: {0}")]
    QueryFailed(String),
}

/// 扫描相关错误
#[derive(Debug, Error)]
pub enum ScanError {
    #[error("Vulnerability scan failed: {0}")]
    ScanFailed(String),
    
    #[error("Scanner initialization failed: {0}")]
    ScannerInitFailed(String),
    
    #[error("Scan result parsing failed: {0}")]
    ResultParsingFailed(String),
}

/// 结果类型别名
pub type SandboxResult<T> = Result<T, SandboxError>;
pub type ContainerResult<T> = Result<T, ContainerError>;
pub type IsolationResult<T> = Result<T, IsolationError>;
pub type SecurityResult<T> = Result<T, SecurityError>;
pub type MonitoringResult<T> = Result<T, MonitoringError>;
pub type ScanResult<T> = Result<T, ScanError>; 