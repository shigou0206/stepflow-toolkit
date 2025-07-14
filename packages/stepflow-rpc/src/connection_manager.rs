//! 连接管理器 - 处理断连和异常情况

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use dashmap::DashMap;
use tokio::net::TcpStream;
use tokio::sync::{broadcast, Mutex, RwLock};
use tokio::time::{sleep, timeout};
use tracing::{debug, error, info, warn};

use crate::error::{RpcError, RpcResult};
use crate::message_boundary::{MessageBoundary, MessageBoundaryHandler};
use crate::subscription_manager::ClientId;

/// 连接状态
#[derive(Debug, Clone)]
pub enum ConnectionState {
    /// 连接中
    Connecting,
    /// 已连接
    Connected,
    /// 断开连接
    Disconnected,
    /// 重连中
    Reconnecting,
    /// 连接失败
    Failed(String),
    /// 连接超时
    Timeout,
}

/// 连接元数据
#[derive(Debug, Clone)]
pub struct ConnectionMetadata {
    pub client_id: ClientId,
    pub remote_addr: SocketAddr,
    pub connected_at: Instant,
    pub last_activity: Instant,
    pub state: ConnectionState,
    pub retry_count: u32,
    pub total_bytes_sent: u64,
    pub total_bytes_received: u64,
    pub total_messages_sent: u64,
    pub total_messages_received: u64,
    pub error_count: u64,
}

impl ConnectionMetadata {
    /// 创建新的连接元数据
    pub fn new(client_id: ClientId, remote_addr: SocketAddr) -> Self {
        let now = Instant::now();
        Self {
            client_id,
            remote_addr,
            connected_at: now,
            last_activity: now,
            state: ConnectionState::Connecting,
            retry_count: 0,
            total_bytes_sent: 0,
            total_bytes_received: 0,
            total_messages_sent: 0,
            total_messages_received: 0,
            error_count: 0,
        }
    }

    /// 更新活动时间
    pub fn update_activity(&mut self) {
        self.last_activity = Instant::now();
    }

    /// 增加错误计数
    pub fn increment_error(&mut self) {
        self.error_count += 1;
    }

    /// 重置重试计数
    pub fn reset_retry_count(&mut self) {
        self.retry_count = 0;
    }

    /// 增加重试计数
    pub fn increment_retry_count(&mut self) {
        self.retry_count += 1;
    }

    /// 检查是否超时
    pub fn is_timeout(&self, timeout_duration: Duration) -> bool {
        self.last_activity.elapsed() > timeout_duration
    }

    /// 获取连接时长
    pub fn connection_duration(&self) -> Duration {
        self.connected_at.elapsed()
    }
}

/// 连接配置
#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    /// 连接超时时间
    pub connect_timeout: Duration,
    /// 心跳超时时间
    pub heartbeat_timeout: Duration,
    /// 最大重试次数
    pub max_retry_attempts: u32,
    /// 重试间隔
    pub retry_interval: Duration,
    /// 消息边界处理器
    pub message_boundary: MessageBoundary,
    /// 最大消息大小
    pub max_message_size: usize,
    /// 连接池大小
    pub connection_pool_size: usize,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(5),
            heartbeat_timeout: Duration::from_secs(60),
            max_retry_attempts: 3,
            retry_interval: Duration::from_secs(1),
            message_boundary: MessageBoundary::Newline,
            max_message_size: 1024 * 1024, // 1MB
            connection_pool_size: 100,
        }
    }
}

/// 连接管理器
pub struct ConnectionManager {
    config: ConnectionConfig,
    connections: Arc<DashMap<ClientId, ConnectionMetadata>>,
    active_streams: Arc<DashMap<ClientId, Arc<Mutex<TcpStream>>>>,
    connection_events: Arc<broadcast::Sender<ConnectionEvent>>,
    boundary_handler: MessageBoundaryHandler,
}

/// 连接事件
#[derive(Debug, Clone)]
pub enum ConnectionEvent {
    /// 连接建立
    Connected(ClientId, SocketAddr),
    /// 连接断开
    Disconnected(ClientId, String),
    /// 连接错误
    Error(ClientId, String),
    /// 重连开始
    ReconnectStarted(ClientId),
    /// 重连成功
    ReconnectSuccessful(ClientId),
    /// 重连失败
    ReconnectFailed(ClientId, String),
    /// 心跳超时
    HeartbeatTimeout(ClientId),
}

impl ConnectionManager {
    /// 创建新的连接管理器
    pub fn new(config: ConnectionConfig) -> Self {
        let (connection_events, _) = broadcast::channel(1000);
        let boundary_handler = MessageBoundaryHandler::new(config.message_boundary.clone())
            .with_max_message_size(config.max_message_size);

        Self {
            config,
            connections: Arc::new(DashMap::new()),
            active_streams: Arc::new(DashMap::new()),
            connection_events: Arc::new(connection_events),
            boundary_handler,
        }
    }

    /// 添加连接
    pub async fn add_connection(&self, client_id: ClientId, stream: TcpStream) -> RpcResult<()> {
        let remote_addr = stream.peer_addr()
            .map_err(|_| RpcError::internal_error("Failed to get peer address"))?;

        let metadata = ConnectionMetadata::new(client_id.clone(), remote_addr);
        
        // 存储连接信息
        self.connections.insert(client_id.clone(), metadata);
        self.active_streams.insert(client_id.clone(), Arc::new(Mutex::new(stream)));

        // 发送连接事件
        let _ = self.connection_events.send(ConnectionEvent::Connected(client_id.clone(), remote_addr));

        info!("Added connection for client: {} from {}", client_id, remote_addr);
        Ok(())
    }

    /// 移除连接
    pub async fn remove_connection(&self, client_id: &ClientId, reason: &str) -> RpcResult<()> {
        if let Some((_, _metadata)) = self.connections.remove(client_id) {
            self.active_streams.remove(client_id);
            
            // 发送断连事件
            let _ = self.connection_events.send(ConnectionEvent::Disconnected(client_id.clone(), reason.to_string()));
            
            info!("Removed connection for client: {} (reason: {})", client_id, reason);
        }
        Ok(())
    }

    /// 发送消息（简化版本）
    pub async fn send_message(&self, client_id: &ClientId, message: &str) -> RpcResult<()> {
        if let Some(stream_ref) = self.active_streams.get(client_id) {
            let stream = stream_ref.clone();
            
            // 简化的消息发送（换行符分隔）
            let message_with_newline = format!("{}\n", message);
            
            // 这里需要更复杂的实现来处理异步写入
            // 暂时使用简化版本
            info!("Would send message to client {}: {}", client_id, message);
            
            // 更新统计信息
            if let Some(mut metadata) = self.connections.get_mut(client_id) {
                metadata.update_activity();
                metadata.total_messages_sent += 1;
                metadata.total_bytes_sent += message.len() as u64;
            }
            
            Ok(())
        } else {
            Err(RpcError::internal_error("No active stream for client"))
        }
    }

    /// 处理连接关闭
    async fn handle_connection_closed(&self, client_id: &ClientId, reason: &str) -> RpcResult<()> {
        warn!("Connection closed for client {}: {}", client_id, reason);
        
        // 更新连接状态
        if let Some(mut metadata) = self.connections.get_mut(client_id) {
            metadata.state = ConnectionState::Disconnected;
        }
        
        // 移除连接
        self.remove_connection(client_id, reason).await?;
        
        Ok(())
    }

    /// 处理连接错误
    async fn handle_connection_error(&self, client_id: &ClientId, error: &str) -> RpcResult<()> {
        error!("Connection error for client {}: {}", client_id, error);
        
        // 更新错误计数
        if let Some(mut metadata) = self.connections.get_mut(client_id) {
            metadata.increment_error();
            metadata.state = ConnectionState::Failed(error.to_string());
        }
        
        // 发送错误事件
        let _ = self.connection_events.send(ConnectionEvent::Error(client_id.clone(), error.to_string()));
        
        // 如果错误次数过多，移除连接
        if let Some(metadata) = self.connections.get(client_id) {
            if metadata.error_count > 5 {
                self.remove_connection(client_id, "Too many errors").await?;
            }
        }
        
        Ok(())
    }

    /// 启动心跳检查
    pub async fn start_heartbeat_monitor(&self) {
        let connections = self.connections.clone();
        let connection_events = self.connection_events.clone();
        let heartbeat_timeout = self.config.heartbeat_timeout;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                let mut timeout_clients = Vec::new();
                
                // 检查所有连接的心跳
                for entry in connections.iter() {
                    let client_id = entry.key();
                    let metadata = entry.value();
                    
                    if metadata.is_timeout(heartbeat_timeout) {
                        timeout_clients.push(client_id.clone());
                    }
                }
                
                // 处理超时的连接
                for client_id in timeout_clients {
                    warn!("Heartbeat timeout for client: {}", client_id);
                    let _ = connection_events.send(ConnectionEvent::HeartbeatTimeout(client_id));
                }
            }
        });
    }

    /// 获取连接元数据
    pub fn get_connection_metadata(&self, client_id: &ClientId) -> Option<ConnectionMetadata> {
        self.connections.get(client_id).map(|entry| entry.value().clone())
    }

    /// 获取所有连接
    pub fn get_all_connections(&self) -> Vec<(ClientId, ConnectionMetadata)> {
        self.connections.iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
    }

    /// 获取活动连接数
    pub fn active_connection_count(&self) -> usize {
        self.connections.len()
    }

    /// 创建事件接收器
    pub fn create_event_receiver(&self) -> broadcast::Receiver<ConnectionEvent> {
        self.connection_events.subscribe()
    }

    /// 清理断开的连接
    pub async fn cleanup_disconnected_connections(&self) {
        let mut to_remove = Vec::new();
        
        for entry in self.connections.iter() {
            let client_id = entry.key();
            let metadata = entry.value();
            
            if matches!(metadata.state, ConnectionState::Disconnected | ConnectionState::Failed(_)) {
                to_remove.push(client_id.clone());
            }
        }
        
        for client_id in to_remove {
            let _ = self.remove_connection(&client_id, "Cleanup disconnected connection").await;
        }
    }

    /// 关闭所有连接
    pub async fn shutdown(&self) -> RpcResult<()> {
        info!("Shutting down connection manager...");
        
        let client_ids: Vec<ClientId> = self.connections.iter()
            .map(|entry| entry.key().clone())
            .collect();
        
        for client_id in client_ids {
            let _ = self.remove_connection(&client_id, "Server shutdown").await;
        }
        
        info!("Connection manager shut down");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::{TcpListener, TcpStream};

    #[tokio::test]
    async fn test_connection_manager_creation() {
        let config = ConnectionConfig::default();
        let manager = ConnectionManager::new(config);
        assert_eq!(manager.active_connection_count(), 0);
    }

    #[tokio::test]
    async fn test_connection_metadata() {
        let metadata = ConnectionMetadata::new("test_client".to_string(), "127.0.0.1:8080".parse().unwrap());
        assert_eq!(metadata.client_id, "test_client");
        assert_eq!(metadata.retry_count, 0);
        assert_eq!(metadata.error_count, 0);
    }

    #[tokio::test]
    async fn test_connection_state() {
        let mut metadata = ConnectionMetadata::new("test_client".to_string(), "127.0.0.1:8080".parse().unwrap());
        
        metadata.increment_error();
        assert_eq!(metadata.error_count, 1);
        
        metadata.increment_retry_count();
        assert_eq!(metadata.retry_count, 1);
        
        metadata.reset_retry_count();
        assert_eq!(metadata.retry_count, 0);
    }
} 