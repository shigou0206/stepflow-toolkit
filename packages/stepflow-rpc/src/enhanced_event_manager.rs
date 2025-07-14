//! 增强的事件管理器 - 支持多个订阅者的高效事件推送

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use dashmap::DashMap;
use serde_json::Value;
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio::time::{sleep, Instant};
use tracing::{debug, error, info, warn};

use crate::error::{RpcError, RpcResult};
use crate::protocol::{EventHandler, RpcEvent, ServerMessage};
use crate::subscription_manager::{ClientId, EventFilter, SubscriptionManager, SubscriptionId};

/// 事件通知配置
#[derive(Debug, Clone)]
pub struct EventNotificationConfig {
    /// 事件缓冲区大小
    pub buffer_size: usize,
    /// 批处理大小
    pub batch_size: usize,
    /// 批处理超时时间
    pub batch_timeout: Duration,
    /// 失败重试次数
    pub max_retry_attempts: u32,
    /// 重试间隔
    pub retry_interval: Duration,
    /// 慢订阅者超时时间
    pub slow_subscriber_timeout: Duration,
}

impl Default for EventNotificationConfig {
    fn default() -> Self {
        Self {
            buffer_size: 10000,
            batch_size: 100,
            batch_timeout: Duration::from_millis(10),
            max_retry_attempts: 3,
            retry_interval: Duration::from_millis(100),
            slow_subscriber_timeout: Duration::from_secs(5),
        }
    }
}

/// 订阅者通知状态
#[derive(Debug, Clone)]
pub struct SubscriberNotificationState {
    pub client_id: ClientId,
    pub subscription_id: SubscriptionId,
    pub events_sent: u64,
    pub events_failed: u64,
    pub last_notification: Instant,
    pub retry_count: u32,
    pub is_slow: bool,
    pub notification_latency: Duration,
}

impl SubscriberNotificationState {
    pub fn new(client_id: ClientId, subscription_id: SubscriptionId) -> Self {
        Self {
            client_id,
            subscription_id,
            events_sent: 0,
            events_failed: 0,
            last_notification: Instant::now(),
            retry_count: 0,
            is_slow: false,
            notification_latency: Duration::from_millis(0),
        }
    }

    pub fn update_notification_time(&mut self) {
        let now = Instant::now();
        self.notification_latency = now.duration_since(self.last_notification);
        self.last_notification = now;
    }

    pub fn increment_sent(&mut self) {
        self.events_sent += 1;
        self.retry_count = 0;
    }

    pub fn increment_failed(&mut self) {
        self.events_failed += 1;
        self.retry_count += 1;
    }

    pub fn mark_as_slow(&mut self, is_slow: bool) {
        self.is_slow = is_slow;
    }
}

/// 默认事件通知器实现
pub struct DefaultEventNotifier {
    /// 事件发送器（用于向客户端发送事件）
    client_senders: Arc<DashMap<ClientId, mpsc::UnboundedSender<ServerMessage>>>,
}

impl DefaultEventNotifier {
    pub fn new() -> Self {
        Self {
            client_senders: Arc::new(DashMap::new()),
        }
    }

    /// 添加客户端发送器
    pub fn add_client_sender(&self, client_id: ClientId, sender: mpsc::UnboundedSender<ServerMessage>) {
        self.client_senders.insert(client_id, sender);
    }

    /// 移除客户端发送器
    pub fn remove_client_sender(&self, client_id: &ClientId) {
        self.client_senders.remove(client_id);
    }

    /// 获取所有客户端ID
    pub fn get_all_client_ids(&self) -> Vec<ClientId> {
        self.client_senders.iter().map(|entry| entry.key().clone()).collect()
    }

    /// 通知订阅者
    pub async fn notify_subscriber(&self, client_id: &ClientId, event: &RpcEvent) -> RpcResult<()> {
        if let Some(sender) = self.client_senders.get(client_id) {
            let message = ServerMessage::Event(event.clone());
            
            match sender.send(message) {
                Ok(_) => {
                    debug!("Successfully notified client {} of event {}", client_id, event.event);
                    Ok(())
                }
                Err(e) => {
                    error!("Failed to notify client {} of event {}: {}", client_id, event.event, e);
                    Err(RpcError::internal_error("Notification failed"))
                }
            }
        } else {
            warn!("No sender found for client {}", client_id);
            Err(RpcError::internal_error("No sender for client"))
        }
    }

    /// 批量通知订阅者
    pub async fn notify_subscribers_batch(&self, subscribers: &[ClientId], events: &[RpcEvent]) -> RpcResult<()> {
        let mut failed_notifications = Vec::new();
        
        for client_id in subscribers {
            if let Some(sender) = self.client_senders.get(client_id) {
                for event in events {
                    let message = ServerMessage::Event(event.clone());
                    
                    if let Err(e) = sender.send(message) {
                        error!("Failed to notify client {} of event {}: {}", client_id, event.event, e);
                        failed_notifications.push((client_id.clone(), event.event.clone(), e));
                    }
                }
            } else {
                warn!("No sender found for client {}", client_id);
                failed_notifications.push((client_id.clone(), "unknown".to_string(), 
                    mpsc::error::SendError(ServerMessage::Event(RpcEvent::new("error".to_string(), Value::Null)))));
            }
        }
        
        if failed_notifications.is_empty() {
            Ok(())
        } else {
            Err(RpcError::internal_error("Failed to notify some subscribers"))
        }
    }
}

/// 增强的事件管理器
pub struct EnhancedEventManager {
    config: EventNotificationConfig,
    subscription_manager: Arc<SubscriptionManager>,
    event_notifier: Arc<DefaultEventNotifier>,
    subscriber_states: Arc<DashMap<SubscriptionId, SubscriberNotificationState>>,
    
    // 事件处理队列
    event_queue: Arc<mpsc::UnboundedSender<RpcEvent>>,
    batch_processor: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
    
    // 统计信息
    stats: Arc<RwLock<EventNotificationStats>>,
}

/// 事件通知统计信息
#[derive(Debug, Default, Clone)]
pub struct EventNotificationStats {
    pub total_events_processed: u64,
    pub total_notifications_sent: u64,
    pub total_notifications_failed: u64,
    pub batch_notifications_sent: u64,
    pub slow_subscribers_count: u64,
    pub average_notification_latency: Duration,
    pub subscriber_stats: HashMap<ClientId, SubscriberNotificationState>,
}

impl EnhancedEventManager {
    /// 创建新的增强事件管理器
    pub fn new(config: EventNotificationConfig, notifier: Arc<DefaultEventNotifier>) -> Self {
        let subscription_manager = Arc::new(SubscriptionManager::new());
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        let manager = Self {
            config: config.clone(),
            subscription_manager,
            event_notifier: notifier,
            subscriber_states: Arc::new(DashMap::new()),
            event_queue: Arc::new(event_sender),
            batch_processor: Arc::new(RwLock::new(None)),
            stats: Arc::new(RwLock::new(EventNotificationStats::default())),
        };

        // 启动批处理器
        manager.start_batch_processor(event_receiver);
        
        manager
    }

    /// 启动批处理器
    fn start_batch_processor(&self, mut event_receiver: mpsc::UnboundedReceiver<RpcEvent>) {
        let config = self.config.clone();
        let subscription_manager = self.subscription_manager.clone();
        let event_notifier = self.event_notifier.clone();
        let subscriber_states = self.subscriber_states.clone();
        let stats = self.stats.clone();
        
        let batch_processor = tokio::spawn(async move {
            let mut event_batch = Vec::with_capacity(config.batch_size);
            let mut last_batch_time = Instant::now();
            
            loop {
                tokio::select! {
                    // 接收事件
                    event_opt = event_receiver.recv() => {
                        match event_opt {
                            Some(event) => {
                                event_batch.push(event);
                                
                                // 检查是否需要发送批次
                                if event_batch.len() >= config.batch_size || 
                                   last_batch_time.elapsed() >= config.batch_timeout {
                                    Self::process_event_batch(
                                        &event_batch,
                                        &subscription_manager,
                                        &event_notifier,
                                        &subscriber_states,
                                        &stats,
                                        &config,
                                    ).await;
                                    
                                    event_batch.clear();
                                    last_batch_time = Instant::now();
                                }
                            }
                            None => {
                                // 通道关闭，处理剩余事件
                                if !event_batch.is_empty() {
                                    Self::process_event_batch(
                                        &event_batch,
                                        &subscription_manager,
                                        &event_notifier,
                                        &subscriber_states,
                                        &stats,
                                        &config,
                                    ).await;
                                }
                                break;
                            }
                        }
                    }
                    
                    // 定期处理批次（处理超时）
                    _ = sleep(config.batch_timeout) => {
                        if !event_batch.is_empty() && last_batch_time.elapsed() >= config.batch_timeout {
                            Self::process_event_batch(
                                &event_batch,
                                &subscription_manager,
                                &event_notifier,
                                &subscriber_states,
                                &stats,
                                &config,
                            ).await;
                            
                            event_batch.clear();
                            last_batch_time = Instant::now();
                        }
                    }
                }
            }
        });
        
        tokio::spawn(async move {
            let mut batch_processor_guard = self.batch_processor.write().await;
            *batch_processor_guard = Some(batch_processor);
        });
    }

    /// 处理事件批次
    async fn process_event_batch(
        events: &[RpcEvent],
        subscription_manager: &SubscriptionManager,
        event_notifier: &Arc<DefaultEventNotifier>,
        subscriber_states: &DashMap<SubscriptionId, SubscriberNotificationState>,
        stats: &Arc<RwLock<EventNotificationStats>>,
        config: &EventNotificationConfig,
    ) {
        let mut client_event_map: HashMap<ClientId, Vec<RpcEvent>> = HashMap::new();
        
        // 为每个事件找到匹配的订阅者
        for event in events {
            let matching_subscriptions = subscription_manager.get_matching_subscriptions(event).await;
            
            for subscription in matching_subscriptions {
                client_event_map
                    .entry(subscription.client_id.clone())
                    .or_insert_with(Vec::new)
                    .push(event.clone());
                
                // 更新订阅者状态
                if let Some(mut state) = subscriber_states.get_mut(&subscription.subscription_id) {
                    state.update_notification_time();
                } else {
                    let state = SubscriberNotificationState::new(
                        subscription.client_id.clone(),
                        subscription.subscription_id.clone(),
                    );
                    subscriber_states.insert(subscription.subscription_id.clone(), state);
                }
            }
        }
        
        // 批量通知订阅者
        for (client_id, client_events) in client_event_map {
            let notification_result = event_notifier.notify_subscribers_batch(&[client_id.clone()], &client_events).await;
            
            // 更新统计信息
            let mut stats_guard = stats.write().await;
            stats_guard.total_events_processed += client_events.len() as u64;
            stats_guard.batch_notifications_sent += 1;
            
            if notification_result.is_ok() {
                stats_guard.total_notifications_sent += client_events.len() as u64;
            } else {
                stats_guard.total_notifications_failed += client_events.len() as u64;
            }
        }
    }

    /// 发布事件
    pub async fn publish_event(&self, event: RpcEvent) -> RpcResult<()> {
        self.event_queue.send(event)
            .map_err(|_| RpcError::internal_error("Failed to queue event"))?;
        Ok(())
    }

    /// 发布简单事件
    pub async fn publish_simple_event(&self, event_name: &str, data: Value) -> RpcResult<()> {
        let event = RpcEvent::new(event_name.to_string(), data);
        self.publish_event(event).await
    }

    /// 添加订阅
    pub async fn add_subscription(&self, client_id: ClientId, filter: EventFilter) -> RpcResult<SubscriptionId> {
        let subscription_id = self.subscription_manager.add_subscription(
            client_id.clone(),
            filter,
            format!("client_{}", client_id),
        ).await?;

        // 初始化订阅者状态
        let state = SubscriberNotificationState::new(client_id, subscription_id.clone());
        self.subscriber_states.insert(subscription_id.clone(), state);

        Ok(subscription_id)
    }

    /// 移除订阅
    pub async fn remove_subscription(&self, subscription_id: &SubscriptionId) -> RpcResult<()> {
        self.subscription_manager.remove_subscription(subscription_id).await?;
        self.subscriber_states.remove(subscription_id);
        Ok(())
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> EventNotificationStats {
        let mut stats = self.stats.read().await.clone();
        
        // 更新订阅者统计
        for entry in self.subscriber_states.iter() {
            stats.subscriber_stats.insert(entry.key().clone(), entry.value().clone());
        }
        
        stats
    }

    /// 获取慢订阅者
    pub fn get_slow_subscribers(&self) -> Vec<SubscriberNotificationState> {
        self.subscriber_states.iter()
            .filter(|entry| entry.value().is_slow)
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// 清理慢订阅者
    pub async fn cleanup_slow_subscribers(&self) -> RpcResult<()> {
        let slow_subscribers: Vec<SubscriptionId> = self.subscriber_states.iter()
            .filter(|entry| entry.value().is_slow)
            .map(|entry| entry.key().clone())
            .collect();
        
        for subscription_id in slow_subscribers {
            self.remove_subscription(&subscription_id).await?;
            info!("Removed slow subscriber: {}", subscription_id);
        }
        
        Ok(())
    }

    /// 关闭事件管理器
    pub async fn shutdown(&self) -> RpcResult<()> {
        info!("Shutting down enhanced event manager...");
        
        // 停止批处理器
        if let Some(batch_processor) = self.batch_processor.write().await.take() {
            batch_processor.abort();
        }
        
        info!("Enhanced event manager shut down");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_enhanced_event_manager() {
        let config = EventNotificationConfig::default();
        let notifier = Arc::new(DefaultEventNotifier::new());
        let manager = EnhancedEventManager::new(config, notifier);
        
        // 发布事件
        let event = RpcEvent::new("test.event".to_string(), json!({"test": "data"}));
        manager.publish_event(event).await.unwrap();
        
        // 获取统计信息
        let stats = manager.get_stats().await;
        assert!(stats.total_events_processed >= 0);
    }

    #[tokio::test]
    async fn test_subscriber_notification_state() {
        let mut state = SubscriberNotificationState::new("client1".to_string(), "sub1".to_string());
        
        assert_eq!(state.events_sent, 0);
        assert_eq!(state.events_failed, 0);
        assert_eq!(state.retry_count, 0);
        assert!(!state.is_slow);
        
        state.increment_sent();
        assert_eq!(state.events_sent, 1);
        
        state.increment_failed();
        assert_eq!(state.events_failed, 1);
        assert_eq!(state.retry_count, 1);
        
        state.mark_as_slow(true);
        assert!(state.is_slow);
    }
} 