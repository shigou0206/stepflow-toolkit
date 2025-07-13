use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use stepflow_database::SqliteDatabase;
use tracing::{debug, error, info, warn};

use crate::errors::*;
use crate::sandbox::SecurityManager;
use crate::types::*;

/// 安全管理器配置
#[derive(Debug, Clone)]
pub struct SecurityManagerConfig {
    pub enable_audit_logging: bool,
    pub enable_violation_tracking: bool,
    pub max_violations_per_sandbox: usize,
    pub violation_retention_days: u32,
    pub enable_real_time_monitoring: bool,
    pub alert_on_critical_violations: bool,
}

impl Default for SecurityManagerConfig {
    fn default() -> Self {
        Self {
            enable_audit_logging: true,
            enable_violation_tracking: true,
            max_violations_per_sandbox: 100,
            violation_retention_days: 30,
            enable_real_time_monitoring: true,
            alert_on_critical_violations: true,
        }
    }
}

/// 安全管理器实现
pub struct SecurityManagerImpl {
    db: Arc<SqliteDatabase>,
    config: SecurityManagerConfig,
    violations: Arc<tokio::sync::RwLock<HashMap<SandboxId, Vec<SecurityViolation>>>>,
    policies: Arc<tokio::sync::RwLock<HashMap<SandboxId, SecurityPolicy>>>,
}

impl SecurityManagerImpl {
    pub fn new(db: Arc<SqliteDatabase>, config: SecurityManagerConfig) -> Self {
        Self {
            db,
            config,
            violations: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            policies: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    /// 验证安全策略的内部实现
    fn validate_policy_internal(&self, policy: &SecurityPolicy) -> SecurityResult<()> {
        // 验证系统调用列表
        for syscall in &policy.blocked_system_calls {
            if syscall.is_empty() {
                return Err(SecurityError::PolicyValidationFailed(
                    "Empty syscall name in blocked list".to_string()
                ));
            }
        }

        for syscall in &policy.allow_system_calls {
            if syscall.is_empty() {
                return Err(SecurityError::PolicyValidationFailed(
                    "Empty syscall name in allow list".to_string()
                ));
            }
        }

        // 验证能力设置
        for capability in &policy.capabilities {
            if !self.is_valid_capability(capability) {
                return Err(SecurityError::PolicyValidationFailed(
                    format!("Invalid capability: {}", capability)
                ));
            }
        }

        // 验证 seccomp 配置
        if let Some(profile) = &policy.seccomp_profile {
            if profile.is_empty() {
                return Err(SecurityError::PolicyValidationFailed(
                    "Empty seccomp profile".to_string()
                ));
            }
        }

        // 验证策略一致性
        if !policy.allow_network_access && !policy.allow_file_system_access && !policy.allow_process_creation {
            warn!("Very restrictive policy: no network, filesystem, or process creation allowed");
        }

        Ok(())
    }

    /// 检查能力是否有效
    fn is_valid_capability(&self, capability: &str) -> bool {
        let valid_capabilities = vec![
            "CAP_CHOWN",
            "CAP_DAC_OVERRIDE",
            "CAP_DAC_READ_SEARCH",
            "CAP_FOWNER",
            "CAP_FSETID",
            "CAP_KILL",
            "CAP_SETGID",
            "CAP_SETUID",
            "CAP_SETPCAP",
            "CAP_LINUX_IMMUTABLE",
            "CAP_NET_BIND_SERVICE",
            "CAP_NET_BROADCAST",
            "CAP_NET_ADMIN",
            "CAP_NET_RAW",
            "CAP_IPC_LOCK",
            "CAP_IPC_OWNER",
            "CAP_SYS_MODULE",
            "CAP_SYS_RAWIO",
            "CAP_SYS_CHROOT",
            "CAP_SYS_PTRACE",
            "CAP_SYS_PACCT",
            "CAP_SYS_ADMIN",
            "CAP_SYS_BOOT",
            "CAP_SYS_NICE",
            "CAP_SYS_RESOURCE",
            "CAP_SYS_TIME",
            "CAP_SYS_TTY_CONFIG",
            "CAP_MKNOD",
            "CAP_LEASE",
            "CAP_AUDIT_WRITE",
            "CAP_AUDIT_CONTROL",
            "CAP_SETFCAP",
            "CAP_MAC_OVERRIDE",
            "CAP_MAC_ADMIN",
            "CAP_SYSLOG",
            "CAP_WAKE_ALARM",
            "CAP_BLOCK_SUSPEND",
            "CAP_AUDIT_READ",
        ];

        valid_capabilities.contains(&capability)
    }

    /// 检查权限
    async fn check_permission_internal(&self, sandbox_id: &SandboxId, action: &str) -> SecurityResult<bool> {
        let policies = self.policies.read().await;
        let policy = policies.get(sandbox_id);

        if let Some(policy) = policy {
            match action {
                "network_access" => Ok(policy.allow_network_access),
                "file_system_access" => Ok(policy.allow_file_system_access),
                "process_creation" => Ok(policy.allow_process_creation),
                _ => {
                    // 对于其他动作，检查是否在允许的系统调用列表中
                    if policy.allow_system_calls.is_empty() {
                        // 如果没有明确的允许列表，检查是否在阻止列表中
                        Ok(!policy.blocked_system_calls.contains(&action.to_string()))
                    } else {
                        // 如果有允许列表，检查是否在其中
                        Ok(policy.allow_system_calls.contains(&action.to_string()))
                    }
                }
            }
        } else {
            // 如果没有找到策略，默认拒绝
            Ok(false)
        }
    }

    /// 创建安全违规记录
    fn create_violation(&self, sandbox_id: &SandboxId, violation_type: ViolationType, description: String, severity: Severity) -> SecurityViolation {
        SecurityViolation {
            sandbox_id: sandbox_id.clone(),
            violation_type,
            description,
            timestamp: Utc::now(),
            severity,
            details: HashMap::new(),
        }
    }

    /// 清理过期的违规记录
    async fn cleanup_expired_violations(&self) -> SecurityResult<()> {
        let retention_duration = chrono::Duration::days(self.config.violation_retention_days as i64);
        let cutoff_time = Utc::now() - retention_duration;

        let mut violations = self.violations.write().await;
        for (_, sandbox_violations) in violations.iter_mut() {
            sandbox_violations.retain(|v| v.timestamp > cutoff_time);
        }

        // 移除空的违规列表
        violations.retain(|_, v| !v.is_empty());

        Ok(())
    }

    /// 检查违规数量限制
    async fn check_violation_limits(&self, sandbox_id: &SandboxId) -> SecurityResult<()> {
        let violations = self.violations.read().await;
        if let Some(sandbox_violations) = violations.get(sandbox_id) {
            if sandbox_violations.len() >= self.config.max_violations_per_sandbox {
                return Err(SecurityError::AccessDenied(
                    format!("Sandbox {} has exceeded maximum violations limit", sandbox_id.as_str())
                ));
            }
        }
        Ok(())
    }

    /// 发送关键违规警报
    async fn send_critical_alert(&self, violation: &SecurityViolation) -> SecurityResult<()> {
        if self.config.alert_on_critical_violations && violation.severity == Severity::Critical {
            // 在实际实现中，这里会发送警报到监控系统
            error!("CRITICAL SECURITY VIOLATION: {:?}", violation);
        }
        Ok(())
    }
}

#[async_trait]
impl SecurityManager for SecurityManagerImpl {
    async fn validate_policy(&self, policy: &SecurityPolicy) -> SecurityResult<()> {
        debug!("Validating security policy");
        self.validate_policy_internal(policy)
    }

    async fn apply_policy(&self, sandbox_id: &SandboxId, policy: &SecurityPolicy) -> SecurityResult<()> {
        info!("Applying security policy for sandbox: {}", sandbox_id.as_str());
        
        // 验证策略
        self.validate_policy_internal(policy)?;

        // 存储策略
        let mut policies = self.policies.write().await;
        policies.insert(sandbox_id.clone(), policy.clone());

        info!("Applied security policy for sandbox: {}", sandbox_id.as_str());
        Ok(())
    }

    async fn monitor_policy(&self, sandbox_id: &SandboxId) -> SecurityResult<PolicyStatus> {
        debug!("Monitoring policy for sandbox: {}", sandbox_id.as_str());
        
        let violations = self.violations.read().await;
        let policies = self.policies.read().await;
        
        let active = policies.contains_key(sandbox_id);
        let sandbox_violations = violations.get(sandbox_id);
        
        let violation_count = sandbox_violations.map(|v| v.len()).unwrap_or(0) as u64;
        let last_violation = sandbox_violations
            .and_then(|v| v.last())
            .map(|v| v.timestamp);

        Ok(PolicyStatus {
            active,
            violations: violation_count,
            last_violation,
        })
    }

    async fn record_violation(&self, violation: SecurityViolation) -> SecurityResult<()> {
        info!("Recording security violation for sandbox: {}", violation.sandbox_id.as_str());
        
        // 检查违规数量限制
        self.check_violation_limits(&violation.sandbox_id).await?;

        // 发送关键违规警报
        self.send_critical_alert(&violation).await?;

        // 记录违规
        let mut violations = self.violations.write().await;
        violations.entry(violation.sandbox_id.clone())
            .or_insert_with(Vec::new)
            .push(violation.clone());

        // 如果启用了审计日志，记录到数据库
        if self.config.enable_audit_logging {
            // 在实际实现中，这里会将违规记录保存到数据库
            debug!("Violation recorded to audit log");
        }

        info!("Recorded security violation for sandbox: {}", violation.sandbox_id.as_str());
        Ok(())
    }

    async fn get_violations(&self, sandbox_id: &SandboxId) -> SecurityResult<Vec<SecurityViolation>> {
        debug!("Getting violations for sandbox: {}", sandbox_id.as_str());
        
        let violations = self.violations.read().await;
        let sandbox_violations = violations.get(sandbox_id)
            .cloned()
            .unwrap_or_default();

        Ok(sandbox_violations)
    }

    async fn audit_sandbox_creation(&self, sandbox_id: &SandboxId, config: &SandboxConfig) -> SecurityResult<()> {
        info!("Auditing sandbox creation: {}", sandbox_id.as_str());
        
        if !self.config.enable_audit_logging {
            return Ok(());
        }

        // 检查配置中的潜在安全问题
        if config.security_policy.allow_network_access && config.security_policy.allow_process_creation {
            warn!("Sandbox {} allows both network access and process creation", sandbox_id.as_str());
        }

        if config.security_policy.capabilities.contains(&"CAP_SYS_ADMIN".to_string()) {
            warn!("Sandbox {} has CAP_SYS_ADMIN capability", sandbox_id.as_str());
        }

        // 在实际实现中，这里会将审计记录保存到数据库
        debug!("Sandbox creation audited");
        Ok(())
    }

    async fn audit_execution(&self, sandbox_id: &SandboxId, command: &Command) -> SecurityResult<()> {
        debug!("Auditing command execution for sandbox: {}", sandbox_id.as_str());
        
        if !self.config.enable_audit_logging {
            return Ok(());
        }

        // 检查命令中的潜在安全问题
        let dangerous_commands = vec![
            "rm", "rmdir", "dd", "mkfs", "fdisk", "mount", "umount",
            "chmod", "chown", "su", "sudo", "passwd", "useradd", "userdel",
            "iptables", "netstat", "ss", "lsof", "ps", "kill", "killall",
        ];

        if dangerous_commands.contains(&command.program.as_str()) {
            warn!("Potentially dangerous command executed in sandbox {}: {}", 
                  sandbox_id.as_str(), command.program);
        }

        // 检查命令参数中的可疑模式
        for arg in &command.args {
            if arg.contains("..") || arg.starts_with('/') {
                warn!("Suspicious argument in sandbox {}: {}", sandbox_id.as_str(), arg);
            }
        }

        // 在实际实现中，这里会将审计记录保存到数据库
        debug!("Command execution audited");
        Ok(())
    }

    async fn check_permission(&self, sandbox_id: &SandboxId, action: &str) -> SecurityResult<bool> {
        self.check_permission_internal(sandbox_id, action).await
    }
}

/// 安全审计器
pub struct SecurityAuditor {
    db: Arc<SqliteDatabase>,
    config: SecurityManagerConfig,
}

impl SecurityAuditor {
    pub fn new(db: Arc<SqliteDatabase>, config: SecurityManagerConfig) -> Self {
        Self { db, config }
    }

    /// 审计沙箱创建
    pub async fn audit_sandbox_creation(&self, sandbox_id: &SandboxId, config: &SandboxConfig) -> SecurityResult<()> {
        info!("Auditing sandbox creation: {}", sandbox_id.as_str());
        
        // 检查隔离类型
        match config.isolation_type {
            IsolationType::None => {
                warn!("Sandbox {} has no isolation", sandbox_id.as_str());
            }
            IsolationType::Container => {
                info!("Sandbox {} uses container isolation", sandbox_id.as_str());
            }
            _ => {
                info!("Sandbox {} uses {:?} isolation", sandbox_id.as_str(), config.isolation_type);
            }
        }

        // 检查网络配置
        if config.network_config.network_mode == NetworkMode::Host {
            warn!("Sandbox {} uses host networking", sandbox_id.as_str());
        }

        // 检查卷挂载
        for volume in &config.storage_config.volumes {
            if !volume.read_only {
                warn!("Sandbox {} has writable volume mount: {}", 
                      sandbox_id.as_str(), volume.host_path);
            }
        }

        Ok(())
    }

    /// 审计命令执行
    pub async fn audit_execution(&self, sandbox_id: &SandboxId, command: &Command) -> SecurityResult<()> {
        debug!("Auditing command execution for sandbox: {}", sandbox_id.as_str());
        
        // 记录命令执行
        info!("Sandbox {} executing: {} {:?}", 
              sandbox_id.as_str(), command.program, command.args);

        // 检查环境变量
        for (key, value) in &command.environment {
            if key.contains("PASSWORD") || key.contains("SECRET") || key.contains("TOKEN") {
                warn!("Sensitive environment variable in sandbox {}: {}", 
                      sandbox_id.as_str(), key);
            }
        }

        Ok(())
    }
}

/// 漏洞扫描器实现
pub struct VulnerabilityScannerImpl {
    db: Arc<SqliteDatabase>,
    vulnerability_db: Arc<tokio::sync::RwLock<HashMap<String, Vec<Vulnerability>>>>,
}

impl VulnerabilityScannerImpl {
    pub fn new(db: Arc<SqliteDatabase>) -> Self {
        Self {
            db,
            vulnerability_db: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    /// 扫描容器镜像
    async fn scan_image_internal(&self, image: &str) -> ScanResult<Vec<Vulnerability>> {
        info!("Scanning image for vulnerabilities: {}", image);
        
        // 在实际实现中，这里会使用真实的漏洞扫描工具
        // 例如 Trivy, Clair, 或者其他安全扫描工具
        
        let vulnerability_db = self.vulnerability_db.read().await;
        let vulnerabilities = vulnerability_db.get(image)
            .cloned()
            .unwrap_or_default();

        info!("Found {} vulnerabilities in image {}", vulnerabilities.len(), image);
        Ok(vulnerabilities)
    }

    /// 模拟漏洞数据库更新
    async fn update_database_internal(&self) -> ScanResult<()> {
        info!("Updating vulnerability database");
        
        // 在实际实现中，这里会从官方漏洞数据库下载最新数据
        // 例如 CVE 数据库、NVD 等
        
        let mut vulnerability_db = self.vulnerability_db.write().await;
        
        // 添加一些示例漏洞数据
        let sample_vulnerabilities = vec![
            Vulnerability {
                id: "CVE-2023-1234".to_string(),
                severity: Severity::High,
                description: "Buffer overflow in example library".to_string(),
                affected_component: "example-lib".to_string(),
                fix_available: true,
                fix_version: Some("1.2.3".to_string()),
            },
            Vulnerability {
                id: "CVE-2023-5678".to_string(),
                severity: Severity::Medium,
                description: "Information disclosure vulnerability".to_string(),
                affected_component: "another-lib".to_string(),
                fix_available: false,
                fix_version: None,
            },
        ];

        vulnerability_db.insert("alpine:latest".to_string(), sample_vulnerabilities);
        
        info!("Vulnerability database updated");
        Ok(())
    }
}

#[async_trait]
impl crate::sandbox::VulnerabilityScanner for VulnerabilityScannerImpl {
    async fn scan_container(&self, container_id: &ContainerId) -> ScanResult<Vec<Vulnerability>> {
        info!("Scanning container for vulnerabilities: {}", container_id.as_str());
        
        // 在实际实现中，这里会扫描运行中的容器
        // 这里简化为扫描默认镜像
        self.scan_image_internal("alpine:latest").await
    }

    async fn scan_image(&self, image: &str) -> ScanResult<Vec<Vulnerability>> {
        self.scan_image_internal(image).await
    }

    async fn get_scan_history(&self, _container_id: &ContainerId) -> ScanResult<Vec<ScanResult<Vec<Vulnerability>>>> {
        // 在实际实现中，这里会从数据库查询扫描历史
        Ok(vec![])
    }

    async fn update_vulnerability_database(&self) -> ScanResult<()> {
        self.update_database_internal().await
    }
} 