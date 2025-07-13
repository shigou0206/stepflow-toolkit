use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use stepflow_database::SqliteDatabase;
use stepflow_sandbox::*;

async fn create_test_database() -> SqliteDatabase {
    SqliteDatabase::new("sqlite::memory:").await.unwrap()
}

#[tokio::test]
async fn test_sandbox_id_creation() {
    let id1 = SandboxId::new();
    let id2 = SandboxId::new();
    
    assert!(!id1.as_str().is_empty());
    assert!(!id2.as_str().is_empty());
    assert_ne!(id1, id2);
}

#[tokio::test]
async fn test_sandbox_config_defaults() {
    let config = SandboxConfig::default();
    
    assert_eq!(config.isolation_type, IsolationType::Container);
    assert!(config.resource_limits.memory_limit.is_some());
    assert!(config.resource_limits.cpu_limit.is_some());
    assert!(!config.security_policy.allow_network_access);
    assert!(config.security_policy.allow_file_system_access);
}

#[tokio::test]
async fn test_resource_limits_validation() {
    let db = Arc::new(create_test_database().await);
    let config = ResourceLimitsConfig::default();
    let manager = ResourceLimitsManager::new(db, config);
    
    // Valid limits
    let valid_limits = ResourceLimits {
        memory_limit: Some(256 * 1024 * 1024),
        cpu_limit: Some(1.0),
        ..Default::default()
    };
    
    let sandbox_id = SandboxId::new();
    let result = manager.apply_resource_limits(&sandbox_id, valid_limits).await;
    assert!(result.is_ok());
    
    // Invalid limits - zero memory
    let invalid_limits = ResourceLimits {
        memory_limit: Some(0),
        ..Default::default()
    };
    
    let result = manager.apply_resource_limits(&sandbox_id, invalid_limits).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_security_policy_validation() {
    let db = Arc::new(create_test_database().await);
    let config = SecurityManagerConfig::default();
    let security_manager = SecurityManagerImpl::new(db, config);
    
    // Valid policy
    let valid_policy = SecurityPolicy {
        allow_network_access: false,
        allow_file_system_access: true,
        capabilities: vec!["CAP_CHOWN".to_string()],
        ..Default::default()
    };
    
    let result = security_manager.validate_policy(&valid_policy).await;
    assert!(result.is_ok());
    
    // Invalid policy - invalid capability
    let invalid_policy = SecurityPolicy {
        capabilities: vec!["INVALID_CAP".to_string()],
        ..Default::default()
    };
    
    let result = security_manager.validate_policy(&invalid_policy).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_command_builder() {
    let command = Command::new("echo".to_string())
        .with_args(vec!["hello".to_string(), "world".to_string()])
        .with_env("TEST_VAR".to_string(), "test_value".to_string())
        .with_timeout(Duration::from_secs(30));
    
    assert_eq!(command.program, "echo");
    assert_eq!(command.args, vec!["hello", "world"]);
    assert_eq!(command.environment.get("TEST_VAR"), Some(&"test_value".to_string()));
    assert_eq!(command.timeout, Some(Duration::from_secs(30)));
}

#[tokio::test]
async fn test_container_config_defaults() {
    let config = ContainerConfig::default();
    
    assert_eq!(config.image, "alpine:latest");
    assert_eq!(config.command, vec!["/bin/sh"]);
    assert!(config.environment.is_empty());
    assert!(config.volumes.is_empty());
    assert!(config.ports.is_empty());
}

#[tokio::test]
async fn test_isolation_types() {
    let types = vec![
        IsolationType::Container,
        IsolationType::Namespace,
        IsolationType::Process,
        IsolationType::Chroot,
        IsolationType::None,
    ];
    
    for isolation_type in types {
        let config = SandboxConfig {
            isolation_type: isolation_type.clone(),
            ..Default::default()
        };
        
        assert_eq!(config.isolation_type, isolation_type);
    }
}

#[tokio::test]
async fn test_sandbox_status_transitions() {
    let statuses = vec![
        SandboxStatus::Creating,
        SandboxStatus::Running,
        SandboxStatus::Stopped,
        SandboxStatus::Destroyed,
        SandboxStatus::Error,
    ];
    
    for status in statuses {
        // Test that each status can be created and compared
        let info = SandboxInfo {id:SandboxId::new(),name:"test".to_string(),status:status.clone(),isolation_type:IsolationType::Container,created_at:chrono::Utc::now(),destroyed_at:None,created_by:"test".to_string(),tenant_id:"test".to_string(),resource_usage:ResourceUsage::default(), container_id: todo!() };
        
        assert_eq!(info.status, status);
    }
}

#[tokio::test]
async fn test_security_violation_creation() {
    let sandbox_id = SandboxId::new();
    let violation = SecurityViolation {
        sandbox_id: sandbox_id.clone(),
        violation_type: ViolationType::UnauthorizedSystemCall,
        description: "Test violation".to_string(),
        timestamp: chrono::Utc::now(),
        severity: Severity::High,
        details: HashMap::new(),
    };
    
    assert_eq!(violation.sandbox_id, sandbox_id);
    assert_eq!(violation.violation_type, ViolationType::UnauthorizedSystemCall);
    assert_eq!(violation.severity, Severity::High);
}

#[tokio::test]
async fn test_resource_usage_defaults() {
    let usage = ResourceUsage::default();
    
    assert_eq!(usage.memory_used, 0);
    assert_eq!(usage.cpu_time, Duration::from_secs(0));
    assert_eq!(usage.disk_read, 0);
    assert_eq!(usage.disk_write, 0);
    assert_eq!(usage.network_rx, 0);
    assert_eq!(usage.network_tx, 0);
}

#[tokio::test]
async fn test_sandbox_metrics_defaults() {
    let metrics = SandboxMetrics::default();
    
    assert_eq!(metrics.total_sandboxes, 0);
    assert_eq!(metrics.active_sandboxes, 0);
    assert_eq!(metrics.total_executions, 0);
    assert_eq!(metrics.security_violations, 0);
    assert_eq!(metrics.resource_violations, 0);
    assert_eq!(metrics.average_execution_time, Duration::from_secs(0));
    assert_eq!(metrics.memory_usage, 0);
    assert_eq!(metrics.cpu_usage, 0.0);
}

#[tokio::test]
async fn test_network_config_defaults() {
    let config = NetworkConfig::default();
    
    assert_eq!(config.network_mode, NetworkMode::None);
    assert!(config.port_mappings.is_empty());
    assert!(!config.dns_servers.is_empty());
    assert!(config.hostname.is_none());
}

#[tokio::test]
async fn test_storage_config_defaults() {
    let config = StorageConfig::default();
    
    assert!(config.volumes.is_empty());
    assert!(config.tmpfs_mounts.is_empty());
    assert_eq!(config.working_directory, Some("/workspace".to_string()));
}

#[tokio::test]
async fn test_volume_mount_creation() {
    let volume = VolumeMount {
        host_path: "/host/path".to_string(),
        container_path: "/container/path".to_string(),
        read_only: true,
    };
    
    assert_eq!(volume.host_path, "/host/path");
    assert_eq!(volume.container_path, "/container/path");
    assert!(volume.read_only);
}

#[tokio::test]
async fn test_port_mapping_creation() {
    let port_mapping = PortMapping {
        host_port: 8080,
        container_port: 80,
        protocol: Protocol::TCP,
    };
    
    assert_eq!(port_mapping.host_port, 8080);
    assert_eq!(port_mapping.container_port, 80);
    assert_eq!(port_mapping.protocol, Protocol::TCP);
}

#[tokio::test]
async fn test_execution_result_creation() {
    let result = ExecutionResult {
        exit_code: 0,
        stdout: "Hello, World!".to_string(),
        stderr: "".to_string(),
        execution_time: Duration::from_millis(100),
        resource_usage: ResourceUsage::default(),
    };
    
    assert_eq!(result.exit_code, 0);
    assert_eq!(result.stdout, "Hello, World!");
    assert!(result.stderr.is_empty());
    assert_eq!(result.execution_time, Duration::from_millis(100));
}

#[tokio::test]
async fn test_sandbox_filter_creation() {
    let filter = SandboxFilter {
        status: Some(SandboxStatus::Running),
        isolation_type: Some(IsolationType::Container),
        created_by: Some("test_user".to_string()),
        tenant_id: Some("test_tenant".to_string()),
        created_after: None,
        created_before: None,
    };
    
    assert_eq!(filter.status, Some(SandboxStatus::Running));
    assert_eq!(filter.isolation_type, Some(IsolationType::Container));
    assert_eq!(filter.created_by, Some("test_user".to_string()));
    assert_eq!(filter.tenant_id, Some("test_tenant".to_string()));
}

#[tokio::test]
async fn test_metric_creation() {
    let metric = Metric {
        name: "test_metric".to_string(),
        value: 42.0,
        unit: "count".to_string(),
        timestamp: chrono::Utc::now(),
        labels: HashMap::from([("key".to_string(), "value".to_string())]),
    };
    
    assert_eq!(metric.name, "test_metric");
    assert_eq!(metric.value, 42.0);
    assert_eq!(metric.unit, "count");
    assert_eq!(metric.labels.get("key"), Some(&"value".to_string()));
}

#[tokio::test]
async fn test_vulnerability_creation() {
    let vuln = Vulnerability {
        id: "CVE-2023-1234".to_string(),
        severity: Severity::High,
        description: "Test vulnerability".to_string(),
        affected_component: "test-component".to_string(),
        fix_available: true,
        fix_version: Some("1.2.3".to_string()),
    };
    
    assert_eq!(vuln.id, "CVE-2023-1234");
    assert_eq!(vuln.severity, Severity::High);
    assert!(vuln.fix_available);
    assert_eq!(vuln.fix_version, Some("1.2.3".to_string()));
}

#[tokio::test]
async fn test_seccomp_profile_creation() {
    let profile = SeccompProfile {
        default_action: SeccompAction::Allow,
        syscalls: vec![
            SeccompRule {
                syscall: "mount".to_string(),
                action: SeccompAction::Deny,
                args: None,
            },
        ],
    };
    
    assert_eq!(profile.default_action, SeccompAction::Allow);
    assert_eq!(profile.syscalls.len(), 1);
    assert_eq!(profile.syscalls[0].syscall, "mount");
    assert_eq!(profile.syscalls[0].action, SeccompAction::Deny);
}

#[tokio::test]
async fn test_namespace_config_defaults() {
    let config = NamespaceConfig::default();
    
    assert!(config.enable_pid_namespace);
    assert!(config.enable_network_namespace);
    assert!(config.enable_mount_namespace);
    assert!(config.enable_uts_namespace);
    assert!(config.enable_ipc_namespace);
    assert!(!config.enable_user_namespace); // Usually requires special privileges
}

#[tokio::test]
async fn test_capability_config_defaults() {
    let config = CapabilityConfig::default();
    
    assert!(config.effective_capabilities.is_empty());
    assert!(config.permitted_capabilities.is_empty());
    assert!(config.inheritable_capabilities.is_empty());
    assert!(config.bounding_set.is_empty());
} 