//! 事件发布和订阅管理

use std::collections::HashMap;
use std::sync::Arc;

use dashmap::DashMap;
use serde_json::Value;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, info, warn};

use crate::error::RpcResult;
use crate::protocol::{EventHandler, EventSubscription, RpcEvent};

/// 事件发布者
#[derive(Clone)]
pub struct EventPublisher {
    /// 事件通道发送者
    sender: broadcast::Sender<RpcEvent>,
    /// 事件统计
    stats: Arc<RwLock<EventStats>>,
}

/// 事件统计信息
#[derive(Debug, Default, Clone)]
pub struct EventStats {
    pub total_events: u64,
    pub events_by_type: HashMap<String, u64>,
    pub active_subscribers: u64,
    pub total_subscribers: u64,
}

impl EventPublisher {
    /// 创建新的事件发布者
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1000); // 缓冲区大小
        Self {
            sender,
            stats: Arc::new(RwLock::new(EventStats::default())),
        }
    }

    /// 发布事件
    pub async fn publish(&self, event: RpcEvent) -> RpcResult<()> {
        debug!("Publishing event: {:?}", event);
        
        // 更新统计信息
        {
            let mut stats = self.stats.write().await;
            stats.total_events += 1;
            *stats.events_by_type.entry(event.event.clone()).or_insert(0) += 1;
        }

        // 发送事件
        match self.sender.send(event.clone()) {
            Ok(receiver_count) => {
                debug!("Event sent to {} subscribers", receiver_count);
                Ok(())
            }
            Err(broadcast::error::SendError(_)) => {
                warn!("No subscribers for event: {}", event.event);
                Ok(()) // 没有订阅者不是错误
            }
        }
    }

    /// 发布简单事件
    pub async fn publish_simple(&self, event_name: &str, data: Value) -> RpcResult<()> {
        let event = RpcEvent::new(event_name.to_string(), data);
        self.publish(event).await
    }

    /// 发布流事件
    pub async fn publish_stream(&self, event_name: &str, data: Value, stream_id: String) -> RpcResult<()> {
        let event = RpcEvent::with_stream(event_name.to_string(), data, stream_id);
        self.publish(event).await
    }

    /// 发布序列化事件
    pub async fn publish_sequenced(&self, event_name: &str, data: Value, stream_id: String, sequence: u64) -> RpcResult<()> {
        let event = RpcEvent::with_sequence(event_name.to_string(), data, stream_id, sequence);
        self.publish(event).await
    }

    /// 创建订阅者
    pub fn subscribe(&self) -> EventSubscriber {
        let receiver = self.sender.subscribe();
        EventSubscriber::new(receiver)
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> EventStats {
        self.stats.read().await.clone()
    }

    /// 获取订阅者数量
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

/// 事件订阅者
pub struct EventSubscriber {
    receiver: broadcast::Receiver<RpcEvent>,
    subscriptions: Arc<DashMap<String, EventSubscription>>,
    handlers: Arc<DashMap<String, Arc<dyn EventHandler>>>,
}

impl EventSubscriber {
    /// 创建新的事件订阅者
    fn new(receiver: broadcast::Receiver<RpcEvent>) -> Self {
        Self {
            receiver,
            subscriptions: Arc::new(DashMap::new()),
            handlers: Arc::new(DashMap::new()),
        }
    }

    /// 订阅事件
    pub fn subscribe_to(&self, event_pattern: String, callback_id: String) -> String {
        let subscription = EventSubscription::new(event_pattern.clone(), callback_id.clone());
        let subscription_id = uuid::Uuid::new_v4().to_string();
        
        self.subscriptions.insert(subscription_id.clone(), subscription);
        info!("Subscribed to event pattern: {} with callback: {}", event_pattern, callback_id);
        
        subscription_id
    }

    /// 取消订阅
    pub fn unsubscribe(&self, subscription_id: &str) {
        if let Some((_, subscription)) = self.subscriptions.remove(subscription_id) {
            info!("Unsubscribed from event pattern: {}", subscription.event_pattern);
        }
    }

    /// 添加事件处理器
    pub fn add_handler(&self, handler: Arc<dyn EventHandler>) {
        for event_pattern in handler.interested_events() {
            let handler_id = format!("{}_{}", event_pattern, uuid::Uuid::new_v4());
            self.handlers.insert(handler_id.clone(), handler.clone());
            self.subscribe_to(event_pattern, handler_id);
        }
    }

    /// 开始监听事件
    pub async fn start_listening(&mut self) {
        loop {
            match self.receiver.recv().await {
                Ok(event) => {
                    debug!("Received event: {:?}", event);
                    self.handle_event(&event).await;
                }
                Err(broadcast::error::RecvError::Lagged(skipped)) => {
                    warn!("Event subscriber lagged, skipped {} events", skipped);
                }
                Err(broadcast::error::RecvError::Closed) => {
                    info!("Event channel closed");
                    break;
                }
            }
        }
    }

    /// 处理事件
    async fn handle_event(&self, event: &RpcEvent) {
        // 检查订阅
        let matching_subscriptions: Vec<_> = self.subscriptions
            .iter()
            .filter(|entry| entry.value().matches(event))
            .collect();

        for subscription_entry in matching_subscriptions {
            let subscription = subscription_entry.value();
            
            // 如果有对应的处理器，调用处理器
            if let Some(handler) = self.handlers.get(&subscription.callback_id) {
                handler.handle_event(event).await;
            } else {
                debug!("No handler found for callback: {}", subscription.callback_id);
            }
        }
    }

    /// 获取所有订阅
    pub fn get_subscriptions(&self) -> Vec<EventSubscription> {
        self.subscriptions.iter().map(|entry| entry.value().clone()).collect()
    }
}

/// 事件管理器 - 统一管理发布和订阅
pub struct EventManager {
    publisher: EventPublisher,
    global_subscriber: Arc<RwLock<Option<EventSubscriber>>>,
}

impl EventManager {
    /// 创建新的事件管理器
    pub fn new() -> Self {
        let publisher = EventPublisher::new();
        Self {
            publisher,
            global_subscriber: Arc::new(RwLock::new(None)),
        }
    }

    /// 获取发布者
    pub fn publisher(&self) -> &EventPublisher {
        &self.publisher
    }

    /// 创建新的订阅者
    pub fn create_subscriber(&self) -> EventSubscriber {
        self.publisher.subscribe()
    }

    /// 设置全局订阅者
    pub async fn set_global_subscriber(&self, subscriber: EventSubscriber) {
        let mut global_subscriber = self.global_subscriber.write().await;
        *global_subscriber = Some(subscriber);
    }

    /// 启动全局事件监听
    pub async fn start_global_listening(&self) {
        let global_subscriber = self.global_subscriber.clone();
        
        tokio::spawn(async move {
            if let Some(mut subscriber) = global_subscriber.write().await.take() {
                subscriber.start_listening().await;
            }
        });
    }

    /// 发布事件（简化接口）
    pub async fn publish(&self, event_name: &str, data: Value) -> RpcResult<()> {
        self.publisher.publish_simple(event_name, data).await
    }

    /// 发布流事件
    pub async fn publish_stream(&self, event_name: &str, data: Value, stream_id: String) -> RpcResult<()> {
        self.publisher.publish_stream(event_name, data, stream_id).await
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> EventStats {
        self.publisher.get_stats().await
    }
}

/// 简单的事件处理器实现
pub struct SimpleEventHandler {
    event_patterns: Vec<String>,
    handler: Box<dyn Fn(&RpcEvent) -> futures::future::BoxFuture<'static, ()> + Send + Sync>,
}

impl SimpleEventHandler {
    pub fn new<F, Fut>(event_patterns: Vec<String>, handler: F) -> Self
    where
        F: Fn(&RpcEvent) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        Self {
            event_patterns,
            handler: Box::new(move |event| {
                let event_clone = event.clone();
                Box::pin(handler(&event_clone))
            }),
        }
    }
}

#[async_trait::async_trait]
impl EventHandler for SimpleEventHandler {
    async fn handle_event(&self, event: &RpcEvent) {
        (self.handler)(event).await;
    }

    fn interested_events(&self) -> Vec<String> {
        self.event_patterns.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_event_publisher() {
        let publisher = EventPublisher::new();
        let event = RpcEvent::new("test.event".to_string(), json!({"message": "hello"}));
        
        // 没有订阅者时不应该出错
        publisher.publish(event).await.unwrap();
        
        let stats = publisher.get_stats().await;
        assert_eq!(stats.total_events, 1);
    }

    #[tokio::test]
    async fn test_event_subscription() {
        let publisher = EventPublisher::new();
        let subscriber = publisher.subscribe();
        
        let subscription_id = subscriber.subscribe_to("test.*".to_string(), "callback1".to_string());
        assert!(!subscription_id.is_empty());
        
        let subscriptions = subscriber.get_subscriptions();
        assert_eq!(subscriptions.len(), 1);
        assert_eq!(subscriptions[0].event_pattern, "test.*");
    }
} 