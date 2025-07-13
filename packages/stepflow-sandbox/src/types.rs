use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// 沙箱 ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SandboxId(pub String);

impl SandboxId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
    
    pub fn from_string(s: String) -> Self {
        Self(s)
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for SandboxId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for SandboxId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for SandboxId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// 容器 ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContainerId(pub String);

impl ContainerId {
    pub fn new(id: String) -> Self {
        Self(id)
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for ContainerId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// 命名空间 ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NamespaceId(pub String);

impl NamespaceId {
    pub fn new(id: String) -> Self {
        Self(id)
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 隔离 ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IsolationId(pub String);

impl IsolationId {
    pub fn new(id: String) -> Self {
        Self(id)
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 隔离类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IsolationType {
    Container,
    Namespace,
    Chroot,
    Process,
    None,
}

/// 沙箱状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SandboxStatus {
    Creating,
    Running,
    Stopped,
    Destroyed,
    Error,
}

/// 容器状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContainerStatus {
    Created,
    Running,
    Paused,
    Stopped,
    Removing,
    Exited,
    Dead,
}

/// 隔离状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IsolationStatus {
    Active,
    Inactive,
    Failed,
}

/// 资源限制
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub memory_limit: Option<usize>,
    pub cpu_limit: Option<f64>,
    pub disk_limit: Option<usize>,
    pub network_bandwidth: Option<usize>,
    pub process_limit: Option<usize>,
    pub file_descriptor_limit: Option<usize>,
    pub execution_timeout: Option<Duration>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            memory_limit: Some(512 * 1024 * 1024), // 512MB
            cpu_limit: Some(1.0), // 1 CPU
            disk_limit: Some(1024 * 1024 * 1024), // 1GB
            network_bandwidth: Some(100 * 1024 * 1024), // 100MB/s
            process_limit: Some(100),
            file_descriptor_limit: Some(1024),
            execution_timeout: Some(Duration::from_secs(300)), // 5 minutes
        }
    }
}

/// 安全策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    pub allow_network_access: bool,
    pub allow_file_system_access: bool,
    pub allow_process_creation: bool,
    pub allow_system_calls: Vec<String>,
    pub blocked_system_calls: Vec<String>,
    pub seccomp_profile: Option<String>,
    pub capabilities: Vec<String>,
    pub read_only_root: bool,
    pub no_new_privileges: bool,
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        Self {
            allow_network_access: false,
            allow_file_system_access: true,
            allow_process_creation: true,
            allow_system_calls: vec![],
            blocked_system_calls: vec![
                "mount".to_string(),
                "umount".to_string(),
                "reboot".to_string(),
                "kexec_load".to_string(),
            ],
            seccomp_profile: None,
            capabilities: vec![],
            read_only_root: false,
            no_new_privileges: true,
        }
    }
}

/// 网络配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub network_mode: NetworkMode,
    pub port_mappings: Vec<PortMapping>,
    pub dns_servers: Vec<String>,
    pub hostname: Option<String>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            network_mode: NetworkMode::None,
            port_mappings: vec![],
            dns_servers: vec!["8.8.8.8".to_string(), "8.8.4.4".to_string()],
            hostname: None,
        }
    }
}

/// 网络模式
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkMode {
    Bridge,
    Host,
    None,
    Container(String),
}

/// 端口映射
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortMapping {
    pub host_port: u16,
    pub container_port: u16,
    pub protocol: Protocol,
}

/// 协议类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Protocol {
    TCP,
    UDP,
}

/// 存储配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub volumes: Vec<VolumeMount>,
    pub tmpfs_mounts: Vec<TmpfsMount>,
    pub working_directory: Option<String>,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            volumes: vec![],
            tmpfs_mounts: vec![],
            working_directory: Some("/workspace".to_string()),
        }
    }
}

/// 卷挂载
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeMount {
    pub host_path: String,
    pub container_path: String,
    pub read_only: bool,
}

/// tmpfs 挂载
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmpfsMount {
    pub container_path: String,
    pub size: Option<usize>,
    pub mode: Option<u32>,
}

/// 沙箱配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub isolation_type: IsolationType,
    pub resource_limits: ResourceLimits,
    pub security_policy: SecurityPolicy,
    pub network_config: NetworkConfig,
    pub storage_config: StorageConfig,
    pub environment: HashMap<String, String>,
    pub container_config: Option<ContainerConfig>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            isolation_type: IsolationType::Container,
            resource_limits: ResourceLimits::default(),
            security_policy: SecurityPolicy::default(),
            network_config: NetworkConfig::default(),
            storage_config: StorageConfig::default(),
            environment: HashMap::new(),
            container_config: None,
        }
    }
}

/// 容器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerConfig {
    pub image: String,
    pub command: Vec<String>,
    pub environment: HashMap<String, String>,
    pub volumes: Vec<VolumeMount>,
    pub ports: Vec<PortMapping>,
    pub resource_limits: ResourceLimits,
    pub security_options: Vec<String>,
    pub labels: HashMap<String, String>,
}

impl Default for ContainerConfig {
    fn default() -> Self {
        Self {
            image: "alpine:latest".to_string(),
            command: vec!["/bin/sh".to_string()],
            environment: HashMap::new(),
            volumes: vec![],
            ports: vec![],
            resource_limits: ResourceLimits::default(),
            security_options: vec![],
            labels: HashMap::new(),
        }
    }
}

/// 执行命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    pub program: String,
    pub args: Vec<String>,
    pub environment: HashMap<String, String>,
    pub working_directory: Option<String>,
    pub timeout: Option<Duration>,
}

impl Command {
    pub fn new(program: String) -> Self {
        Self {
            program,
            args: vec![],
            environment: HashMap::new(),
            working_directory: None,
            timeout: None,
        }
    }
    
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }
    
    pub fn with_env(mut self, key: String, value: String) -> Self {
        self.environment.insert(key, value);
        self
    }
    
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

/// 执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub execution_time: Duration,
    pub resource_usage: ResourceUsage,
}

/// 资源使用情况
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub memory_used: usize,
    pub memory_usage: usize,
    pub cpu_time: Duration,
    pub cpu_usage: f64,
    pub disk_read: usize,
    pub disk_write: usize,
    pub disk_usage: usize,
    pub network_rx: usize,
    pub network_tx: usize,
    pub processes: u32,
}

impl Default for ResourceUsage {
    fn default() -> Self {
        Self {
            memory_used: 0,
            memory_usage: 0,
            cpu_time: Duration::from_secs(0),
            cpu_usage: 0.0,
            disk_read: 0,
            disk_write: 0,
            disk_usage: 0,
            network_rx: 0,
            network_tx: 0,
            processes: 0,
        }
    }
}

/// 沙箱信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxInfo {
    pub id: SandboxId,
    pub name: String,
    pub status: SandboxStatus,
    pub isolation_type: IsolationType,
    pub container_id: ContainerId,
    pub created_at: DateTime<Utc>,
    pub destroyed_at: Option<DateTime<Utc>>,
    pub created_by: String,
    pub tenant_id: String,
    pub resource_usage: ResourceUsage,
}

/// 沙箱过滤器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxFilter {
    pub status: Option<SandboxStatus>,
    pub isolation_type: Option<IsolationType>,
    pub created_by: Option<String>,
    pub tenant_id: Option<String>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}

impl Default for SandboxFilter {
    fn default() -> Self {
        Self {
            status: None,
            isolation_type: None,
            created_by: None,
            tenant_id: None,
            created_after: None,
            created_before: None,
        }
    }
}

/// 容器信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerInfo {
    pub id: ContainerId,
    pub name: String,
    pub image: String,
    pub status: ContainerStatus,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub resource_usage: ResourceUsage,
}

/// 命名空间配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespaceConfig {
    pub enable_pid_namespace: bool,
    pub enable_network_namespace: bool,
    pub enable_mount_namespace: bool,
    pub enable_uts_namespace: bool,
    pub enable_ipc_namespace: bool,
    pub enable_user_namespace: bool,
}

impl Default for NamespaceConfig {
    fn default() -> Self {
        Self {
            enable_pid_namespace: true,
            enable_network_namespace: true,
            enable_mount_namespace: true,
            enable_uts_namespace: true,
            enable_ipc_namespace: true,
            enable_user_namespace: false, // 通常需要特殊权限
        }
    }
}

/// 能力配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityConfig {
    pub effective_capabilities: Vec<String>,
    pub permitted_capabilities: Vec<String>,
    pub inheritable_capabilities: Vec<String>,
    pub bounding_set: Vec<String>,
}

impl Default for CapabilityConfig {
    fn default() -> Self {
        Self {
            effective_capabilities: vec![],
            permitted_capabilities: vec![],
            inheritable_capabilities: vec![],
            bounding_set: vec![],
        }
    }
}

/// Seccomp 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeccompProfile {
    pub default_action: SeccompAction,
    pub syscalls: Vec<SeccompRule>,
}

/// Seccomp 动作
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeccompAction {
    Allow,
    Deny,
    Trap,
    Kill,
    Trace,
}

/// Seccomp 规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeccompRule {
    pub syscall: String,
    pub action: SeccompAction,
    pub args: Option<Vec<SeccompArg>>,
}

/// Seccomp 参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeccompArg {
    pub index: u32,
    pub value: u64,
    pub op: SeccompOp,
}

/// Seccomp 操作
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeccompOp {
    NotEqual,
    LessThan,
    LessEqual,
    Equal,
    GreaterEqual,
    GreaterThan,
    MaskedEqual,
}

/// 安全违规
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityViolation {
    pub sandbox_id: SandboxId,
    pub violation_type: ViolationType,
    pub description: String,
    pub timestamp: DateTime<Utc>,
    pub severity: Severity,
    pub details: HashMap<String, String>,
}

/// 违规类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationType {
    UnauthorizedSystemCall,
    ResourceLimitExceeded,
    NetworkViolation,
    FileSystemViolation,
    ProcessViolation,
    CapabilityViolation,
}

/// 严重程度
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// 漏洞信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    pub id: String,
    pub severity: Severity,
    pub description: String,
    pub affected_component: String,
    pub fix_available: bool,
    pub fix_version: Option<String>,
}

/// 策略状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyStatus {
    pub active: bool,
    pub violations: u64,
    pub last_violation: Option<DateTime<Utc>>,
}

/// 指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub value: f64,
    pub unit: String,
    pub timestamp: DateTime<Utc>,
    pub labels: HashMap<String, String>,
}

/// 指标过滤器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricFilter {
    pub name: Option<String>,
    pub labels: HashMap<String, String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

/// 沙箱指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxMetrics {
    pub total_sandboxes: u64,
    pub active_sandboxes: u64,
    pub total_executions: u64,
    pub security_violations: u64,
    pub resource_violations: u64,
    pub average_execution_time: Duration,
    pub memory_usage: usize,
    pub cpu_usage: f64,
    pub disk_usage: usize,
    pub network_rx: usize,
    pub network_tx: usize,
    pub processes: u32,
    pub uptime: u64,
}

impl Default for SandboxMetrics {
    fn default() -> Self {
        Self {
            total_sandboxes: 0,
            active_sandboxes: 0,
            total_executions: 0,
            security_violations: 0,
            resource_violations: 0,
            average_execution_time: Duration::from_secs(0),
            memory_usage: 0,
            cpu_usage: 0.0,
            disk_usage: 0,
            network_rx: 0,
            network_tx: 0,
            processes: 0,
            uptime: 0,
        }
    }
} 