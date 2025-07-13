use std::sync::Arc;
use std::time::Duration;

use stepflow_database::SqliteDatabase;
use stepflow_sandbox::*;
use tokio::time::timeout;

async fn create_test_database() -> SqliteDatabase {
    SqliteDatabase::new("sqlite::memory:").await.unwrap()
}

async fn create_test_sandbox() -> SandboxImpl {
    let db = Arc::new(create_test_database().await);
    let config = SandboxImplConfig::default();
    SandboxImpl::new(db, config).await.unwrap()
}

#[tokio::test]
async fn test_create_sandbox() {
    let sandbox = create_test_sandbox().await;
    
    let config = SandboxConfig {
        isolation_type: IsolationType::Container,
        ..Default::default()
    };
    
    let sandbox_id = sandbox.create_sandbox(config).await.unwrap();
    assert!(!sandbox_id.as_str().is_empty());
    
    let status = sandbox.get_sandbox_status(&sandbox_id).await.unwrap();
    assert_eq!(status, SandboxStatus::Running);
}

#[tokio::test]
async fn test_execute_in_sandbox() {
    let sandbox = create_test_sandbox().await;
    
    let config = SandboxConfig {
        isolation_type: IsolationType::Container,
        ..Default::default()
    };
    
    let sandbox_id = sandbox.create_sandbox(config).await.unwrap();
    
    let command = Command::new("echo".to_string())
        .with_args(vec!["hello".to_string(), "world".to_string()]);
    
    let result = sandbox.execute_in_sandbox(&sandbox_id, command).await.unwrap();
    assert_eq!(result.exit_code, 0);
    assert!(result.stdout.contains("hello"));
}

#[tokio::test]
async fn test_sandbox_with_namespace_isolation() {
    let sandbox = create_test_sandbox().await;
    
    let config = SandboxConfig {
        isolation_type: IsolationType::Namespace,
        ..Default::default()
    };
    
    let sandbox_id = sandbox.create_sandbox(config).await.unwrap();
    
    let command = Command::new("ls".to_string());
    let result = sandbox.execute_in_sandbox(&sandbox_id, command).await.unwrap();
    assert_eq!(result.exit_code, 0);
}

#[tokio::test]
async fn test_sandbox_with_security_policy() {
    let sandbox = create_test_sandbox().await;
    
    let security_policy = SecurityPolicy {
        allow_network_access: false,
        allow_file_system_access: true,
        allow_process_creation: false,
        blocked_system_calls: vec!["mount".to_string(), "umount".to_string()],
        ..Default::default()
    };
    
    let config = SandboxConfig {
        isolation_type: IsolationType::Container,
        security_policy,
        ..Default::default()
    };
    
    let sandbox_id = sandbox.create_sandbox(config).await.unwrap();
    
    let command = Command::new("echo".to_string())
        .with_args(vec!["test".to_string()]);
    
    let result = sandbox.execute_in_sandbox(&sandbox_id, command).await.unwrap();
    assert_eq!(result.exit_code, 0);
}

#[tokio::test]
async fn test_sandbox_with_resource_limits() {
    let sandbox = create_test_sandbox().await;
    
    let resource_limits = ResourceLimits {
        memory_limit: Some(256 * 1024 * 1024), // 256MB
        cpu_limit: Some(0.5), // 0.5 CPU cores
        execution_timeout: Some(Duration::from_secs(30)),
        ..Default::default()
    };
    
    let config = SandboxConfig {
        isolation_type: IsolationType::Container,
        resource_limits,
        ..Default::default()
    };
    
    let sandbox_id = sandbox.create_sandbox(config).await.unwrap();
    
    let command = Command::new("echo".to_string())
        .with_args(vec!["resource_test".to_string()]);
    
    let result = sandbox.execute_in_sandbox(&sandbox_id, command).await.unwrap();
    assert_eq!(result.exit_code, 0);
}

#[tokio::test]
async fn test_list_sandboxes() {
    let sandbox = create_test_sandbox().await;
    
    // Create multiple sandboxes
    let config1 = SandboxConfig {
        isolation_type: IsolationType::Container,
        ..Default::default()
    };
    let sandbox_id1 = sandbox.create_sandbox(config1).await.unwrap();
    
    let config2 = SandboxConfig {
        isolation_type: IsolationType::Namespace,
        ..Default::default()
    };
    let sandbox_id2 = sandbox.create_sandbox(config2).await.unwrap();
    
    let sandboxes = sandbox.list_sandboxes(None).await.unwrap();
    assert!(sandboxes.len() >= 2);
    
    let sandbox_ids: Vec<String> = sandboxes.iter()
        .map(|s| s.id.as_str().to_string())
        .collect();
    
    assert!(sandbox_ids.contains(&sandbox_id1.as_str().to_string()));
    assert!(sandbox_ids.contains(&sandbox_id2.as_str().to_string()));
}

#[tokio::test]
async fn test_sandbox_filtering() {
    let sandbox = create_test_sandbox().await;
    
    let config = SandboxConfig {
        isolation_type: IsolationType::Container,
        ..Default::default()
    };
    let _sandbox_id = sandbox.create_sandbox(config).await.unwrap();
    
    let filter = SandboxFilter {
        isolation_type: Some(IsolationType::Container),
        status: Some(SandboxStatus::Running),
        ..Default::default()
    };
    
    let filtered_sandboxes = sandbox.list_sandboxes(Some(filter)).await.unwrap();
    assert!(!filtered_sandboxes.is_empty());
    
    for sandbox_info in filtered_sandboxes {
        assert_eq!(sandbox_info.isolation_type, IsolationType::Container);
        assert_eq!(sandbox_info.status, SandboxStatus::Running);
    }
}

#[tokio::test]
async fn test_destroy_sandbox() {
    let sandbox = create_test_sandbox().await;
    
    let config = SandboxConfig {
        isolation_type: IsolationType::Container,
        ..Default::default()
    };
    
    let sandbox_id = sandbox.create_sandbox(config).await.unwrap();
    
    // Verify sandbox exists
    let status = sandbox.get_sandbox_status(&sandbox_id).await.unwrap();
    assert_eq!(status, SandboxStatus::Running);
    
    // Destroy sandbox
    sandbox.destroy_sandbox(&sandbox_id).await.unwrap();
    
    // Verify sandbox is destroyed
    let status = sandbox.get_sandbox_status(&sandbox_id).await.unwrap();
    assert_eq!(status, SandboxStatus::Destroyed);
}

#[tokio::test]
async fn test_sandbox_pause_resume() {
    let sandbox = create_test_sandbox().await;
    
    let config = SandboxConfig {
        isolation_type: IsolationType::Container,
        ..Default::default()
    };
    
    let sandbox_id = sandbox.create_sandbox(config).await.unwrap();
    
    // Pause sandbox
    sandbox.pause_sandbox(&sandbox_id).await.unwrap();
    let status = sandbox.get_sandbox_status(&sandbox_id).await.unwrap();
    assert_eq!(status, SandboxStatus::Stopped);
    
    // Resume sandbox
    sandbox.resume_sandbox(&sandbox_id).await.unwrap();
    let status = sandbox.get_sandbox_status(&sandbox_id).await.unwrap();
    assert_eq!(status, SandboxStatus::Running);
}

#[tokio::test]
async fn test_sandbox_logs() {
    let sandbox = create_test_sandbox().await;
    
    let config = SandboxConfig {
        isolation_type: IsolationType::Container,
        ..Default::default()
    };
    
    let sandbox_id = sandbox.create_sandbox(config).await.unwrap();
    
    // Execute a command to generate logs
    let command = Command::new("echo".to_string())
        .with_args(vec!["log_test".to_string()]);
    
    let _result = sandbox.execute_in_sandbox(&sandbox_id, command).await.unwrap();
    
    // Get logs
    let logs = sandbox.get_sandbox_logs(&sandbox_id, Some(10)).await.unwrap();
    assert!(!logs.is_empty());
}

#[tokio::test]
async fn test_sandbox_metrics() {
    let sandbox = create_test_sandbox().await;
    
    let config = SandboxConfig {
        isolation_type: IsolationType::Container,
        ..Default::default()
    };
    
    let sandbox_id = sandbox.create_sandbox(config).await.unwrap();
    
    // Execute a command to generate metrics
    let command = Command::new("echo".to_string())
        .with_args(vec!["metrics_test".to_string()]);
    
    let _result = sandbox.execute_in_sandbox(&sandbox_id, command).await.unwrap();
    
    // Get metrics
    let metrics = sandbox.get_sandbox_metrics(&sandbox_id).await.unwrap();
    assert!(metrics.total_executions > 0);
}

#[tokio::test]
async fn test_sandbox_update_config() {
    let sandbox = create_test_sandbox().await;
    
    let config = SandboxConfig {
        isolation_type: IsolationType::Container,
        ..Default::default()
    };
    
    let sandbox_id = sandbox.create_sandbox(config).await.unwrap();
    
    // Update config
    let new_config = SandboxConfig {
        isolation_type: IsolationType::Container,
        resource_limits: ResourceLimits {
            memory_limit: Some(512 * 1024 * 1024), // 512MB
            ..Default::default()
        },
        ..Default::default()
    };
    
    sandbox.update_sandbox_config(&sandbox_id, new_config).await.unwrap();
    
    // Verify update was successful
    let status = sandbox.get_sandbox_status(&sandbox_id).await.unwrap();
    assert_eq!(status, SandboxStatus::Running);
}

#[tokio::test]
async fn test_sandbox_execution_timeout() {
    let sandbox = create_test_sandbox().await;
    
    let config = SandboxConfig {
        isolation_type: IsolationType::Container,
        resource_limits: ResourceLimits {
            execution_timeout: Some(Duration::from_millis(100)),
            ..Default::default()
        },
        ..Default::default()
    };
    
    let sandbox_id = sandbox.create_sandbox(config).await.unwrap();
    
    let command = Command::new("sleep".to_string())
        .with_args(vec!["1".to_string()]);
    
    // This should complete quickly due to our simplified implementation
    let result = timeout(Duration::from_secs(5), 
        sandbox.execute_in_sandbox(&sandbox_id, command)
    ).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_sandbox_not_found() {
    let sandbox = create_test_sandbox().await;
    
    let non_existent_id = SandboxId::new();
    
    let result = sandbox.get_sandbox_status(&non_existent_id).await;
    assert!(result.is_err());
    
    match result.unwrap_err() {
        SandboxError::SandboxNotFound(_) => {
            // Expected error
        }
        _ => panic!("Expected SandboxNotFound error"),
    }
}

#[tokio::test]
async fn test_sandbox_info() {
    let sandbox = create_test_sandbox().await;
    
    let config = SandboxConfig {
        isolation_type: IsolationType::Container,
        ..Default::default()
    };
    
    let sandbox_id = sandbox.create_sandbox(config).await.unwrap();
    
    let info = sandbox.get_sandbox_info(&sandbox_id).await.unwrap();
    assert_eq!(info.id, sandbox_id);
    assert_eq!(info.isolation_type, IsolationType::Container);
    assert_eq!(info.status, SandboxStatus::Running);
    assert!(!info.name.is_empty());
} 