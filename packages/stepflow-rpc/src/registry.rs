//! Service Registry and Discovery for Single Machine Deployment

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::error::{RpcFrameworkError, RpcResult};

/// 服务信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub service_name: String,
    pub service_id: String,
    pub address: SocketAddr,
    pub version: String,
    pub metadata: HashMap<String, String>,
    pub health_check_endpoint: Option<String>,
    pub registered_at: u64,
    pub last_heartbeat: u64,
}

impl ServiceInfo {
    pub fn new(
        service_name: String,
        service_id: String,
        address: SocketAddr,
        version: String,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            service_name,
            service_id,
            address,
            version,
            metadata: HashMap::new(),
            health_check_endpoint: None,
            registered_at: now,
            last_heartbeat: now,
        }
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn with_health_check(mut self, endpoint: String) -> Self {
        self.health_check_endpoint = Some(endpoint);
        self
    }

    /// 更新心跳时间
    pub fn update_heartbeat(&mut self) {
        self.last_heartbeat = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }

    /// 检查服务是否过期（超过指定时间没有心跳）
    pub fn is_expired(&self, ttl_seconds: u64) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        now - self.last_heartbeat > ttl_seconds
    }
}

/// 服务注册配置
#[derive(Debug, Clone)]
pub struct RegistryConfig {
    pub heartbeat_interval: Duration,
    pub service_ttl: Duration,
    pub cleanup_interval: Duration,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            heartbeat_interval: Duration::from_secs(10),
            service_ttl: Duration::from_secs(30),
            cleanup_interval: Duration::from_secs(60),
        }
    }
}

/// 服务注册中心
pub struct ServiceRegistry {
    config: RegistryConfig,
    services: Arc<DashMap<String, ServiceInfo>>, // service_id -> ServiceInfo
    services_by_name: Arc<DashMap<String, Vec<String>>>, // service_name -> vec[service_id]
    stats: Arc<RwLock<RegistryStats>>,
    _cleanup_task: Option<tokio::task::JoinHandle<()>>,
}

/// 注册中心统计信息
#[derive(Debug, Default, Clone)]
pub struct RegistryStats {
    pub total_services: usize,
    pub active_services: usize,
    pub expired_services: usize,
    pub registrations: u64,
    pub deregistrations: u64,
    pub heartbeats: u64,
    pub lookups: u64,
}

impl ServiceRegistry {
    /// 创建新的服务注册中心
    pub fn new(config: RegistryConfig) -> Self {
        let registry = Self {
            config,
            services: Arc::new(DashMap::new()),
            services_by_name: Arc::new(DashMap::new()),
            stats: Arc::new(RwLock::new(RegistryStats::default())),
            _cleanup_task: None,
        };

        registry
    }

    /// 启动服务注册中心（包括后台清理任务）
    pub fn start(&mut self) {
        let services = self.services.clone();
        let services_by_name = self.services_by_name.clone();
        let stats = self.stats.clone();
        let cleanup_interval = self.config.cleanup_interval;
        let service_ttl = self.config.service_ttl;

        let cleanup_task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(cleanup_interval);
            
            loop {
                interval.tick().await;
                Self::cleanup_expired_services(&services, &services_by_name, &stats, service_ttl.as_secs()).await;
            }
        });

        self._cleanup_task = Some(cleanup_task);
        info!("Service registry started with cleanup interval: {:?}", self.config.cleanup_interval);
    }

    /// 注册服务
    pub async fn register_service(&self, service_info: ServiceInfo) -> RpcResult<()> {
        let service_id = service_info.service_id.clone();
        let service_name = service_info.service_name.clone();

        info!("Registering service: {} ({})", service_name, service_id);

        // 添加到主服务映射
        self.services.insert(service_id.clone(), service_info);

        // 添加到按名称索引
        self.services_by_name
            .entry(service_name.clone())
            .or_insert_with(Vec::new)
            .push(service_id.clone());

        // 更新统计
        {
            let mut stats = self.stats.write().await;
            stats.registrations += 1;
            stats.total_services = self.services.len();
            stats.active_services = self.services.len();
        }

        debug!("Service registered successfully: {}", service_id);
        Ok(())
    }

    /// 注销服务
    pub async fn deregister_service(&self, service_id: &str) -> RpcResult<()> {
        if let Some((_, service_info)) = self.services.remove(service_id) {
            info!("Deregistering service: {} ({})", service_info.service_name, service_id);

            // 从按名称索引中移除
            if let Some(mut service_ids) = self.services_by_name.get_mut(&service_info.service_name) {
                service_ids.retain(|id| id != service_id);
                if service_ids.is_empty() {
                    drop(service_ids);
                    self.services_by_name.remove(&service_info.service_name);
                }
            }

            // 更新统计
            {
                let mut stats = self.stats.write().await;
                stats.deregistrations += 1;
                stats.total_services = self.services.len();
                stats.active_services = self.services.len();
            }

            debug!("Service deregistered successfully: {}", service_id);
            Ok(())
        } else {
            Err(RpcFrameworkError::ServiceNotFound(service_id.to_string()))
        }
    }

    /// 更新服务心跳
    pub async fn heartbeat(&self, service_id: &str) -> RpcResult<()> {
        if let Some(mut service_info) = self.services.get_mut(service_id) {
            service_info.update_heartbeat();
            
            {
                let mut stats = self.stats.write().await;
                stats.heartbeats += 1;
            }

            debug!("Heartbeat updated for service: {}", service_id);
            Ok(())
        } else {
            Err(RpcFrameworkError::ServiceNotFound(service_id.to_string()))
        }
    }

    /// 根据服务名称查找服务
    pub async fn discover_services(&self, service_name: &str) -> Vec<ServiceInfo> {
        {
            let mut stats = self.stats.write().await;
            stats.lookups += 1;
        }

        if let Some(service_ids) = self.services_by_name.get(service_name) {
            let mut services = Vec::new();
            for service_id in service_ids.iter() {
                if let Some(service_info) = self.services.get(service_id) {
                    services.push(service_info.clone());
                }
            }
            services
        } else {
            Vec::new()
        }
    }

    /// 获取特定服务信息
    pub async fn get_service(&self, service_id: &str) -> Option<ServiceInfo> {
        self.services.get(service_id).map(|entry| entry.clone())
    }

    /// 列出所有服务
    pub async fn list_all_services(&self) -> Vec<ServiceInfo> {
        self.services.iter().map(|entry| entry.value().clone()).collect()
    }

    /// 列出所有服务名称
    pub async fn list_service_names(&self) -> Vec<String> {
        self.services_by_name.iter().map(|entry| entry.key().clone()).collect()
    }

    /// 获取注册中心统计信息
    pub async fn get_stats(&self) -> RegistryStats {
        self.stats.read().await.clone()
    }

    /// 清理过期服务
    async fn cleanup_expired_services(
        services: &DashMap<String, ServiceInfo>,
        services_by_name: &DashMap<String, Vec<String>>,
        stats: &RwLock<RegistryStats>,
        ttl_seconds: u64,
    ) {
        let mut expired_services = Vec::new();

        // 找出过期的服务
        for entry in services.iter() {
            if entry.value().is_expired(ttl_seconds) {
                expired_services.push((entry.key().clone(), entry.value().clone()));
            }
        }

        if !expired_services.is_empty() {
            info!("Cleaning up {} expired services", expired_services.len());
            let expired_count = expired_services.len();

            // 移除过期服务
            for (service_id, service_info) in expired_services {
                warn!("Service expired: {} ({})", service_info.service_name, service_id);

                // 从主映射中移除
                services.remove(&service_id);

                // 从按名称索引中移除
                if let Some(mut service_ids) = services_by_name.get_mut(&service_info.service_name) {
                    service_ids.retain(|id| id != &service_id);
                    if service_ids.is_empty() {
                        drop(service_ids);
                        services_by_name.remove(&service_info.service_name);
                    }
                }
            }

            // 更新统计
            {
                let mut stats = stats.write().await;
                stats.expired_services += expired_count;
                stats.total_services = services.len();
                stats.active_services = services.len();
            }
        }
    }

    /// 关闭注册中心
    pub async fn shutdown(&mut self) {
        if let Some(task) = self._cleanup_task.take() {
            task.abort();
        }
        
        self.services.clear();
        self.services_by_name.clear();
        
        info!("Service registry shutdown");
    }
}

/// 服务发现客户端
pub struct ServiceDiscoveryClient {
    registry: Arc<ServiceRegistry>,
}

impl ServiceDiscoveryClient {
    pub fn new(registry: Arc<ServiceRegistry>) -> Self {
        Self { registry }
    }

    /// 发现服务
    pub async fn discover(&self, service_name: &str) -> Vec<ServiceInfo> {
        self.registry.discover_services(service_name).await
    }

    /// 选择一个服务实例（简单轮询）
    pub async fn select_service(&self, service_name: &str) -> Option<ServiceInfo> {
        let services = self.registry.discover_services(service_name).await;
        if services.is_empty() {
            None
        } else {
            // 简单的轮询选择
            // 在实际应用中，可以实现更复杂的负载均衡策略
            Some(services[0].clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_service_registration() {
        let config = RegistryConfig::default();
        let registry = ServiceRegistry::new(config);

        let service_info = ServiceInfo::new(
            "test-service".to_string(),
            "test-001".to_string(),
            "127.0.0.1:8001".parse().unwrap(),
            "1.0.0".to_string(),
        );

        registry.register_service(service_info.clone()).await.unwrap();

        let discovered = registry.discover_services("test-service").await;
        assert_eq!(discovered.len(), 1);
        assert_eq!(discovered[0].service_id, "test-001");
    }

    #[tokio::test]
    async fn test_service_deregistration() {
        let config = RegistryConfig::default();
        let registry = ServiceRegistry::new(config);

        let service_info = ServiceInfo::new(
            "test-service".to_string(),
            "test-001".to_string(),
            "127.0.0.1:8001".parse().unwrap(),
            "1.0.0".to_string(),
        );

        registry.register_service(service_info).await.unwrap();
        registry.deregister_service("test-001").await.unwrap();

        let discovered = registry.discover_services("test-service").await;
        assert_eq!(discovered.len(), 0);
    }

    #[tokio::test]
    async fn test_heartbeat() {
        let config = RegistryConfig::default();
        let registry = ServiceRegistry::new(config);

        let service_info = ServiceInfo::new(
            "test-service".to_string(),
            "test-001".to_string(),
            "127.0.0.1:8001".parse().unwrap(),
            "1.0.0".to_string(),
        );

        registry.register_service(service_info).await.unwrap();
        
        let result = registry.heartbeat("test-001").await;
        assert!(result.is_ok());

        let stats = registry.get_stats().await;
        assert_eq!(stats.heartbeats, 1);
    }
} 