//! Error types for the registry system

use stepflow_core::{ValidationError, StepflowError};

/// Registry error type
#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("Tool not found: {0}")]
    ToolNotFound(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Validation failed")]
    ValidationFailed(Vec<ValidationError>),
    
    #[error("Tool already exists: {0}")]
    ToolAlreadyExists(String),
    
    #[error("Version conflict: {0}")]
    VersionConflict(String),
    
    #[error("Discovery error: {0}")]
    DiscoveryError(String),
    
    #[error("Cache error: {0}")]
    CacheError(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Timeout error: {0}")]
    TimeoutError(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Resource not available: {0}")]
    ResourceNotAvailable(String),
    
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

/// Registry result type
pub type RegistryResult<T> = Result<T, RegistryError>;

impl From<StepflowError> for RegistryError {
    fn from(error: StepflowError) -> Self {
        RegistryError::DatabaseError(error.to_string())
    }
}

impl From<serde_json::Error> for RegistryError {
    fn from(error: serde_json::Error) -> Self {
        RegistryError::InternalError(format!("JSON error: {}", error))
    }
}

impl From<std::io::Error> for RegistryError {
    fn from(error: std::io::Error) -> Self {
        RegistryError::InternalError(format!("IO error: {}", error))
    }
}

/// Validation error builder for creating validation errors
pub struct ValidationErrorBuilder;

impl ValidationErrorBuilder {
    pub fn required_field(field: &str) -> ValidationError {
        ValidationError::RequiredFieldMissing(field.to_string())
    }
    
    pub fn invalid_format(field: &str, description: &str) -> ValidationError {
        ValidationError::InvalidFormat(format!("{}: {}", field, description))
    }
    
    pub fn too_short(field: &str, min: usize, actual: usize) -> ValidationError {
        ValidationError::StringTooShort(format!("{}: expected at least {} characters, got {}", field, min, actual))
    }
    
    pub fn too_long(field: &str, max: usize, actual: usize) -> ValidationError {
        ValidationError::StringTooLong(format!("{}: expected at most {} characters, got {}", field, max, actual))
    }
    
    pub fn invalid_value(field: &str, value: &str) -> ValidationError {
        ValidationError::InvalidEnumValue(format!("{}: invalid value '{}'", field, value))
    }
} 