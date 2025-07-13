use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;
use stepflow_database::SqliteDatabase;
use tracing::{debug, error, info, warn};

use crate::errors::*;
use crate::sandbox::SandboxMonitoring;
use crate::types::*;

/// 监控配置
#[derive(Debug, Clone)]
pub struct MonitoringConfig {
    pub enable_metrics_collection: bool,
    pub enable_real_time_monitoring: bool,
    pub metrics_collection_interval: Duration,
    pub metrics_retention_days: u32,
    pub alert_thresholds: AlertThresholds,
    pub enable_performance_monitoring: bool,
    pub enable_security_monitoring: bool,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enable_metrics_collection: true,
            enable_real_time_monitoring: true,
            metrics_collection_interval: Duration::from_secs(30),
            metrics_retention_days: 30,
            alert_thresholds: AlertThresholds::default(),
            enable_performance_monitoring: true,
            enable_security_monitoring: true,
        }
    }
}

/// 警报阈值配置
#[derive(Debug, Clone)]
pub struct AlertThresholds {
    pub memory_usage_threshold: f64,
    pub cpu_usage_threshold: f64,
    pub disk_usage_threshold: f64,
    pub network_usage_threshold: f64,
    pub security_violation_threshold: u64,
    pub execution_time_threshold: Duration,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            memory_usage_threshold: 0.8, // 80%
            cpu_usage_threshold: 0.8,    // 80%
            disk_usage_threshold: 0.9,   // 90%
            network_usage_threshold: 0.8, // 80%
            security_violation_threshold: 10,
            execution_time_threshold: Duration::from_secs(300), // 5 minutes
        }
    }
}

/// 沙箱监控实现
pub struct SandboxMonitoringImpl {
    db: Arc<SqliteDatabase>,
    metrics_collector: Arc<SimpleMetricsCollector>,
    config: MonitoringConfig,
    sandbox_metrics: Arc<tokio::sync::RwLock<HashMap<SandboxId, SandboxMetrics>>>,
    system_metrics: Arc<tokio::sync::RwLock<SandboxMetrics>>,
}

impl SandboxMonitoringImpl {
    pub fn new(
        db: Arc<SqliteDatabase>,
        metrics_collector: Arc<SimpleMetricsCollector>,
        config: MonitoringConfig,
    ) -> Self {
        Self {
            db,
            metrics_collector,
            config,
            sandbox_metrics: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            system_metrics: Arc::new(tokio::sync::RwLock::new(SandboxMetrics::default())),
        }
    }

    /// 启动监控服务
    pub async fn start_monitoring(&self) -> MonitoringResult<()> {
        if !self.config.enable_real_time_monitoring {
            return Ok(());
        }

        info!("Starting sandbox monitoring service");
        
        // 启动指标收集任务
        let metrics_collector = self.metrics_collector.clone();
        let config = self.config.clone();
        let sandbox_metrics = self.sandbox_metrics.clone();
        let system_metrics = self.system_metrics.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(config.metrics_collection_interval);
            loop {
                interval.tick().await;
                
                // 收集系统指标
                if let Err(e) = Self::collect_system_metrics(&system_metrics).await {
                    error!("Failed to collect system metrics: {}", e);
                }
                
                // 收集沙箱指标
                if let Err(e) = Self::collect_sandbox_metrics(&sandbox_metrics).await {
                    error!("Failed to collect sandbox metrics: {}", e);
                }
            }
        });

        info!("Sandbox monitoring service started");
        Ok(())
    }

    /// 收集系统指标
    async fn collect_system_metrics(
        system_metrics: &Arc<tokio::sync::RwLock<SandboxMetrics>>,
    ) -> MonitoringResult<()> {
        // 在实际实现中，这里会收集真实的系统指标
        // 例如使用 sysinfo crate 或直接读取 /proc 文件系统
        
        let mut metrics = system_metrics.write().await;
        
        // 模拟系统指标收集
        metrics.memory_usage = Self::get_system_memory_usage().await?;
        metrics.cpu_usage = Self::get_system_cpu_usage().await?;
        
        Ok(())
    }

    /// 收集沙箱指标
    async fn collect_sandbox_metrics(
        sandbox_metrics: &Arc<tokio::sync::RwLock<HashMap<SandboxId, SandboxMetrics>>>,
    ) -> MonitoringResult<()> {
        let mut metrics = sandbox_metrics.write().await;
        
        // 为每个活跃的沙箱收集指标
        for (sandbox_id, sandbox_metrics) in metrics.iter_mut() {
            // 在实际实现中，这里会收集每个沙箱的真实指标
            // 例如通过 cgroups 或容器运行时 API
            
            sandbox_metrics.memory_usage = Self::get_sandbox_memory_usage(sandbox_id).await?;
            sandbox_metrics.cpu_usage = Self::get_sandbox_cpu_usage(sandbox_id).await?;
        }
        
        Ok(())
    }

    /// 获取系统内存使用情况
    async fn get_system_memory_usage() -> MonitoringResult<usize> {
        // 在实际实现中，这里会读取真实的内存使用情况
        // 例如从 /proc/meminfo 或使用 sysinfo crate
        Ok(1024 * 1024 * 1024) // 1GB 示例值
    }

    /// 获取系统 CPU 使用情况
    async fn get_system_cpu_usage() -> MonitoringResult<f64> {
        // 在实际实现中，这里会读取真实的 CPU 使用情况
        // 例如从 /proc/stat 或使用 sysinfo crate
        Ok(0.5) // 50% 示例值
    }

    /// 获取沙箱内存使用情况
    async fn get_sandbox_memory_usage(_sandbox_id: &SandboxId) -> MonitoringResult<usize> {
        // 在实际实现中，这里会读取沙箱的内存使用情况
        // 例如从 cgroups 或容器运行时 API
        Ok(256 * 1024 * 1024) // 256MB 示例值
    }

    /// 获取沙箱 CPU 使用情况
    async fn get_sandbox_cpu_usage(_sandbox_id: &SandboxId) -> MonitoringResult<f64> {
        // 在实际实现中，这里会读取沙箱的 CPU 使用情况
        // 例如从 cgroups 或容器运行时 API
        Ok(0.2) // 20% 示例值
    }

    /// 检查警报阈值
    async fn check_alert_thresholds(&self, sandbox_id: &SandboxId, metrics: &SandboxMetrics) -> MonitoringResult<()> {
        let thresholds = &self.config.alert_thresholds;
        
        // 检查内存使用率
        let memory_usage_ratio = metrics.memory_usage as f64 / (1024.0 * 1024.0 * 1024.0); // 假设 1GB 限制
        if memory_usage_ratio > thresholds.memory_usage_threshold {
            warn!("Memory usage alert for sandbox {}: {:.2}%", 
                  sandbox_id.as_str(), memory_usage_ratio * 100.0);
        }

        // 检查 CPU 使用率
        if metrics.cpu_usage > thresholds.cpu_usage_threshold {
            warn!("CPU usage alert for sandbox {}: {:.2}%", 
                  sandbox_id.as_str(), metrics.cpu_usage * 100.0);
        }

        // 检查安全违规数量
        if metrics.security_violations > thresholds.security_violation_threshold {
            warn!("Security violations alert for sandbox {}: {}", 
                  sandbox_id.as_str(), metrics.security_violations);
        }

        Ok(())
    }

    /// 清理过期的指标
    async fn cleanup_expired_metrics(&self) -> MonitoringResult<()> {
        // 在实际实现中，这里会清理数据库中的过期指标
        info!("Cleaning up expired metrics");
        Ok(())
    }

    /// 创建指标记录
    fn create_metric(&self, name: &str, value: f64, unit: &str, labels: HashMap<String, String>) -> Metric {
        Metric {
            name: name.to_string(),
            value,
            unit: unit.to_string(),
            timestamp: Utc::now(),
            labels,
        }
    }
}

#[async_trait]
impl SandboxMonitoring for SandboxMonitoringImpl {
    async fn record_sandbox_creation(&self, sandbox_id: &SandboxId) -> MonitoringResult<()> {
        info!("Recording sandbox creation: {}", sandbox_id.as_str());
        
        // 初始化沙箱指标
        let mut sandbox_metrics = self.sandbox_metrics.write().await;
        sandbox_metrics.insert(sandbox_id.clone(), SandboxMetrics::default());
        
        // 更新系统指标
        let mut system_metrics = self.system_metrics.write().await;
        system_metrics.total_sandboxes += 1;
        system_metrics.active_sandboxes += 1;
        
        // 记录指标
        if self.config.enable_metrics_collection {
            let metric = self.create_metric(
                "sandbox_created",
                1.0,
                "count",
                HashMap::from([("sandbox_id".to_string(), sandbox_id.as_str().to_string())]),
            );
            self.record_metric(metric).await?;
        }
        
        Ok(())
    }

    async fn record_sandbox_destruction(&self, sandbox_id: &SandboxId) -> MonitoringResult<()> {
        info!("Recording sandbox destruction: {}", sandbox_id.as_str());
        
        // 移除沙箱指标
        let mut sandbox_metrics = self.sandbox_metrics.write().await;
        sandbox_metrics.remove(sandbox_id);
        
        // 更新系统指标
        let mut system_metrics = self.system_metrics.write().await;
        system_metrics.active_sandboxes = system_metrics.active_sandboxes.saturating_sub(1);
        
        // 记录指标
        if self.config.enable_metrics_collection {
            let metric = self.create_metric(
                "sandbox_destroyed",
                1.0,
                "count",
                HashMap::from([("sandbox_id".to_string(), sandbox_id.as_str().to_string())]),
            );
            self.record_metric(metric).await?;
        }
        
        Ok(())
    }

    async fn record_execution(&self, sandbox_id: &SandboxId, execution: &ExecutionResult) -> MonitoringResult<()> {
        debug!("Recording execution for sandbox: {}", sandbox_id.as_str());
        
        // 更新沙箱指标
        let mut sandbox_metrics = self.sandbox_metrics.write().await;
        if let Some(metrics) = sandbox_metrics.get_mut(sandbox_id) {
            metrics.total_executions += 1;
            metrics.average_execution_time = Duration::from_secs(
                (metrics.average_execution_time.as_secs() + execution.execution_time.as_secs()) / 2
            );
        }
        
        // 更新系统指标
        let mut system_metrics = self.system_metrics.write().await;
        system_metrics.total_executions += 1;
        system_metrics.average_execution_time = Duration::from_secs(
            (system_metrics.average_execution_time.as_secs() + execution.execution_time.as_secs()) / 2
        );
        
        // 记录指标
        if self.config.enable_metrics_collection {
            let metric = self.create_metric(
                "execution_time",
                execution.execution_time.as_secs_f64(),
                "seconds",
                HashMap::from([
                    ("sandbox_id".to_string(), sandbox_id.as_str().to_string()),
                    ("exit_code".to_string(), execution.exit_code.to_string()),
                ]),
            );
            self.record_metric(metric).await?;
        }
        
        Ok(())
    }

    async fn record_security_violation(&self, sandbox_id: &SandboxId, violation: &SecurityViolation) -> MonitoringResult<()> {
        warn!("Recording security violation for sandbox: {}", sandbox_id.as_str());
        
        // 更新沙箱指标
        let mut sandbox_metrics = self.sandbox_metrics.write().await;
        if let Some(metrics) = sandbox_metrics.get_mut(sandbox_id) {
            metrics.security_violations += 1;
        }
        
        // 更新系统指标
        let mut system_metrics = self.system_metrics.write().await;
        system_metrics.security_violations += 1;
        
        // 记录指标
        if self.config.enable_metrics_collection {
            let metric = self.create_metric(
                "security_violation",
                1.0,
                "count",
                HashMap::from([
                    ("sandbox_id".to_string(), sandbox_id.as_str().to_string()),
                    ("violation_type".to_string(), format!("{:?}", violation.violation_type)),
                    ("severity".to_string(), format!("{:?}", violation.severity)),
                ]),
            );
            self.record_metric(metric).await?;
        }
        
        Ok(())
    }

    async fn get_metrics(&self, filter: Option<MetricFilter>) -> MonitoringResult<Vec<Metric>> {
        debug!("Getting metrics with filter: {:?}", filter);
        
        // 在实际实现中，这里会从数据库查询指标
        // 这里提供一个简化的实现
        
        let mut metrics = Vec::new();
        
        // 添加系统指标
        let system_metrics = self.system_metrics.read().await;
        metrics.push(self.create_metric(
            "total_sandboxes",
            system_metrics.total_sandboxes as f64,
            "count",
            HashMap::new(),
        ));
        metrics.push(self.create_metric(
            "active_sandboxes",
            system_metrics.active_sandboxes as f64,
            "count",
            HashMap::new(),
        ));
        
        // 应用过滤器
        if let Some(filter) = filter {
            if let Some(name) = &filter.name {
                metrics.retain(|m| m.name == *name);
            }
            
            if let Some(start_time) = filter.start_time {
                metrics.retain(|m| m.timestamp >= start_time);
            }
            
            if let Some(end_time) = filter.end_time {
                metrics.retain(|m| m.timestamp <= end_time);
            }
            
            if !filter.labels.is_empty() {
                metrics.retain(|m| {
                    filter.labels.iter().all(|(k, v)| {
                        m.labels.get(k).map_or(false, |label_value| label_value == v)
                    })
                });
            }
        }
        
        Ok(metrics)
    }

    async fn get_sandbox_metrics(&self, sandbox_id: &SandboxId) -> MonitoringResult<SandboxMetrics> {
        debug!("Getting metrics for sandbox: {}", sandbox_id.as_str());
        
        let sandbox_metrics = self.sandbox_metrics.read().await;
        let metrics = sandbox_metrics.get(sandbox_id)
            .cloned()
            .unwrap_or_default();
        
        Ok(metrics)
    }

    async fn record_metric(&self, metric: Metric) -> MonitoringResult<()> {
        debug!("Recording metric: {} = {}", metric.name, metric.value);
        
        // 在实际实现中，这里会将指标保存到数据库
        // 这里使用 stepflow-monitoring 的 MetricsCollector
        
        self.metrics_collector.record_metric(&metric.name, metric.value, &metric.labels)
            .await
            .map_err(|e| MonitoringError::MetricStorageFailed(e.to_string()))?;
        
        Ok(())
    }

    async fn get_system_metrics(&self) -> MonitoringResult<SandboxMetrics> {
        debug!("Getting system metrics");
        
        let system_metrics = self.system_metrics.read().await;
        Ok(system_metrics.clone())
    }
}

/// 简化的指标收集器
#[derive(Debug, Clone)]
pub struct SimpleMetricsCollector {
    metrics: Arc<tokio::sync::RwLock<HashMap<String, f64>>>,
}

impl SimpleMetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    pub async fn record_metric(&self, name: &str, value: f64, _labels: &HashMap<String, String>) -> Result<(), String> {
        let mut metrics = self.metrics.write().await;
        metrics.insert(name.to_string(), value);
        Ok(())
    }

    pub async fn get_metric(&self, name: &str) -> Option<f64> {
        let metrics = self.metrics.read().await;
        metrics.get(name).copied()
    }

    pub async fn get_all_metrics(&self) -> HashMap<String, f64> {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }
}

/// 性能监控器
pub struct PerformanceMonitor {
    db: Arc<SqliteDatabase>,
    config: MonitoringConfig,
    performance_data: Arc<tokio::sync::RwLock<HashMap<SandboxId, PerformanceData>>>,
}

impl PerformanceMonitor {
    pub fn new(db: Arc<SqliteDatabase>, config: MonitoringConfig) -> Self {
        Self {
            db,
            config,
            performance_data: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    /// 开始性能监控
    pub async fn start_performance_monitoring(&self, sandbox_id: &SandboxId) -> MonitoringResult<()> {
        if !self.config.enable_performance_monitoring {
            return Ok(());
        }

        info!("Starting performance monitoring for sandbox: {}", sandbox_id.as_str());
        
        let mut performance_data = self.performance_data.write().await;
        performance_data.insert(sandbox_id.clone(), PerformanceData::new());
        
        Ok(())
    }

    /// 停止性能监控
    pub async fn stop_performance_monitoring(&self, sandbox_id: &SandboxId) -> MonitoringResult<()> {
        info!("Stopping performance monitoring for sandbox: {}", sandbox_id.as_str());
        
        let mut performance_data = self.performance_data.write().await;
        performance_data.remove(sandbox_id);
        
        Ok(())
    }

    /// 记录性能数据
    pub async fn record_performance_data(&self, sandbox_id: &SandboxId, data: PerformanceData) -> MonitoringResult<()> {
        let mut performance_data = self.performance_data.write().await;
        performance_data.insert(sandbox_id.clone(), data);
        
        Ok(())
    }

    /// 获取性能数据
    pub async fn get_performance_data(&self, sandbox_id: &SandboxId) -> MonitoringResult<Option<PerformanceData>> {
        let performance_data = self.performance_data.read().await;
        Ok(performance_data.get(sandbox_id).cloned())
    }
}

/// 性能数据
#[derive(Debug, Clone)]
pub struct PerformanceData {
    pub cpu_usage_history: Vec<f64>,
    pub memory_usage_history: Vec<usize>,
    pub disk_io_history: Vec<DiskIoData>,
    pub network_io_history: Vec<NetworkIoData>,
    pub start_time: chrono::DateTime<Utc>,
    pub last_updated: chrono::DateTime<Utc>,
}

impl PerformanceData {
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            cpu_usage_history: Vec::new(),
            memory_usage_history: Vec::new(),
            disk_io_history: Vec::new(),
            network_io_history: Vec::new(),
            start_time: now,
            last_updated: now,
        }
    }
}

/// 磁盘 I/O 数据
#[derive(Debug, Clone)]
pub struct DiskIoData {
    pub read_bytes: u64,
    pub write_bytes: u64,
    pub read_ops: u64,
    pub write_ops: u64,
    pub timestamp: chrono::DateTime<Utc>,
}

/// 网络 I/O 数据
#[derive(Debug, Clone)]
pub struct NetworkIoData {
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_packets: u64,
    pub tx_packets: u64,
    pub timestamp: chrono::DateTime<Utc>,
}

/// 实时监控器
pub struct RealTimeMonitor {
    db: Arc<SqliteDatabase>,
    config: MonitoringConfig,
    monitoring_tasks: Arc<tokio::sync::RwLock<HashMap<SandboxId, tokio::task::JoinHandle<()>>>>,
}

impl RealTimeMonitor {
    pub fn new(db: Arc<SqliteDatabase>, config: MonitoringConfig) -> Self {
        Self {
            db,
            config,
            monitoring_tasks: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    /// 开始实时监控
    pub async fn start_real_time_monitoring(&self, sandbox_id: &SandboxId) -> MonitoringResult<()> {
        if !self.config.enable_real_time_monitoring {
            return Ok(());
        }

        info!("Starting real-time monitoring for sandbox: {}", sandbox_id.as_str());
        
        let sandbox_id_clone = sandbox_id.clone();
        let config = self.config.clone();
        
        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(config.metrics_collection_interval);
            loop {
                interval.tick().await;
                
                // 收集实时指标
                if let Err(e) = Self::collect_real_time_metrics(&sandbox_id_clone).await {
                    error!("Failed to collect real-time metrics for sandbox {}: {}", 
                           sandbox_id_clone.as_str(), e);
                }
            }
        });
        
        let mut monitoring_tasks = self.monitoring_tasks.write().await;
        monitoring_tasks.insert(sandbox_id.clone(), handle);
        
        Ok(())
    }

    /// 停止实时监控
    pub async fn stop_real_time_monitoring(&self, sandbox_id: &SandboxId) -> MonitoringResult<()> {
        info!("Stopping real-time monitoring for sandbox: {}", sandbox_id.as_str());
        
        let mut monitoring_tasks = self.monitoring_tasks.write().await;
        if let Some(handle) = monitoring_tasks.remove(sandbox_id) {
            handle.abort();
        }
        
        Ok(())
    }

    /// 收集实时指标
    async fn collect_real_time_metrics(sandbox_id: &SandboxId) -> MonitoringResult<()> {
        debug!("Collecting real-time metrics for sandbox: {}", sandbox_id.as_str());
        
        // 在实际实现中，这里会收集真实的实时指标
        // 例如通过 cgroups、容器运行时 API 或系统调用
        
        Ok(())
    }
} 