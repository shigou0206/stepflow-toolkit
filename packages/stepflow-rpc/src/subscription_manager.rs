//! 订阅管理器 - 管理多个订阅和过滤器

use std::collections::HashMap;
use std::sync::Arc;

use dashmap::DashMap;
use tokio::sync::RwLock;
use tracing::info;

use crate::error::RpcResult;
use crate::protocol::{EventHandler, RpcEvent};

/// 客户端ID类型
pub type ClientId = String;

/// 订阅ID类型
pub type SubscriptionId = String;

/// 事件过滤器
#[derive(Debug, Clone)]
pub struct EventFilter {
    /// 事件类型匹配模式（支持通配符 * 和 ?）
    pub event_pattern: String,
    /// 事件数据过滤器（JSONPath 表达式）
    pub data_filter: Option<String>,
    /// 流ID过滤器
    pub stream_filter: Option<String>,
    /// 序列号范围过滤器
    pub sequence_range: Option<(u64, u64)>,
}

impl EventFilter {
    /// 创建新的事件过滤器
    pub fn new(event_pattern: String) -> Self {
        Self {
            event_pattern,
            data_filter: None,
            stream_filter: None,
            sequence_range: None,
        }
    }

    /// 设置数据过滤器
    pub fn with_data_filter(mut self, filter: String) -> Self {
        self.data_filter = Some(filter);
        self
    }

    /// 设置流过滤器
    pub fn with_stream_filter(mut self, filter: String) -> Self {
        self.stream_filter = Some(filter);
        self
    }

    /// 设置序列号范围过滤器
    pub fn with_sequence_range(mut self, start: u64, end: u64) -> Self {
        self.sequence_range = Some((start, end));
        self
    }

    /// 检查事件是否匹配过滤器
    pub fn matches(&self, event: &RpcEvent) -> bool {
        // 检查事件类型模式
        if !self.matches_pattern(&event.event, &self.event_pattern) {
            return false;
        }

        // 检查流ID过滤器
        if let Some(stream_filter) = &self.stream_filter {
            if let Some(stream_id) = &event.stream_id {
                if !self.matches_pattern(stream_id, stream_filter) {
                    return false;
                }
            } else {
                return false;
            }
        }

        // 检查序列号范围
        if let Some((start, end)) = self.sequence_range {
            if let Some(sequence) = event.sequence {
                if sequence < start || sequence > end {
                    return false;
                }
            } else {
                return false;
            }
        }

        // 检查数据过滤器（简化版本，实际应该使用JSONPath）
        if let Some(data_filter) = &self.data_filter {
            // 这里简化处理，实际应该使用JSONPath库
            let data_str = event.data.to_string();
            if !data_str.contains(data_filter) {
                return false;
            }
        }

        true
    }

    /// 模式匹配（支持 * 和 ?）
    fn matches_pattern(&self, text: &str, pattern: &str) -> bool {
        if pattern == "*" {
            return true;
        }

        // 简化的模式匹配实现
        if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                let prefix = parts[0];
                let suffix = parts[1];
                return text.starts_with(prefix) && text.ends_with(suffix);
            }
        }

        if pattern.contains('?') {
            // 简化的单字符匹配
            if pattern.len() == text.len() {
                return pattern.chars().zip(text.chars()).all(|(p, t)| p == '?' || p == t);
            }
        }

        text == pattern
    }
}

/// 客户端订阅信息
#[derive(Debug, Clone)]
pub struct ClientSubscription {
    pub client_id: ClientId,
    pub subscription_id: SubscriptionId,
    pub filter: EventFilter,
    pub callback_id: String,
    pub created_at: std::time::Instant,
    pub message_count: u64,
}

impl ClientSubscription {
    /// 创建新的客户端订阅
    pub fn new(client_id: ClientId, filter: EventFilter, callback_id: String) -> Self {
        Self {
            client_id,
            subscription_id: uuid::Uuid::new_v4().to_string(),
            filter,
            callback_id,
            created_at: std::time::Instant::now(),
            message_count: 0,
        }
    }

    /// 检查事件是否匹配订阅
    pub fn matches(&self, event: &RpcEvent) -> bool {
        self.filter.matches(event)
    }
}

/// 订阅管理器
pub struct SubscriptionManager {
    /// 按客户端ID分组的订阅
    client_subscriptions: Arc<DashMap<ClientId, Vec<ClientSubscription>>>,
    /// 按订阅ID索引的订阅
    subscription_index: Arc<DashMap<SubscriptionId, ClientSubscription>>,
    /// 事件处理器
    event_handlers: Arc<DashMap<String, Arc<dyn EventHandler>>>,
    /// 统计信息
    stats: Arc<RwLock<SubscriptionStats>>,
}

/// 订阅统计信息
#[derive(Debug, Default, Clone)]
pub struct SubscriptionStats {
    pub total_subscriptions: u64,
    pub active_subscriptions: u64,
    pub total_clients: u64,
    pub events_processed: u64,
    pub events_filtered: u64,
    pub subscriptions_by_pattern: HashMap<String, u64>,
}

impl SubscriptionManager {
    /// 创建新的订阅管理器
    pub fn new() -> Self {
        Self {
            client_subscriptions: Arc::new(DashMap::new()),
            subscription_index: Arc::new(DashMap::new()),
            event_handlers: Arc::new(DashMap::new()),
            stats: Arc::new(RwLock::new(SubscriptionStats::default())),
        }
    }

    /// 添加客户端订阅
    pub async fn add_subscription(
        &self,
        client_id: ClientId,
        filter: EventFilter,
        callback_id: String,
    ) -> RpcResult<SubscriptionId> {
        let subscription = ClientSubscription::new(client_id.clone(), filter, callback_id);
        let subscription_id = subscription.subscription_id.clone();

        // 添加到客户端订阅列表
        self.client_subscriptions
            .entry(client_id.clone())
            .or_insert_with(Vec::new)
            .push(subscription.clone());

        // 添加到订阅索引
        self.subscription_index.insert(subscription_id.clone(), subscription.clone());

        // 更新统计信息
        {
            let mut stats = self.stats.write().await;
            stats.total_subscriptions += 1;
            stats.active_subscriptions += 1;
            *stats.subscriptions_by_pattern
                .entry(subscription.filter.event_pattern.clone())
                .or_insert(0) += 1;
        }

        info!("Added subscription {} for client {}", subscription_id, client_id);
        Ok(subscription_id)
    }

    /// 移除订阅
    pub async fn remove_subscription(&self, subscription_id: &SubscriptionId) -> RpcResult<()> {
        if let Some((_, subscription)) = self.subscription_index.remove(subscription_id) {
            // 从客户端订阅列表中移除
            if let Some(mut client_subs) = self.client_subscriptions.get_mut(&subscription.client_id) {
                client_subs.retain(|s| s.subscription_id != *subscription_id);
                if client_subs.is_empty() {
                    self.client_subscriptions.remove(&subscription.client_id);
                }
            }

            // 更新统计信息
            {
                let mut stats = self.stats.write().await;
                stats.active_subscriptions = stats.active_subscriptions.saturating_sub(1);
                if let Some(count) = stats.subscriptions_by_pattern.get_mut(&subscription.filter.event_pattern) {
                    *count = count.saturating_sub(1);
                }
            }

            info!("Removed subscription {} for client {}", subscription_id, subscription.client_id);
        }
        Ok(())
    }

    /// 移除客户端的所有订阅
    pub async fn remove_client_subscriptions(&self, client_id: &ClientId) -> RpcResult<()> {
        if let Some((_, subscriptions)) = self.client_subscriptions.remove(client_id) {
            for subscription in subscriptions {
                self.subscription_index.remove(&subscription.subscription_id);
                
                // 更新统计信息
                {
                    let mut stats = self.stats.write().await;
                    stats.active_subscriptions = stats.active_subscriptions.saturating_sub(1);
                    if let Some(count) = stats.subscriptions_by_pattern.get_mut(&subscription.filter.event_pattern) {
                        *count = count.saturating_sub(1);
                    }
                }
            }
            info!("Removed all subscriptions for client {}", client_id);
        }
        Ok(())
    }

    /// 获取匹配的订阅
    pub async fn get_matching_subscriptions(&self, event: &RpcEvent) -> Vec<ClientSubscription> {
        let mut matching_subscriptions = Vec::new();
        
        for entry in self.subscription_index.iter() {
            let subscription = entry.value();
            if subscription.matches(event) {
                matching_subscriptions.push(subscription.clone());
            }
        }

        // 更新统计信息
        {
            let mut stats = self.stats.write().await;
            stats.events_processed += 1;
            if matching_subscriptions.is_empty() {
                stats.events_filtered += 1;
            }
        }

        matching_subscriptions
    }

    /// 获取客户端的所有订阅
    pub fn get_client_subscriptions(&self, client_id: &ClientId) -> Vec<ClientSubscription> {
        self.client_subscriptions
            .get(client_id)
            .map(|subs| subs.clone())
            .unwrap_or_default()
    }

    /// 获取所有客户端
    pub fn get_all_clients(&self) -> Vec<ClientId> {
        self.client_subscriptions.iter().map(|entry| entry.key().clone()).collect()
    }

    /// 添加事件处理器
    pub fn add_event_handler(&self, handler: Arc<dyn EventHandler>) {
        for event_pattern in handler.interested_events() {
            let handler_id = format!("{}_{}", event_pattern, uuid::Uuid::new_v4());
            self.event_handlers.insert(handler_id, handler.clone());
        }
    }

    /// 处理事件
    pub async fn handle_event(&self, event: &RpcEvent) -> RpcResult<()> {
        let matching_subscriptions = self.get_matching_subscriptions(event).await;
        
        for subscription in matching_subscriptions {
            // 调用事件处理器
            if let Some(handler) = self.event_handlers.get(&subscription.callback_id) {
                handler.handle_event(event).await;
            }
        }

        Ok(())
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> SubscriptionStats {
        let mut stats = self.stats.write().await;
        stats.total_clients = self.client_subscriptions.len() as u64;
        stats.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_subscription_manager() {
        let manager = SubscriptionManager::new();
        
        // 添加订阅
        let filter = EventFilter::new("user.*".to_string());
        let subscription_id = manager.add_subscription(
            "client1".to_string(),
            filter,
            "callback1".to_string(),
        ).await.unwrap();

        // 检查订阅
        let event = RpcEvent::new("user.login".to_string(), json!({"user": "test"}));
        let matches = manager.get_matching_subscriptions(&event).await;
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].client_id, "client1");

        // 移除订阅
        manager.remove_subscription(&subscription_id).await.unwrap();
        let matches = manager.get_matching_subscriptions(&event).await;
        assert_eq!(matches.len(), 0);
    }

    #[tokio::test]
    async fn test_event_filter() {
        let filter = EventFilter::new("user.*".to_string())
            .with_stream_filter("stream1".to_string())
            .with_sequence_range(1, 100);

        let event1 = RpcEvent::with_sequence(
            "user.login".to_string(),
            json!({"user": "test"}),
            "stream1".to_string(),
            50,
        );
        assert!(filter.matches(&event1));

        let event2 = RpcEvent::with_sequence(
            "user.login".to_string(),
            json!({"user": "test"}),
            "stream2".to_string(),
            50,
        );
        assert!(!filter.matches(&event2)); // 不匹配流ID

        let event3 = RpcEvent::with_sequence(
            "order.create".to_string(),
            json!({"order": "test"}),
            "stream1".to_string(),
            50,
        );
        assert!(!filter.matches(&event3)); // 不匹配事件类型
    }
} 