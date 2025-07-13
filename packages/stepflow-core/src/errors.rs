//! Error types for Stepflow Tool System

use thiserror::Error;

/// Result type for Stepflow operations
pub type StepflowResult<T> = Result<T, StepflowError>;

/// Main error type for Stepflow Tool System
#[derive(Error, Debug)]
pub enum StepflowError {
    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Tool execution failed: {0}")]
    ToolExecutionFailed(String),

    #[error("Tool validation failed: {0}")]
    ToolValidationFailed(String),

    #[error("Execution not found: {0}")]
    ExecutionNotFound(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Tenant not found: {0}")]
    TenantNotFound(String),

    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Authorization failed: {0}")]
    AuthorizationFailed(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] DatabaseError),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Timeout error: {0}")]
    TimeoutError(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Resource not available: {0}")]
    ResourceNotAvailable(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    #[error("Version mismatch: {0}")]
    VersionMismatch(String),

    #[error("Dependency error: {0}")]
    DependencyError(String),

    #[error("Security violation: {0}")]
    SecurityViolation(String),

    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Already exists: {0}")]
    AlreadyExists(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Cancelled: {0}")]
    Cancelled(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Database-specific errors
#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Query failed: {0}")]
    QueryFailed(String),

    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    #[error("Migration failed: {0}")]
    MigrationFailed(String),

    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),

    #[error("Deadlock detected")]
    Deadlock,

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Connection pool exhausted")]
    PoolExhausted,

    #[error("Invalid schema: {0}")]
    InvalidSchema(String),

    #[error("Data corruption: {0}")]
    DataCorruption(String),
}

/// Validation errors
#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Required field missing: {0}")]
    RequiredFieldMissing(String),

    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    #[error("Value out of range: {0}")]
    ValueOutOfRange(String),

    #[error("Invalid enum value: {0}")]
    InvalidEnumValue(String),

    #[error("String too long: {0}")]
    StringTooLong(String),

    #[error("String too short: {0}")]
    StringTooShort(String),

    #[error("Invalid email: {0}")]
    InvalidEmail(String),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Invalid UUID: {0}")]
    InvalidUuid(String),

    #[error("Invalid JSON: {0}")]
    InvalidJson(String),

    #[error("Invalid date: {0}")]
    InvalidDate(String),

    #[error("Invalid time: {0}")]
    InvalidTime(String),

    #[error("Invalid datetime: {0}")]
    InvalidDateTime(String),

    #[error("Invalid IP address: {0}")]
    InvalidIpAddress(String),

    #[error("Invalid port number: {0}")]
    InvalidPortNumber(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Invalid regex: {0}")]
    InvalidRegex(String),

    #[error("Custom validation failed: {0}")]
    Custom(String),
}

/// Security-related errors
#[derive(Error, Debug)]
pub enum SecurityError {
    #[error("Invalid token: {0}")]
    InvalidToken(String),

    #[error("Token expired: {0}")]
    TokenExpired(String),

    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    #[error("Invalid certificate: {0}")]
    InvalidCertificate(String),

    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("Hash mismatch: {0}")]
    HashMismatch(String),

    #[error("Salt error: {0}")]
    SaltError(String),

    #[error("Key derivation failed: {0}")]
    KeyDerivationFailed(String),

    #[error("Invalid key: {0}")]
    InvalidKey(String),

    #[error("Access denied: {0}")]
    AccessDenied(String),

    #[error("Insufficient permissions: {0}")]
    InsufficientPermissions(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Malicious input detected: {0}")]
    MaliciousInput(String),

    #[error("SQL injection attempt: {0}")]
    SqlInjectionAttempt(String),

    #[error("XSS attempt: {0}")]
    XssAttempt(String),

    #[error("CSRF token missing: {0}")]
    CsrfTokenMissing(String),

    #[error("CSRF token invalid: {0}")]
    CsrfTokenInvalid(String),
}

/// Network-related errors
#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Connection timeout: {0}")]
    ConnectionTimeout(String),

    #[error("Read timeout: {0}")]
    ReadTimeout(String),

    #[error("Write timeout: {0}")]
    WriteTimeout(String),

    #[error("DNS resolution failed: {0}")]
    DnsResolutionFailed(String),

    #[error("SSL/TLS error: {0}")]
    SslTlsError(String),

    #[error("Certificate error: {0}")]
    CertificateError(String),

    #[error("Proxy error: {0}")]
    ProxyError(String),

    #[error("Redirect loop: {0}")]
    RedirectLoop(String),

    #[error("Too many redirects: {0}")]
    TooManyRedirects(String),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Unsupported protocol: {0}")]
    UnsupportedProtocol(String),

    #[error("HTTP error: {0}")]
    HttpError(String),

    #[error("WebSocket error: {0}")]
    WebSocketError(String),
}

/// Configuration errors
#[derive(Error, Debug)]
pub enum ConfigurationError {
    #[error("Missing required configuration: {0}")]
    MissingRequired(String),

    #[error("Invalid configuration value: {0}")]
    InvalidValue(String),

    #[error("Configuration file not found: {0}")]
    FileNotFound(String),

    #[error("Configuration file parse error: {0}")]
    ParseError(String),

    #[error("Environment variable not set: {0}")]
    EnvironmentVariableNotSet(String),

    #[error("Invalid environment variable: {0}")]
    InvalidEnvironmentVariable(String),

    #[error("Configuration validation failed: {0}")]
    ValidationFailed(String),

    #[error("Configuration conflict: {0}")]
    Conflict(String),

    #[error("Configuration deprecated: {0}")]
    Deprecated(String),

    #[error("Configuration migration failed: {0}")]
    MigrationFailed(String),
}

/// Monitoring and observability errors
#[derive(Error, Debug)]
pub enum MonitoringError {
    #[error("Metrics collection failed: {0}")]
    MetricsCollectionFailed(String),

    #[error("Logging failed: {0}")]
    LoggingFailed(String),

    #[error("Tracing failed: {0}")]
    TracingFailed(String),

    #[error("Alert evaluation failed: {0}")]
    AlertEvaluationFailed(String),

    #[error("Health check failed: {0}")]
    HealthCheckFailed(String),

    #[error("Performance profiling failed: {0}")]
    ProfilingFailed(String),

    #[error("Export failed: {0}")]
    ExportFailed(String),

    #[error("Monitoring backend unavailable: {0}")]
    BackendUnavailable(String),

    #[error("Monitoring quota exceeded: {0}")]
    QuotaExceeded(String),

    #[error("Monitoring configuration error: {0}")]
    ConfigurationError(String),
}

impl From<ValidationError> for StepflowError {
    fn from(err: ValidationError) -> Self {
        StepflowError::ValidationError(err.to_string())
    }
}

impl From<SecurityError> for StepflowError {
    fn from(err: SecurityError) -> Self {
        StepflowError::SecurityViolation(err.to_string())
    }
}

impl From<NetworkError> for StepflowError {
    fn from(err: NetworkError) -> Self {
        StepflowError::NetworkError(err.to_string())
    }
}

impl From<ConfigurationError> for StepflowError {
    fn from(err: ConfigurationError) -> Self {
        StepflowError::ConfigurationError(err.to_string())
    }
}

impl From<MonitoringError> for StepflowError {
    fn from(err: MonitoringError) -> Self {
        StepflowError::InternalError(err.to_string())
    }
}

impl From<serde_json::Error> for StepflowError {
    fn from(err: serde_json::Error) -> Self {
        StepflowError::SerializationError(err.to_string())
    }
}

impl From<std::io::Error> for StepflowError {
    fn from(err: std::io::Error) -> Self {
        StepflowError::InternalError(err.to_string())
    }
}

impl From<uuid::Error> for StepflowError {
    fn from(err: uuid::Error) -> Self {
        StepflowError::ValidationError(err.to_string())
    }
}

impl From<chrono::ParseError> for StepflowError {
    fn from(err: chrono::ParseError) -> Self {
        StepflowError::ValidationError(err.to_string())
    }
} 