//! JSON-RPC TCP 客户端

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use dashmap::DashMap;
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::{broadcast, RwLock};
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

use crate::error::{RpcError, RpcFrameworkError, RpcResult};
use crate::protocol::{RpcEvent, RpcRequest, RpcResponse, RpcResponseMessage, ServerMessage};
use crate::subscription_manager::{ClientId, EventFilter, SubscriptionId, SubscriptionManager};

/// 客户端连接状态
#[derive(Debug, Clone)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Failed(String),
}

/// 客户端统计信息
#[derive(Debug, Clone)]
pub struct ClientStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub connection_attempts: u64,
    pub reconnection_attempts: u64,
    pub events_received: u64,
    pub active_subscriptions: u64,
}

impl Default for ClientStats {
    fn default() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            connection_attempts: 0,
            reconnection_attempts: 0,
            events_received: 0,
            active_subscriptions: 0,
        }
    }
}

/// 客户端连接配置
#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub client_id: ClientId,
    pub connect_timeout: Duration,
    pub request_timeout: Duration,
    pub reconnect_interval: Duration,
    pub max_reconnect_attempts: u32,
    pub enable_heartbeat: bool,
    pub heartbeat_interval: Duration,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            client_id: "default_client".to_string(),
            connect_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(30),
            reconnect_interval: Duration::from_secs(5),
            max_reconnect_attempts: 5,
            enable_heartbeat: true,
            heartbeat_interval: Duration::from_secs(30),
        }
    }
}

/// JSON-RPC 客户端
pub struct RpcClient {
    config: ClientConfig,
    server_addr: SocketAddr,
    connection_state: Arc<RwLock<ConnectionState>>,
    stream: Arc<RwLock<Option<TcpStream>>>,
    
    // 请求管理
    pending_requests: Arc<DashMap<String, tokio::sync::oneshot::Sender<RpcResponse>>>,
    request_counter: Arc<RwLock<u64>>,
    
    // 事件管理
    event_sender: Arc<broadcast::Sender<RpcEvent>>,
    subscription_manager: Arc<SubscriptionManager>,
    
    // 统计信息
    stats: Arc<RwLock<ClientStats>>,
    
    // 控制通道
    shutdown_sender: Arc<RwLock<Option<tokio::sync::oneshot::Sender<()>>>>,
}

impl RpcClient {
    /// 创建新的 RPC 客户端
    pub fn new(server_addr: SocketAddr, config: ClientConfig) -> Self {
        let (event_sender, _) = broadcast::channel(1000);
        let subscription_manager = Arc::new(SubscriptionManager::new());
        
        Self {
            config,
            server_addr,
            connection_state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            stream: Arc::new(RwLock::new(None)),
            pending_requests: Arc::new(DashMap::new()),
            request_counter: Arc::new(RwLock::new(0)),
            event_sender: Arc::new(event_sender),
            subscription_manager,
            stats: Arc::new(RwLock::new(ClientStats::default())),
            shutdown_sender: Arc::new(RwLock::new(None)),
        }
    }

    /// 使用客户端ID创建客户端
    pub fn with_client_id(server_addr: SocketAddr, client_id: ClientId) -> Self {
        let config = ClientConfig {
            client_id,
            ..Default::default()
        };
        Self::new(server_addr, config)
    }

    /// 连接到服务器
    pub async fn connect(&self) -> RpcResult<()> {
        let mut connection_state = self.connection_state.write().await;
        *connection_state = ConnectionState::Connecting;
        
        // 更新统计信息
        {
            let mut stats = self.stats.write().await;
            stats.connection_attempts += 1;
        }

        let stream = match timeout(self.config.connect_timeout, TcpStream::connect(self.server_addr)).await {
            Ok(Ok(stream)) => stream,
            Ok(Err(e)) => {
                *connection_state = ConnectionState::Failed(format!("Connection error: {}", e));
                return Err(RpcFrameworkError::RpcError(RpcError::internal_error(&format!("Failed to connect: {}", e))));
            }
            Err(_) => {
                *connection_state = ConnectionState::Failed("Connection timeout".to_string());
                return Err(RpcFrameworkError::RpcError(RpcError::internal_error("Connection timeout")));
            }
        };

        // 保存连接
        {
            let mut stream_guard = self.stream.write().await;
            *stream_guard = Some(stream);
        }

        *connection_state = ConnectionState::Connected;
        info!("Connected to server at {}", self.server_addr);

        // 启动消息处理器
        self.start_message_handler().await;

        // 启动心跳
        if self.config.enable_heartbeat {
            self.start_heartbeat().await;
        }

        Ok(())
    }

    /// 发送请求并等待响应
    pub async fn send_request(&self, method: &str, params: Value) -> RpcResult<Value> {
        let request = RpcRequest::new(method.to_string(), Some(params));
        
        // 获取请求ID
        let request_id = match request.get_id() {
            Some(id) => id.as_str().unwrap_or_default().to_string(),
            None => return Err(RpcFrameworkError::RpcError(RpcError::internal_error("No request ID generated"))),
        };
        
        // 创建响应通道
        let (response_tx, response_rx) = tokio::sync::oneshot::channel();
        self.pending_requests.insert(request_id.clone(), response_tx);

        // 发送请求
        let request_json = serde_json::to_string(&request)
            .map_err(|e| RpcFrameworkError::RpcError(RpcError::internal_error(&format!("Failed to serialize request: {}", e))))?;

        self.send_message(&request_json).await?;

        // 更新统计信息
        {
            let mut stats = self.stats.write().await;
            stats.total_requests += 1;
        }

        // 等待响应
        let response = timeout(self.config.request_timeout, response_rx).await
            .map_err(|_| RpcFrameworkError::RpcError(RpcError::internal_error("Request timeout")))?
            .map_err(|_| RpcFrameworkError::RpcError(RpcError::internal_error("Response channel closed")))?;

        // 处理响应
        if let Some(result) = response.result {
            let mut stats = self.stats.write().await;
            stats.successful_requests += 1;
            Ok(result)
        } else if let Some(error) = response.error {
            let mut stats = self.stats.write().await;
            stats.failed_requests += 1;
            return Err(RpcFrameworkError::RpcError(error));
        } else {
            let mut stats = self.stats.write().await;
            stats.failed_requests += 1;
            Err(RpcFrameworkError::RpcError(RpcError::internal_error("Invalid response format")))
        }
    }

    /// 订阅事件
    pub async fn subscribe_events(&self, filter: EventFilter) -> RpcResult<SubscriptionId> {
        let subscription_id = self.subscription_manager.add_subscription(
            self.config.client_id.clone(), 
            filter, 
            format!("client_{}", self.config.client_id)
        ).await?;
        
        // 更新统计信息
        {
            let mut stats = self.stats.write().await;
            stats.active_subscriptions += 1;
        }

        Ok(subscription_id)
    }

    /// 取消订阅
    pub async fn unsubscribe(&self, subscription_id: &SubscriptionId) -> RpcResult<()> {
        self.subscription_manager.remove_subscription(subscription_id).await?;
        
        let mut stats = self.stats.write().await;
        stats.active_subscriptions = stats.active_subscriptions.saturating_sub(1);

        Ok(())
    }

    /// 创建事件接收器
    pub fn create_event_receiver(&self) -> broadcast::Receiver<RpcEvent> {
        self.event_sender.subscribe()
    }

    /// 获取客户端ID
    pub fn client_id(&self) -> &ClientId {
        &self.config.client_id
    }

    /// 获取连接状态
    pub async fn connection_state(&self) -> ConnectionState {
        self.connection_state.read().await.clone()
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> ClientStats {
        self.stats.read().await.clone()
    }

    /// 启动消息处理器
    async fn start_message_handler(&self) {
        let stream = self.stream.clone();
        let pending_requests = self.pending_requests.clone();
        let event_sender = self.event_sender.clone();
        let subscription_manager = self.subscription_manager.clone();
        let stats = self.stats.clone();
        let _client_id = self.config.client_id.clone();

        tokio::spawn(async move {
            loop {
                // 取出 TcpStream，避免长时间持有锁
                let tcp_stream = {
                    let mut stream_guard = stream.write().await;
                    stream_guard.take()
                };
                
                if let Some(tcp_stream) = tcp_stream {
                    let mut reader = BufReader::new(tcp_stream);
                    let mut line = String::new();
                    
                    match reader.read_line(&mut line).await {
                        Ok(0) => {
                            // 连接关闭
                            warn!("Connection closed by server");
                            break;
                        }
                        Ok(_) => {
                            line = line.trim().to_string();
                            if !line.is_empty() {
                                Self::handle_message(
                                    &line,
                                    &pending_requests,
                                    &event_sender,
                                    &subscription_manager,
                                    &stats,
                                ).await;
                            }
                            
                            // 重新获取内部的 TcpStream
                            let inner_stream = reader.into_inner();
                            let mut stream_guard = stream.write().await;
                            *stream_guard = Some(inner_stream);
                        }
                        Err(e) => {
                            error!("Error reading from stream: {}", e);
                            break;
                        }
                    }
                } else {
                    // 没有连接，等待一段时间后重试
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        });
    }

    /// 处理接收到的消息
    async fn handle_message(
        message: &str,
        pending_requests: &Arc<DashMap<String, tokio::sync::oneshot::Sender<RpcResponse>>>,
        event_sender: &Arc<broadcast::Sender<RpcEvent>>,
        subscription_manager: &Arc<SubscriptionManager>,
        stats: &Arc<RwLock<ClientStats>>,
    ) {
        debug!("Received message: {}", message);

        // 尝试解析为服务器消息
        if let Ok(server_message) = serde_json::from_str::<ServerMessage>(message) {
            match server_message {
                ServerMessage::Response(response) => {
                    // 处理响应消息
                    match response {
                        RpcResponseMessage::Single(single_response) => {
                            let request_id = single_response.id.as_str().unwrap_or_default().to_string();
                            if let Some((_, sender)) = pending_requests.remove(&request_id) {
                                if let Err(_) = sender.send(single_response) {
                                    warn!("Failed to send response to waiting request");
                                }
                            }
                        }
                        RpcResponseMessage::Batch(batch_responses) => {
                            for single_response in batch_responses {
                                let request_id = single_response.id.as_str().unwrap_or_default().to_string();
                                if let Some((_, sender)) = pending_requests.remove(&request_id) {
                                    if let Err(_) = sender.send(single_response) {
                                        warn!("Failed to send response to waiting request");
                                    }
                                }
                            }
                        }
                    }
                }
                ServerMessage::Event(event) => {
                    // 更新统计信息
                    {
                        let mut stats = stats.write().await;
                        stats.events_received += 1;
                    }

                    // 检查事件过滤器
                    let matching_subscriptions = subscription_manager.get_matching_subscriptions(&event).await;
                    if !matching_subscriptions.is_empty() {
                        let _ = event_sender.send(event);
                    }
                }
            }
        }
    }

    /// 发送消息到服务器
    async fn send_message(&self, message: &str) -> RpcResult<()> {
        let mut stream_guard = self.stream.write().await;
        if let Some(ref mut tcp_stream) = *stream_guard {
            let message_with_newline = format!("{}\n", message);
            
            tcp_stream.write_all(message_with_newline.as_bytes()).await
                .map_err(|e| RpcFrameworkError::RpcError(RpcError::internal_error(&format!("Failed to write to stream: {}", e))))?;
            
            tcp_stream.flush().await
                .map_err(|e| RpcFrameworkError::RpcError(RpcError::internal_error(&format!("Failed to flush stream: {}", e))))?;
            
            Ok(())
        } else {
            Err(RpcFrameworkError::RpcError(RpcError::internal_error("Not connected to server")))
        }
    }

    /// 启动心跳
    async fn start_heartbeat(&self) {
        let client = self.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(client.config.heartbeat_interval);
            
            loop {
                interval.tick().await;
                
                match client.connection_state().await {
                    ConnectionState::Connected => {
                        // 发送心跳
                        if let Err(e) = client.send_request("rpc.ping", Value::Null).await {
                            error!("Heartbeat failed: {:?}", e);
                            // 可以在这里触发重连逻辑
                        }
                    }
                    _ => {
                        // 连接断开，跳过心跳
                    }
                }
            }
        });
    }

    /// 断开连接
    pub async fn disconnect(&self) -> RpcResult<()> {
        // 发送关闭信号
        if let Some(sender) = self.shutdown_sender.write().await.take() {
            let _ = sender.send(());
        }

        // 清理连接
        {
            let mut stream_guard = self.stream.write().await;
            if let Some(stream) = stream_guard.take() {
                drop(stream);
            }
        }

        // 更新状态
        {
            let mut connection_state = self.connection_state.write().await;
            *connection_state = ConnectionState::Disconnected;
        }

        // 清理待处理请求
        self.pending_requests.clear();

        info!("Disconnected from server");
        Ok(())
    }
}

impl Clone for RpcClient {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            server_addr: self.server_addr,
            connection_state: self.connection_state.clone(),
            stream: self.stream.clone(),
            pending_requests: self.pending_requests.clone(),
            request_counter: self.request_counter.clone(),
            event_sender: self.event_sender.clone(),
            subscription_manager: self.subscription_manager.clone(),
            stats: self.stats.clone(),
            shutdown_sender: self.shutdown_sender.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::subscription_manager::EventFilter;

    #[tokio::test]
    async fn test_client_creation() {
        let addr = "127.0.0.1:0".parse().unwrap();
        let client = RpcClient::with_client_id(addr, "test_client".to_string());
        assert_eq!(client.client_id(), "test_client");
        assert!(matches!(client.connection_state().await, ConnectionState::Disconnected));
    }

    #[tokio::test]
    async fn test_event_subscription() {
        let addr = "127.0.0.1:0".parse().unwrap();
        let client = RpcClient::with_client_id(addr, "test_client".to_string());
        
        let filter = EventFilter::new("test.*".to_string());
        let subscription_id = client.subscribe_events(filter).await.unwrap();
        
        assert!(!subscription_id.is_empty());
        
        // 取消订阅
        client.unsubscribe(&subscription_id).await.unwrap();
    }
} 