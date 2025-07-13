use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use stepflow_database::SqliteDatabase;
use tracing::{debug, error, info, warn};

use crate::errors::*;
use crate::types::*;

/// 资源限制管理器配置
#[derive(Debug, Clone)]
pub struct ResourceLimitsConfig {
    pub enable_memory_limits: bool,
    pub enable_cpu_limits: bool,
    pub enable_disk_limits: bool,
    pub enable_network_limits: bool,
    pub enable_process_limits: bool,
    pub enable_file_descriptor_limits: bool,
    pub default_memory_limit: usize,
    pub default_cpu_limit: f64,
    pub default_disk_limit: usize,
    pub default_network_bandwidth: usize,
    pub default_process_limit: usize,
    pub default_file_descriptor_limit: usize,
    pub default_execution_timeout: Duration,
}

impl Default for ResourceLimitsConfig {
    fn default() -> Self {
        Self {
            enable_memory_limits: true,
            enable_cpu_limits: true,
            enable_disk_limits: true,
            enable_network_limits: true,
            enable_process_limits: true,
            enable_file_descriptor_limits: true,
            default_memory_limit: 512 * 1024 * 1024, // 512MB
            default_cpu_limit: 1.0, // 1 CPU core
            default_disk_limit: 1024 * 1024 * 1024, // 1GB
            default_network_bandwidth: 100 * 1024 * 1024, // 100MB/s
            default_process_limit: 100,
            default_file_descriptor_limit: 1024,
            default_execution_timeout: Duration::from_secs(300), // 5 minutes
        }
    }
}

/// 资源限制管理器
pub struct ResourceLimitsManager {
    db: Arc<SqliteDatabase>,
    config: ResourceLimitsConfig,
    active_limits: Arc<tokio::sync::RwLock<HashMap<SandboxId, ResourceLimits>>>,
    resource_usage: Arc<tokio::sync::RwLock<HashMap<SandboxId, ResourceUsage>>>,
}

impl ResourceLimitsManager {
    pub fn new(db: Arc<SqliteDatabase>, config: ResourceLimitsConfig) -> Self {
        Self {
            db,
            config,
            active_limits: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            resource_usage: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    /// 应用资源限制
    pub async fn apply_resource_limits(&self, sandbox_id: &SandboxId, limits: ResourceLimits) -> Result<(), SandboxError> {
        info!("Applying resource limits for sandbox: {}", sandbox_id.as_str());
        
        // 验证资源限制
        self.validate_resource_limits(&limits)?;
        
        // 应用内存限制
        if self.config.enable_memory_limits {
            if let Some(memory_limit) = limits.memory_limit {
                self.apply_memory_limit(sandbox_id, memory_limit).await?;
            }
        }
        
        // 应用 CPU 限制
        if self.config.enable_cpu_limits {
            if let Some(cpu_limit) = limits.cpu_limit {
                self.apply_cpu_limit(sandbox_id, cpu_limit).await?;
            }
        }
        
        // 应用磁盘限制
        if self.config.enable_disk_limits {
            if let Some(disk_limit) = limits.disk_limit {
                self.apply_disk_limit(sandbox_id, disk_limit).await?;
            }
        }
        
        // 应用网络限制
        if self.config.enable_network_limits {
            if let Some(network_limit) = limits.network_bandwidth {
                self.apply_network_limit(sandbox_id, network_limit).await?;
            }
        }
        
        // 应用进程限制
        if self.config.enable_process_limits {
            if let Some(process_limit) = limits.process_limit {
                self.apply_process_limit(sandbox_id, process_limit).await?;
            }
        }
        
        // 应用文件描述符限制
        if self.config.enable_file_descriptor_limits {
            if let Some(fd_limit) = limits.file_descriptor_limit {
                self.apply_file_descriptor_limit(sandbox_id, fd_limit).await?;
            }
        }
        
        // 存储活跃的限制
        let mut active_limits = self.active_limits.write().await;
        active_limits.insert(sandbox_id.clone(), limits);
        
        info!("Applied resource limits for sandbox: {}", sandbox_id.as_str());
        Ok(())
    }

    /// 验证资源限制
    fn validate_resource_limits(&self, limits: &ResourceLimits) -> Result<(), SandboxError> {
        // 验证内存限制
        if let Some(memory_limit) = limits.memory_limit {
            if memory_limit == 0 {
                return Err(SandboxError::InternalError("Memory limit cannot be zero".to_string()));
            }
            if memory_limit > 16 * 1024 * 1024 * 1024 { // 16GB
                return Err(SandboxError::InternalError("Memory limit too high".to_string()));
            }
        }
        
        // 验证 CPU 限制
        if let Some(cpu_limit) = limits.cpu_limit {
            if cpu_limit <= 0.0 {
                return Err(SandboxError::InternalError("CPU limit must be positive".to_string()));
            }
            if cpu_limit > 32.0 { // 32 cores
                return Err(SandboxError::InternalError("CPU limit too high".to_string()));
            }
        }
        
        // 验证磁盘限制
        if let Some(disk_limit) = limits.disk_limit {
            if disk_limit == 0 {
                return Err(SandboxError::InternalError("Disk limit cannot be zero".to_string()));
            }
            if disk_limit > 1024 * 1024 * 1024 * 1024 { // 1TB
                return Err(SandboxError::InternalError("Disk limit too high".to_string()));
            }
        }
        
        // 验证进程限制
        if let Some(process_limit) = limits.process_limit {
            if process_limit == 0 {
                return Err(SandboxError::InternalError("Process limit cannot be zero".to_string()));
            }
            if process_limit > 10000 {
                return Err(SandboxError::InternalError("Process limit too high".to_string()));
            }
        }
        
        // 验证文件描述符限制
        if let Some(fd_limit) = limits.file_descriptor_limit {
            if fd_limit == 0 {
                return Err(SandboxError::InternalError("File descriptor limit cannot be zero".to_string()));
            }
            if fd_limit > 1000000 {
                return Err(SandboxError::InternalError("File descriptor limit too high".to_string()));
            }
        }
        
        Ok(())
    }

    /// 应用内存限制
    async fn apply_memory_limit(&self, sandbox_id: &SandboxId, limit: usize) -> Result<(), SandboxError> {
        debug!("Applying memory limit {} bytes for sandbox: {}", limit, sandbox_id.as_str());
        
        // 在实际实现中，这里会使用 cgroups 或其他机制设置内存限制
        // 例如：
        // - 写入 /sys/fs/cgroup/memory/sandbox_id/memory.limit_in_bytes
        // - 或者使用 Docker API 设置容器内存限制
        // - 或者使用 systemd 设置服务内存限制
        
        // 这里提供一个简化的实现
        debug!("Memory limit applied: {} bytes", limit);
        Ok(())
    }

    /// 应用 CPU 限制
    async fn apply_cpu_limit(&self, sandbox_id: &SandboxId, limit: f64) -> Result<(), SandboxError> {
        debug!("Applying CPU limit {} cores for sandbox: {}", limit, sandbox_id.as_str());
        
        // 在实际实现中，这里会使用 cgroups 设置 CPU 限制
        // 例如：
        // - 写入 /sys/fs/cgroup/cpu/sandbox_id/cpu.cfs_quota_us
        // - 写入 /sys/fs/cgroup/cpu/sandbox_id/cpu.cfs_period_us
        // - 或者使用 Docker API 设置容器 CPU 限制
        
        // 这里提供一个简化的实现
        debug!("CPU limit applied: {} cores", limit);
        Ok(())
    }

    /// 应用磁盘限制
    async fn apply_disk_limit(&self, sandbox_id: &SandboxId, limit: usize) -> Result<(), SandboxError> {
        debug!("Applying disk limit {} bytes for sandbox: {}", limit, sandbox_id.as_str());
        
        // 在实际实现中，这里会使用以下方法之一：
        // - 使用 quota 系统设置磁盘配额
        // - 使用 loop device 创建限制大小的文件系统
        // - 使用 Docker volume 限制
        // - 使用 overlayfs 或其他文件系统功能
        
        // 这里提供一个简化的实现
        debug!("Disk limit applied: {} bytes", limit);
        Ok(())
    }

    /// 应用网络限制
    async fn apply_network_limit(&self, sandbox_id: &SandboxId, limit: usize) -> Result<(), SandboxError> {
        debug!("Applying network limit {} bytes/s for sandbox: {}", limit, sandbox_id.as_str());
        
        // 在实际实现中，这里会使用 tc (traffic control) 设置网络限制
        // 例如：
        // - 使用 tc qdisc 设置队列规则
        // - 使用 tc class 设置流量类别
        // - 使用 tc filter 设置过滤规则
        // - 或者使用 iptables 结合 tc 实现更复杂的限制
        
        // 这里提供一个简化的实现
        debug!("Network limit applied: {} bytes/s", limit);
        Ok(())
    }

    /// 应用进程限制
    async fn apply_process_limit(&self, sandbox_id: &SandboxId, limit: usize) -> Result<(), SandboxError> {
        debug!("Applying process limit {} for sandbox: {}", limit, sandbox_id.as_str());
        
        // 在实际实现中，这里会使用以下方法之一：
        // - 使用 cgroups pids 子系统
        // - 使用 setrlimit 系统调用
        // - 使用 systemd 服务限制
        // - 在容器中设置 ulimit
        
        // 这里提供一个简化的实现
        debug!("Process limit applied: {}", limit);
        Ok(())
    }

    /// 应用文件描述符限制
    async fn apply_file_descriptor_limit(&self, sandbox_id: &SandboxId, limit: usize) -> Result<(), SandboxError> {
        debug!("Applying file descriptor limit {} for sandbox: {}", limit, sandbox_id.as_str());
        
        // 在实际实现中，这里会使用 setrlimit 系统调用设置 RLIMIT_NOFILE
        // 或者在容器中设置 ulimit -n
        
        // 这里提供一个简化的实现
        debug!("File descriptor limit applied: {}", limit);
        Ok(())
    }

    /// 获取资源使用情况
    pub async fn get_resource_usage(&self, sandbox_id: &SandboxId) -> Result<ResourceUsage, SandboxError> {
        debug!("Getting resource usage for sandbox: {}", sandbox_id.as_str());
        
        let resource_usage = self.resource_usage.read().await;
        let usage = resource_usage.get(sandbox_id)
            .cloned()
            .unwrap_or_default();
        
        Ok(usage)
    }

    /// 更新资源使用情况
    pub async fn update_resource_usage(&self, sandbox_id: &SandboxId, usage: ResourceUsage) -> Result<(), SandboxError> {
        debug!("Updating resource usage for sandbox: {}", sandbox_id.as_str());
        
        let mut resource_usage = self.resource_usage.write().await;
        resource_usage.insert(sandbox_id.clone(), usage);
        
        Ok(())
    }

    /// 检查资源限制违规
    pub async fn check_resource_violations(&self, sandbox_id: &SandboxId) -> Result<Vec<ResourceViolation>, SandboxError> {
        debug!("Checking resource violations for sandbox: {}", sandbox_id.as_str());
        
        let active_limits = self.active_limits.read().await;
        let resource_usage = self.resource_usage.read().await;
        
        let limits = active_limits.get(sandbox_id);
        let usage = resource_usage.get(sandbox_id);
        
        if let (Some(limits), Some(usage)) = (limits, usage) {
            let mut violations = Vec::new();
            
            // 检查内存限制
            if let Some(memory_limit) = limits.memory_limit {
                if usage.memory_used > memory_limit {
                    violations.push(ResourceViolation {
                        resource_type: ResourceType::Memory,
                        limit: memory_limit as f64,
                        usage: usage.memory_used as f64,
                        timestamp: chrono::Utc::now(),
                    });
                }
            }
            
            // 检查磁盘限制
            if let Some(disk_limit) = limits.disk_limit {
                let disk_usage = usage.disk_read + usage.disk_write;
                if disk_usage > disk_limit {
                    violations.push(ResourceViolation {
                        resource_type: ResourceType::Disk,
                        limit: disk_limit as f64,
                        usage: disk_usage as f64,
                        timestamp: chrono::Utc::now(),
                    });
                }
            }
            
            // 检查网络限制
            if let Some(network_limit) = limits.network_bandwidth {
                let network_usage = usage.network_rx + usage.network_tx;
                if network_usage > network_limit {
                    violations.push(ResourceViolation {
                        resource_type: ResourceType::Network,
                        limit: network_limit as f64,
                        usage: network_usage as f64,
                        timestamp: chrono::Utc::now(),
                    });
                }
            }
            
            Ok(violations)
        } else {
            Ok(Vec::new())
        }
    }

    /// 移除资源限制
    pub async fn remove_resource_limits(&self, sandbox_id: &SandboxId) -> Result<(), SandboxError> {
        info!("Removing resource limits for sandbox: {}", sandbox_id.as_str());
        
        // 移除活跃的限制
        let mut active_limits = self.active_limits.write().await;
        active_limits.remove(sandbox_id);
        
        // 移除资源使用记录
        let mut resource_usage = self.resource_usage.write().await;
        resource_usage.remove(sandbox_id);
        
        // 在实际实现中，这里会清理 cgroups、tc 规则等
        
        info!("Removed resource limits for sandbox: {}", sandbox_id.as_str());
        Ok(())
    }

    /// 获取活跃的资源限制
    pub async fn get_active_limits(&self, sandbox_id: &SandboxId) -> Result<Option<ResourceLimits>, SandboxError> {
        let active_limits = self.active_limits.read().await;
        Ok(active_limits.get(sandbox_id).cloned())
    }

    /// 列出所有活跃的资源限制
    pub async fn list_active_limits(&self) -> Result<HashMap<SandboxId, ResourceLimits>, SandboxError> {
        let active_limits = self.active_limits.read().await;
        Ok(active_limits.clone())
    }

    /// 启动资源监控
    pub async fn start_resource_monitoring(&self) -> Result<(), SandboxError> {
        if !self.config.enable_memory_limits && !self.config.enable_cpu_limits && 
           !self.config.enable_disk_limits && !self.config.enable_network_limits {
            return Ok(());
        }

        info!("Starting resource monitoring");
        
        let active_limits = self.active_limits.clone();
        let resource_usage = self.resource_usage.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                
                // 收集资源使用情况
                let limits = active_limits.read().await;
                for sandbox_id in limits.keys() {
                    if let Ok(usage) = Self::collect_resource_usage(sandbox_id).await {
                        let mut resource_usage = resource_usage.write().await;
                        resource_usage.insert(sandbox_id.clone(), usage);
                    }
                }
            }
        });
        
        info!("Resource monitoring started");
        Ok(())
    }

    /// 收集资源使用情况
    async fn collect_resource_usage(sandbox_id: &SandboxId) -> Result<ResourceUsage, SandboxError> {
        debug!("Collecting resource usage for sandbox: {}", sandbox_id.as_str());
        
        // 在实际实现中，这里会从 cgroups、/proc 文件系统或其他来源收集真实的资源使用情况
        // 这里提供一个简化的实现
        
        Ok(ResourceUsage {
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
        })
    }
}

/// 资源类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceType {
    Memory,
    Cpu,
    Disk,
    Network,
    Process,
    FileDescriptor,
}

/// 资源违规记录
#[derive(Debug, Clone)]
pub struct ResourceViolation {
    pub resource_type: ResourceType,
    pub limit: f64,
    pub usage: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 资源监控器
pub struct ResourceMonitor {
    limits_manager: Arc<ResourceLimitsManager>,
    violations: Arc<tokio::sync::RwLock<HashMap<SandboxId, Vec<ResourceViolation>>>>,
}

impl ResourceMonitor {
    pub fn new(limits_manager: Arc<ResourceLimitsManager>) -> Self {
        Self {
            limits_manager,
            violations: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    /// 启动资源监控
    pub async fn start_monitoring(&self) -> Result<(), SandboxError> {
        info!("Starting resource monitor");
        
        let limits_manager = self.limits_manager.clone();
        let violations = self.violations.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(10));
            loop {
                interval.tick().await;
                
                // 检查所有活跃的沙箱
                if let Ok(active_limits) = limits_manager.list_active_limits().await {
                    for sandbox_id in active_limits.keys() {
                        if let Ok(sandbox_violations) = limits_manager.check_resource_violations(sandbox_id).await {
                            if !sandbox_violations.is_empty() {
                                warn!("Resource violations detected for sandbox {}: {:?}", 
                                      sandbox_id.as_str(), sandbox_violations);
                                
                                let mut violations = violations.write().await;
                                violations.entry(sandbox_id.clone())
                                    .or_insert_with(Vec::new)
                                    .extend(sandbox_violations);
                            }
                        }
                    }
                }
            }
        });
        
        info!("Resource monitor started");
        Ok(())
    }

    /// 获取资源违规记录
    pub async fn get_violations(&self, sandbox_id: &SandboxId) -> Vec<ResourceViolation> {
        let violations = self.violations.read().await;
        violations.get(sandbox_id).cloned().unwrap_or_default()
    }

    /// 清理过期的违规记录
    pub async fn cleanup_expired_violations(&self, retention_duration: Duration) -> Result<(), SandboxError> {
        let cutoff_time = chrono::Utc::now() - chrono::Duration::from_std(retention_duration)
            .map_err(|e| SandboxError::InternalError(e.to_string()))?;
        
        let mut violations = self.violations.write().await;
        for (_, sandbox_violations) in violations.iter_mut() {
            sandbox_violations.retain(|v| v.timestamp > cutoff_time);
        }
        
        // 移除空的违规列表
        violations.retain(|_, v| !v.is_empty());
        
        Ok(())
    }
} 