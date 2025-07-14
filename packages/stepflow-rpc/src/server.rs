//! TCP JSON-RPC Server Implementation

use std::net::SocketAddr;
use std::sync::Arc;

use bytes::{BufMut, BytesMut};
use dashmap::DashMap;
use futures::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tokio_util::codec::{Decoder, Encoder, Framed};
use tracing::{debug, error, info, warn};

use crate::error::{RpcError, RpcFrameworkError, RpcResult};
use crate::event::{EventManager, EventPublisher};
use crate::protocol::{RpcHandler, RpcMessage, RpcRequest, RpcResponse, RpcResponseMessage, ServerMessage};

/// JSON-RPC TCP Codec for framing messages
#[derive(Debug, Clone)]
pub struct JsonRpcCodec;

impl Decoder for JsonRpcCodec {
    type Item = RpcMessage;
    type Error = RpcFrameworkError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.is_empty() {
            return Ok(None);
        }

        // 寻找换行符作为消息分隔符
        if let Some(newline_offset) = src.iter().position(|b| *b == b'\n') {
            let line = src.split_to(newline_offset + 1);
            let line = &line[..line.len() - 1]; // 移除换行符

            if line.is_empty() {
                return Ok(None);
            }

            let message: RpcMessage = serde_json::from_slice(line)?;
            Ok(Some(message))
        } else {
            Ok(None)
        }
    }
}

impl Encoder<ServerMessage> for JsonRpcCodec {
    type Error = RpcFrameworkError;

    fn encode(&mut self, item: ServerMessage, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let json = serde_json::to_vec(&item)?;
        dst.reserve(json.len() + 1);
        dst.put_slice(&json);
        dst.put_u8(b'\n'); // 添加换行符作为分隔符
        Ok(())
    }
}

/// RPC 服务端配置
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub bind_addr: SocketAddr,
    pub max_connections: usize,
    pub buffer_size: usize,
    pub request_timeout_ms: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_addr: "127.0.0.1:8000".parse().unwrap(),
            max_connections: 1000,
            buffer_size: 8192,
            request_timeout_ms: 30000, // 30 seconds
        }
    }
}

/// 连接状态
#[derive(Debug)]
struct ConnectionState {
    addr: SocketAddr,
    connected_at: std::time::Instant,
}

/// RPC 服务端
pub struct RpcServer {
    config: ServerConfig,
    handlers: Arc<DashMap<String, Arc<dyn RpcHandler>>>,
    connections: Arc<DashMap<String, ConnectionState>>,
    stats: Arc<RwLock<ServerStats>>,
    event_manager: Arc<EventManager>,
}

/// 服务端统计信息
#[derive(Debug, Default, Clone)]
pub struct ServerStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub active_connections: u64,
    pub total_connections: u64,
}

impl RpcServer {
    /// 创建新的 RPC 服务端
    pub fn new(config: ServerConfig) -> Self {
        let server = Self {
            config,
            handlers: Arc::new(DashMap::new()),
            connections: Arc::new(DashMap::new()),
            stats: Arc::new(RwLock::new(ServerStats::default())),
            event_manager: Arc::new(EventManager::new()),
        };
        
        // 注册内置方法
        server.register_builtin_methods();
        server
    }

    /// 获取事件管理器
    pub fn event_manager(&self) -> &Arc<EventManager> {
        &self.event_manager
    }

    /// 获取事件发布者
    pub fn event_publisher(&self) -> &EventPublisher {
        self.event_manager.publisher()
    }

    /// 注册 RPC 处理器
    pub fn register_handler(&self, handler: Arc<dyn RpcHandler>) {
        for method in handler.methods() {
            info!("Registering RPC method: {}", method);
            self.handlers.insert(method, handler.clone());
        }
    }

    /// 移除处理器
    pub fn unregister_method(&self, method: &str) {
        if self.handlers.remove(method).is_some() {
            info!("Unregistered RPC method: {}", method);
        }
    }

    /// 获取已注册的方法列表
    pub fn registered_methods(&self) -> Vec<String> {
        self.handlers.iter().map(|entry| entry.key().clone()).collect()
    }

    /// 启动服务端
    pub async fn serve(self: Arc<Self>) -> RpcResult<()> {
        let listener = TcpListener::bind(self.config.bind_addr).await?;
        info!("RPC Server listening on: {}", self.config.bind_addr);

        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    debug!("New connection from: {}", addr);
                    
                    // 检查连接数限制
                    {
                        let stats = self.stats.read().await;
                        if stats.active_connections >= self.config.max_connections as u64 {
                            warn!("Max connections reached, rejecting connection from: {}", addr);
                            drop(stream);
                            continue;
                        }
                    }

                    let server = self.clone();
                    tokio::spawn(async move {
                        if let Err(e) = server.handle_connection(stream, addr).await {
                            error!("Connection error for {}: {}", addr, e);
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    }

    /// 处理单个连接
    async fn handle_connection(&self, stream: TcpStream, addr: SocketAddr) -> RpcResult<()> {
        let conn_id = format!("{}-{}", addr, uuid::Uuid::new_v4());
        
        // 记录连接
        self.connections.insert(
            conn_id.clone(),
            ConnectionState {
                addr,
                connected_at: std::time::Instant::now(),
            },
        );

        // 更新统计
        {
            let mut stats = self.stats.write().await;
            stats.active_connections += 1;
            stats.total_connections += 1;
        }

        let mut framed = Framed::new(stream, JsonRpcCodec);

        // 处理消息循环
        let result = loop {
            match framed.next().await {
                Some(Ok(message)) => {
                    debug!("Received message from {}: {:?}", addr, message);
                    
                    match self.process_message(message).await {
                        Ok(Some(response)) => {
                            if let Err(e) = framed.send(response).await {
                                error!("Failed to send response to {}: {}", addr, e);
                                break Err(e);
                            }
                        }
                        Ok(None) => {
                            // 通知消息，无需响应
                        }
                        Err(e) => {
                            error!("Error processing message from {}: {}", addr, e);
                            break Err(e);
                        }
                    }
                }
                Some(Err(e)) => {
                    error!("Codec error for {}: {}", addr, e);
                    break Err(e);
                }
                None => {
                    debug!("Connection closed by {}", addr);
                    break Ok(());
                }
            }
        };

        // 清理连接
        self.connections.remove(&conn_id);
        {
            let mut stats = self.stats.write().await;
            stats.active_connections -= 1;
        }

        result
    }

    /// 处理 RPC 消息
    async fn process_message(&self, message: RpcMessage) -> RpcResult<Option<ServerMessage>> {
        // 验证消息
        if let Err(e) = message.validate() {
            return Ok(Some(ServerMessage::Response(RpcResponseMessage::Single(RpcResponse::error(
                Value::Null,
                e,
            )))));
        }

        match message {
            RpcMessage::Single(request) => {
                let response = self.process_request(request).await;
                Ok(response.map(|r| ServerMessage::Response(RpcResponseMessage::Single(r))))
            }
            RpcMessage::Batch(requests) => {
                let mut responses = Vec::new();
                
                for request in requests {
                    if let Some(response) = self.process_request(request).await {
                        responses.push(response);
                    }
                }

                if responses.is_empty() {
                    Ok(None) // 所有都是通知
                } else {
                    Ok(Some(ServerMessage::Response(RpcResponseMessage::Batch(responses))))
                }
            }
        }
    }

    /// 处理单个请求
    async fn process_request(&self, request: RpcRequest) -> Option<RpcResponse> {
        // 更新统计
        {
            let mut stats = self.stats.write().await;
            stats.total_requests += 1;
        }

        // 通知请求不需要响应
        if request.is_notification() {
            self.execute_method(&request.method, request.params).await;
            return None;
        }

        let id = request.get_id().cloned().unwrap_or(Value::Null);

        match self.execute_method(&request.method, request.params).await {
            Ok(result) => {
                {
                    let mut stats = self.stats.write().await;
                    stats.successful_requests += 1;
                }
                Some(RpcResponse::success(id, result))
            }
            Err(error) => {
                {
                    let mut stats = self.stats.write().await;
                    stats.failed_requests += 1;
                }
                Some(RpcResponse::error(id, error))
            }
        }
    }

    /// 执行方法调用
    async fn execute_method(&self, method: &str, params: Option<Value>) -> Result<Value, RpcError> {
        if let Some(handler) = self.handlers.get(method) {
            handler.handle(method, params).await
        } else {
            Err(RpcError::method_not_found(method))
        }
    }

    /// 注册内置方法
    fn register_builtin_methods(&self) {
        use crate::protocol::FunctionHandler;

        // rpc.discover - 服务发现
        let discover_handler = FunctionHandler::new(
            "rpc.discover".to_string(),
            {
                let handlers = self.handlers.clone();
                move |_params| {
                    let handlers = handlers.clone();
                    Box::pin(async move {
                        let methods: Vec<String> = handlers.iter()
                            .map(|entry| entry.key().clone())
                            .collect();
                        Ok(serde_json::json!({
                            "methods": methods,
                            "version": "1.0.0",
                            "description": "Stepflow RPC Server"
                        }))
                    })
                }
            }
        );
        self.register_handler(Arc::new(discover_handler));

        // rpc.ping - 健康检查
        let ping_handler = FunctionHandler::new(
            "rpc.ping".to_string(),
            |_params| {
                Box::pin(async move {
                    Ok(serde_json::json!({
                        "pong": true,
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    }))
                })
            }
        );
        self.register_handler(Arc::new(ping_handler));

        // rpc.stats - 服务器统计
        let stats_handler = FunctionHandler::new(
            "rpc.stats".to_string(),
            {
                let stats = self.stats.clone();
                let connections = self.connections.clone();
                move |_params| {
                    let stats = stats.clone();
                    let connections = connections.clone();
                    Box::pin(async move {
                        let stats = stats.read().await;
                        Ok(serde_json::json!({
                            "total_requests": stats.total_requests,
                            "successful_requests": stats.successful_requests,
                            "failed_requests": stats.failed_requests,
                            "active_connections": stats.active_connections,
                            "total_connections": stats.total_connections,
                            "connection_count": connections.len()
                        }))
                    })
                }
            }
        );
        self.register_handler(Arc::new(stats_handler));
    }

    /// 获取服务器统计信息
    pub async fn get_stats(&self) -> ServerStats {
        self.stats.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tokio_test;

    #[tokio::test]
    async fn test_server_creation() {
        let config = ServerConfig::default();
        let server = RpcServer::new(config);
        assert!(server.registered_methods().len() >= 3); // 内置方法
    }

    #[test]
    fn test_codec() {
        let mut codec = JsonRpcCodec;
        
        // 测试解码 RpcMessage（客户端发送到服务器）
        let mut buf = BytesMut::new();
        let request = RpcRequest::new("test_method".to_string(), Some(json!({"param": "value"})));
        let message = RpcMessage::Single(request);
        let json_data = serde_json::to_vec(&message).unwrap();
        buf.extend_from_slice(&json_data);
        buf.put_u8(b'\n');
        
        let decoded = codec.decode(&mut buf).unwrap();
        assert!(decoded.is_some());
        if let Some(RpcMessage::Single(req)) = decoded {
            assert_eq!(req.method, "test_method");
        }
        
        // 测试编码 ServerMessage（服务器发送到客户端）
        let mut buf2 = BytesMut::new();
        let response = ServerMessage::Response(RpcResponseMessage::Single(RpcResponse::success(json!(1), json!("test"))));
        codec.encode(response, &mut buf2).unwrap();
        assert!(!buf2.is_empty());
        
        // 验证编码的数据是有效的JSON
        let json_str = std::str::from_utf8(&buf2[..buf2.len()-1]).unwrap(); // 移除换行符
        let _: serde_json::Value = serde_json::from_str(json_str).unwrap();
    }
} 