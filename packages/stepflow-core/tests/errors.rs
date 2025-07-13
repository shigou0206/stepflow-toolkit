use stepflow_core::errors::*;
use serde_json;

#[test]
fn test_stepflow_error_basic() {
    let error = StepflowError::ToolNotFound("test-tool".to_string());
    assert_eq!(error.to_string(), "Tool not found: test-tool");
    
    let error = StepflowError::InvalidInput("invalid data".to_string());
    assert_eq!(error.to_string(), "Invalid input: invalid data");
    
    let error = StepflowError::RateLimitExceeded;
    assert_eq!(error.to_string(), "Rate limit exceeded");
}

#[test]
fn test_database_error() {
    let error = DatabaseError::ConnectionFailed("connection failed".to_string());
    assert_eq!(error.to_string(), "Connection failed: connection failed");
    
    let error = DatabaseError::QueryFailed("query failed".to_string());
    assert_eq!(error.to_string(), "Query failed: query failed");
    
    let error = DatabaseError::Deadlock;
    assert_eq!(error.to_string(), "Deadlock detected");
}

#[test]
fn test_validation_error() {
    let error = ValidationError::RequiredFieldMissing("name".to_string());
    assert_eq!(error.to_string(), "Required field missing: name");
    
    let error = ValidationError::InvalidFormat("email".to_string());
    assert_eq!(error.to_string(), "Invalid format: email");
    
    let error = ValidationError::StringTooLong("description".to_string());
    assert_eq!(error.to_string(), "String too long: description");
}

#[test]
fn test_security_error() {
    let error = SecurityError::InvalidToken("token".to_string());
    assert_eq!(error.to_string(), "Invalid token: token");
    
    let error = SecurityError::TokenExpired("token".to_string());
    assert_eq!(error.to_string(), "Token expired: token");
    
    let error = SecurityError::AccessDenied("resource".to_string());
    assert_eq!(error.to_string(), "Access denied: resource");
}

#[test]
fn test_network_error() {
    let error = NetworkError::ConnectionFailed("host".to_string());
    assert_eq!(error.to_string(), "Connection failed: host");
    
    let error = NetworkError::ConnectionTimeout("host".to_string());
    assert_eq!(error.to_string(), "Connection timeout: host");
    
    let error = NetworkError::HttpError("500".to_string());
    assert_eq!(error.to_string(), "HTTP error: 500");
}

#[test]
fn test_configuration_error() {
    let error = ConfigurationError::MissingRequired("database_url".to_string());
    assert_eq!(error.to_string(), "Missing required configuration: database_url");
    
    let error = ConfigurationError::InvalidValue("port".to_string());
    assert_eq!(error.to_string(), "Invalid configuration value: port");
    
    let error = ConfigurationError::FileNotFound("config.toml".to_string());
    assert_eq!(error.to_string(), "Configuration file not found: config.toml");
}

#[test]
fn test_monitoring_error() {
    let error = MonitoringError::MetricsCollectionFailed("cpu".to_string());
    assert_eq!(error.to_string(), "Metrics collection failed: cpu");
    
    let error = MonitoringError::LoggingFailed("file".to_string());
    assert_eq!(error.to_string(), "Logging failed: file");
    
    let error = MonitoringError::HealthCheckFailed("service".to_string());
    assert_eq!(error.to_string(), "Health check failed: service");
}

#[test]
fn test_error_conversions() {
    // Test conversion from ValidationError to StepflowError
    let validation_error = ValidationError::RequiredFieldMissing("field".to_string());
    let stepflow_error: StepflowError = validation_error.into();
    assert!(matches!(stepflow_error, StepflowError::ValidationError(_)));
    
    // Test conversion from SecurityError to StepflowError
    let security_error = SecurityError::InvalidToken("token".to_string());
    let stepflow_error: StepflowError = security_error.into();
    assert!(matches!(stepflow_error, StepflowError::SecurityViolation(_)));
    
    // Test conversion from NetworkError to StepflowError
    let network_error = NetworkError::ConnectionFailed("host".to_string());
    let stepflow_error: StepflowError = network_error.into();
    assert!(matches!(stepflow_error, StepflowError::NetworkError(_)));
    
    // Test conversion from ConfigurationError to StepflowError
    let config_error = ConfigurationError::MissingRequired("key".to_string());
    let stepflow_error: StepflowError = config_error.into();
    assert!(matches!(stepflow_error, StepflowError::ConfigurationError(_)));
    
    // Test conversion from MonitoringError to StepflowError
    let monitoring_error = MonitoringError::MetricsCollectionFailed("metric".to_string());
    let stepflow_error: StepflowError = monitoring_error.into();
    assert!(matches!(stepflow_error, StepflowError::InternalError(_)));
}

#[test]
fn test_serde_json_error_conversion() {
    let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
    let stepflow_error: StepflowError = json_error.into();
    assert!(matches!(stepflow_error, StepflowError::SerializationError(_)));
}

#[test]
fn test_io_error_conversion() {
    use std::io;
    let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let stepflow_error: StepflowError = io_error.into();
    assert!(matches!(stepflow_error, StepflowError::InternalError(_)));
}

#[test]
fn test_uuid_error_conversion() {
    let uuid_error = uuid::Uuid::parse_str("invalid-uuid").unwrap_err();
    let stepflow_error: StepflowError = uuid_error.into();
    assert!(matches!(stepflow_error, StepflowError::ValidationError(_)));
}

#[test]
fn test_chrono_parse_error_conversion() {
    let chrono_error = chrono::DateTime::parse_from_rfc3339("invalid-date").unwrap_err();
    let stepflow_error: StepflowError = chrono_error.into();
    assert!(matches!(stepflow_error, StepflowError::ValidationError(_)));
} 