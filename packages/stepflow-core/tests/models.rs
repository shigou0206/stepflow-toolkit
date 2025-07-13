use stepflow_core::models::*;
use stepflow_core::types::*;
use chrono::{Utc, Duration};
use serde_json::json;

#[test]
fn test_toolspec_basic() {
    let spec = ToolSpec {
        id: ToolId::from_string("tool:ns:name@1.0.0".to_string()),
        name: "Test Tool".to_string(),
        description: "A test tool".to_string(),
        tool_type: ToolType::Custom("custom".to_string()),
        input_schema: json!({"type": "object"}),
        output_schema: json!({"type": "object"}),
        config: json!({}),
        tags: vec!["tag1".to_string(), "tag2".to_string()],
        category: Some("cat".to_string()),
        version: "1.0.0".to_string(),
        tenant_id: TenantId::from_string("tenant-1".to_string()),
        registered_at: Utc::now(),
        execution_config: ExecutionConfig {
            timeout: Duration::seconds(30),
            memory_limit: Some(128),
            cpu_limit: Some(1.0),
            sandbox_level: SandboxLevel::Basic,
            retry_config: RetryConfig {
                max_retries: 2,
                retry_delay: Duration::milliseconds(500),
                backoff_strategy: BackoffStrategy::Exponential,
            },
        },
    };
    assert_eq!(spec.name, "Test Tool");
    assert_eq!(spec.tags.len(), 2);
    assert_eq!(spec.execution_config.sandbox_level, SandboxLevel::Basic);
}

#[test]
fn test_toolspec_serde() {
    let spec = ToolSpec {
        id: ToolId::from_string("tool:ns:name@1.0.0".to_string()),
        name: "Test Tool".to_string(),
        description: "A test tool".to_string(),
        tool_type: ToolType::Custom("custom".to_string()),
        input_schema: json!({"type": "object"}),
        output_schema: json!({"type": "object"}),
        config: json!({}),
        tags: vec!["tag1".to_string()],
        category: None,
        version: "1.0.0".to_string(),
        tenant_id: TenantId::from_string("tenant-1".to_string()),
        registered_at: Utc::now(),
        execution_config: ExecutionConfig {
            timeout: Duration::seconds(10),
            memory_limit: None,
            cpu_limit: None,
            sandbox_level: SandboxLevel::None,
            retry_config: RetryConfig {
                max_retries: 0,
                retry_delay: Duration::milliseconds(100),
                backoff_strategy: BackoffStrategy::Fixed,
            },
        },
    };
    let s = serde_json::to_string(&spec).unwrap();
    let de: ToolSpec = serde_json::from_str(&s).unwrap();
    assert_eq!(de.name, spec.name);
    assert_eq!(de.execution_config.retry_config.max_retries, 0);
}

#[test]
fn test_execution_config() {
    let retry = RetryConfig {
        max_retries: 3,
        retry_delay: Duration::seconds(1),
        backoff_strategy: BackoffStrategy::Linear,
    };
    let config = ExecutionConfig {
        timeout: Duration::seconds(60),
        memory_limit: Some(256),
        cpu_limit: Some(2.0),
        sandbox_level: SandboxLevel::Strict,
        retry_config: retry.clone(),
    };
    assert_eq!(config.retry_config.max_retries, 3);
    assert_eq!(config.sandbox_level, SandboxLevel::Strict);
}

#[test]
fn test_sandbox_level_enum() {
    assert_eq!(SandboxLevel::None as u8, 0);
    assert_eq!(SandboxLevel::Basic as u8, 1);
    assert_eq!(SandboxLevel::Strict as u8, 2);
    assert_eq!(SandboxLevel::Process as u8, 3);
}

#[test]
fn test_backoff_strategy_enum() {
    let s = serde_json::to_string(&BackoffStrategy::Exponential).unwrap();
    assert!(s.contains("Exponential"));
    let b: BackoffStrategy = serde_json::from_str(&s).unwrap();
    match b {
        BackoffStrategy::Exponential => {}
        _ => panic!("Should be Exponential"),
    }
} 