use async_trait::async_trait;
use crate::types::*;
use crate::errors::*;

/// 沙箱核心特征
#[async_trait]
pub trait Sandbox: Send + Sync {
    /// 创建沙箱环境
    async fn create_sandbox(&self, config: SandboxConfig) -> SandboxResult<SandboxId>;
    
    /// 在沙箱中执行命令
    async fn execute_in_sandbox(&self, sandbox_id: &SandboxId, command: Command) -> SandboxResult<ExecutionResult>;
    
    /// 销毁沙箱环境
    async fn destroy_sandbox(&self, sandbox_id: &SandboxId) -> SandboxResult<()>;
    
    /// 获取沙箱状态
    async fn get_sandbox_status(&self, sandbox_id: &SandboxId) -> SandboxResult<SandboxStatus>;
    
    /// 列出所有沙箱
    async fn list_sandboxes(&self, filter: Option<SandboxFilter>) -> SandboxResult<Vec<SandboxInfo>>;
    
    /// 获取沙箱信息
    async fn get_sandbox_info(&self, sandbox_id: &SandboxId) -> SandboxResult<SandboxInfo>;
    
    /// 更新沙箱配置
    async fn update_sandbox_config(&self, sandbox_id: &SandboxId, config: SandboxConfig) -> SandboxResult<()>;
    
    /// 暂停沙箱
    async fn pause_sandbox(&self, sandbox_id: &SandboxId) -> SandboxResult<()>;
    
    /// 恢复沙箱
    async fn resume_sandbox(&self, sandbox_id: &SandboxId) -> SandboxResult<()>;
    
    /// 获取沙箱日志
    async fn get_sandbox_logs(&self, sandbox_id: &SandboxId, lines: Option<usize>) -> SandboxResult<Vec<String>>;
    
    /// 获取沙箱指标
    async fn get_sandbox_metrics(&self, sandbox_id: &SandboxId) -> SandboxResult<SandboxMetrics>;
    
    /// Health check for the sandbox
    async fn health_check(&self) -> SandboxResult<bool>;
}

/// 容器管理特征
#[async_trait]
pub trait ContainerManager: Send + Sync {
    /// 创建容器
    async fn create_container(&self, config: ContainerConfig) -> ContainerResult<ContainerId>;
    
    /// 启动容器
    async fn start_container(&self, container_id: &ContainerId) -> ContainerResult<()>;
    
    /// 停止容器
    async fn stop_container(&self, container_id: &ContainerId) -> ContainerResult<()>;
    
    /// 删除容器
    async fn delete_container(&self, container_id: &ContainerId) -> ContainerResult<()>;
    
    /// 获取容器状态
    async fn get_container_status(&self, container_id: &ContainerId) -> ContainerResult<ContainerStatus>;
    
    /// 获取容器信息
    async fn get_container_info(&self, container_id: &ContainerId) -> ContainerResult<ContainerInfo>;
    
    /// 列出所有容器
    async fn list_containers(&self) -> ContainerResult<Vec<ContainerInfo>>;
    
    /// 执行容器命令
    async fn execute_in_container(&self, container_id: &ContainerId, command: Command) -> ContainerResult<ExecutionResult>;
    
    /// 获取容器日志
    async fn get_container_logs(&self, container_id: &ContainerId, lines: Option<usize>) -> ContainerResult<Vec<String>>;
    
    /// 获取容器资源使用情况
    async fn get_container_stats(&self, container_id: &ContainerId) -> ContainerResult<ResourceUsage>;
    
    /// 暂停容器
    async fn pause_container(&self, container_id: &ContainerId) -> ContainerResult<()>;
    
    /// 恢复容器
    async fn resume_container(&self, container_id: &ContainerId) -> ContainerResult<()>;
}

/// 隔离管理特征
#[async_trait]
pub trait IsolationManager: Send + Sync {
    /// 创建命名空间隔离
    async fn create_namespace_isolation(&self, config: NamespaceConfig) -> IsolationResult<NamespaceId>;
    
    /// 应用安全策略
    async fn apply_security_policy(&self, sandbox_id: &SandboxId, policy: SecurityPolicy) -> IsolationResult<()>;
    
    /// 设置资源限制
    async fn set_resource_limits(&self, sandbox_id: &SandboxId, limits: ResourceLimits) -> IsolationResult<()>;
    
    /// 监控隔离状态
    async fn monitor_isolation(&self, sandbox_id: &SandboxId) -> IsolationResult<IsolationStatus>;
    
    /// 销毁隔离环境
    async fn destroy_isolation(&self, isolation_id: &IsolationId) -> IsolationResult<()>;
    
    /// 获取隔离信息
    async fn get_isolation_info(&self, isolation_id: &IsolationId) -> IsolationResult<IsolationStatus>;
    
    /// 应用 Seccomp 策略
    async fn apply_seccomp_policy(&self, sandbox_id: &SandboxId, profile: SeccompProfile) -> IsolationResult<()>;
    
    /// 设置能力限制
    async fn set_capabilities(&self, sandbox_id: &SandboxId, capabilities: CapabilityConfig) -> IsolationResult<()>;
    
    /// 验证安全策略
    async fn validate_security_policy(&self, policy: &SecurityPolicy) -> IsolationResult<()>;
}

/// 安全管理特征
#[async_trait]
pub trait SecurityManager: Send + Sync {
    /// 验证安全策略
    async fn validate_policy(&self, policy: &SecurityPolicy) -> SecurityResult<()>;
    
    /// 应用安全策略
    async fn apply_policy(&self, sandbox_id: &SandboxId, policy: &SecurityPolicy) -> SecurityResult<()>;
    
    /// 监控安全策略
    async fn monitor_policy(&self, sandbox_id: &SandboxId) -> SecurityResult<PolicyStatus>;
    
    /// 记录安全违规
    async fn record_violation(&self, violation: SecurityViolation) -> SecurityResult<()>;
    
    /// 获取安全违规列表
    async fn get_violations(&self, sandbox_id: &SandboxId) -> SecurityResult<Vec<SecurityViolation>>;
    
    /// 审计沙箱创建
    async fn audit_sandbox_creation(&self, sandbox_id: &SandboxId, config: &SandboxConfig) -> SecurityResult<()>;
    
    /// 审计命令执行
    async fn audit_execution(&self, sandbox_id: &SandboxId, command: &Command) -> SecurityResult<()>;
    
    /// 检查权限
    async fn check_permission(&self, sandbox_id: &SandboxId, action: &str) -> SecurityResult<bool>;
}

/// 沙箱监控特征
#[async_trait]
pub trait SandboxMonitoring: Send + Sync {
    /// 记录沙箱创建
    async fn record_sandbox_creation(&self, sandbox_id: &SandboxId) -> MonitoringResult<()>;
    
    /// 记录沙箱销毁
    async fn record_sandbox_destruction(&self, sandbox_id: &SandboxId) -> MonitoringResult<()>;
    
    /// 记录执行
    async fn record_execution(&self, sandbox_id: &SandboxId, execution: &ExecutionResult) -> MonitoringResult<()>;
    
    /// 记录安全违规
    async fn record_security_violation(&self, sandbox_id: &SandboxId, violation: &SecurityViolation) -> MonitoringResult<()>;
    
    /// 获取指标
    async fn get_metrics(&self, filter: Option<MetricFilter>) -> MonitoringResult<Vec<Metric>>;
    
    /// 获取沙箱指标
    async fn get_sandbox_metrics(&self, sandbox_id: &SandboxId) -> MonitoringResult<SandboxMetrics>;
    
    /// 记录指标
    async fn record_metric(&self, metric: Metric) -> MonitoringResult<()>;
    
    /// 获取系统指标
    async fn get_system_metrics(&self) -> MonitoringResult<SandboxMetrics>;
}

/// 漏洞扫描特征
#[async_trait]
pub trait VulnerabilityScanner: Send + Sync {
    /// 扫描容器
    async fn scan_container(&self, container_id: &ContainerId) -> ScanResult<Vec<Vulnerability>>;
    
    /// 扫描镜像
    async fn scan_image(&self, image: &str) -> ScanResult<Vec<Vulnerability>>;
    
    /// 获取扫描历史
    async fn get_scan_history(&self, container_id: &ContainerId) -> ScanResult<Vec<ScanResult<Vec<Vulnerability>>>>;
    
    /// 更新漏洞数据库
    async fn update_vulnerability_database(&self) -> ScanResult<()>;
}

/// Seccomp 管理特征
#[async_trait]
pub trait SeccompManager: Send + Sync {
    /// 应用 Seccomp 配置
    async fn apply_profile(&self, sandbox_id: &SandboxId, profile: &str) -> IsolationResult<()>;
    
    /// 创建 Seccomp 配置
    async fn create_profile(&self, profile: SeccompProfile) -> IsolationResult<String>;
    
    /// 验证 Seccomp 配置
    async fn validate_profile(&self, profile: &SeccompProfile) -> IsolationResult<()>;
    
    /// 获取默认配置
    async fn get_default_profile(&self) -> IsolationResult<SeccompProfile>;
}

/// 命名空间管理特征
#[async_trait]
pub trait NamespaceManager: Send + Sync {
    /// 设置命名空间
    async fn setup_namespaces(&self, sandbox_id: &SandboxId, policy: &SecurityPolicy) -> IsolationResult<()>;
    
    /// 创建命名空间
    async fn create_namespace(&self, config: NamespaceConfig) -> IsolationResult<NamespaceId>;
    
    /// 销毁命名空间
    async fn destroy_namespace(&self, namespace_id: &NamespaceId) -> IsolationResult<()>;
    
    /// 获取命名空间状态
    async fn get_namespace_status(&self, namespace_id: &NamespaceId) -> IsolationResult<IsolationStatus>;
}

/// 自定义隔离特征
#[async_trait]
pub trait CustomIsolation: Send + Sync {
    /// 创建自定义隔离
    async fn create_isolation(&self, config: serde_json::Value) -> IsolationResult<IsolationId>;
    
    /// 销毁自定义隔离
    async fn destroy_isolation(&self, isolation_id: &IsolationId) -> IsolationResult<()>;
    
    /// 在隔离环境中执行
    async fn execute_in_isolation(&self, isolation_id: &IsolationId, command: Command) -> IsolationResult<ExecutionResult>;
}

/// 自定义安全策略特征
#[async_trait]
pub trait CustomSecurityPolicy: Send + Sync {
    /// 验证策略
    fn validate_policy(&self, policy: &SecurityPolicy) -> SecurityResult<()>;
    
    /// 应用策略
    fn apply_policy(&self, sandbox_id: &SandboxId, policy: &SecurityPolicy) -> SecurityResult<()>;
    
    /// 监控策略
    fn monitor_policy(&self, sandbox_id: &SandboxId) -> SecurityResult<PolicyStatus>;
} 