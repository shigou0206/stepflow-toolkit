//! JSON-RPC 2.0 Protocol Implementation with Event Support

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use std::collections::HashMap;

use crate::error::RpcError;

/// JSON-RPC 2.0 请求对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
}

impl RpcRequest {
    /// 创建一个新的 RPC 请求
    pub fn new(method: String, params: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method,
            params,
            id: Some(Value::String(Uuid::new_v4().to_string())),
        }
    }

    /// 创建一个通知请求（没有 id，不期望响应）
    pub fn notification(method: String, params: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method,
            params,
            id: None,
        }
    }

    /// 判断是否为通知
    pub fn is_notification(&self) -> bool {
        self.id.is_none()
    }

    /// 获取请求 ID
    pub fn get_id(&self) -> Option<&Value> {
        self.id.as_ref()
    }

    /// 验证请求的有效性
    pub fn validate(&self) -> Result<(), RpcError> {
        if self.jsonrpc != "2.0" {
            return Err(RpcError::invalid_request());
        }
        
        if self.method.is_empty() {
            return Err(RpcError::invalid_request());
        }
        
        if self.method.starts_with("rpc.") && !self.method.starts_with("rpc.discover") 
            && !self.method.starts_with("rpc.ping") && !self.method.starts_with("rpc.stats") {
            return Err(RpcError::invalid_request());
        }
        
        Ok(())
    }
}

/// JSON-RPC 2.0 响应对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
    pub id: Value,
}

impl RpcResponse {
    /// 创建成功响应
    pub fn success(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    /// 创建错误响应
    pub fn error(id: Value, error: RpcError) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(error),
            id,
        }
    }

    /// 判断是否为成功响应
    pub fn is_success(&self) -> bool {
        self.error.is_none()
    }

    /// 判断是否为错误响应
    pub fn is_error(&self) -> bool {
        self.error.is_some()
    }
}

/// JSON-RPC 事件对象（服务器主动推送）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcEvent {
    pub jsonrpc: String,
    pub event: String,
    pub data: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sequence: Option<u64>,
}

impl RpcEvent {
    /// 创建新的事件
    pub fn new(event: String, data: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            event,
            data,
            timestamp: Some(chrono::Utc::now().to_rfc3339()),
            stream_id: None,
            sequence: None,
        }
    }

    /// 创建带流ID的事件
    pub fn with_stream(event: String, data: Value, stream_id: String) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            event,
            data,
            timestamp: Some(chrono::Utc::now().to_rfc3339()),
            stream_id: Some(stream_id),
            sequence: None,
        }
    }

    /// 创建带序列号的事件
    pub fn with_sequence(event: String, data: Value, stream_id: String, sequence: u64) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            event,
            data,
            timestamp: Some(chrono::Utc::now().to_rfc3339()),
            stream_id: Some(stream_id),
            sequence: Some(sequence),
        }
    }
}

/// RPC 消息类型（客户端发送）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RpcMessage {
    Single(RpcRequest),
    Batch(Vec<RpcRequest>),
}

impl RpcMessage {
    /// 获取所有请求
    pub fn requests(&self) -> Vec<&RpcRequest> {
        match self {
            RpcMessage::Single(req) => vec![req],
            RpcMessage::Batch(reqs) => reqs.iter().collect(),
        }
    }

    /// 验证消息的有效性
    pub fn validate(&self) -> Result<(), RpcError> {
        match self {
            RpcMessage::Single(req) => req.validate(),
            RpcMessage::Batch(reqs) => {
                if reqs.is_empty() {
                    return Err(RpcError::invalid_request());
                }
                for req in reqs {
                    req.validate()?;
                }
                Ok(())
            }
        }
    }
}

/// RPC 响应消息类型（服务器发送）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RpcResponseMessage {
    Single(RpcResponse),
    Batch(Vec<RpcResponse>),
}

/// 服务器向客户端发送的消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ServerMessage {
    Response(RpcResponseMessage),
    Event(RpcEvent),
}

/// 事件订阅信息
#[derive(Debug, Clone)]
pub struct EventSubscription {
    pub event_pattern: String,
    pub callback_id: String,
    pub filters: HashMap<String, Value>,
    pub created_at: std::time::Instant,
}

impl EventSubscription {
    /// 创建新的事件订阅
    pub fn new(event_pattern: String, callback_id: String) -> Self {
        Self {
            event_pattern,
            callback_id,
            filters: HashMap::new(),
            created_at: std::time::Instant::now(),
        }
    }

    /// 添加过滤器
    pub fn with_filter(mut self, key: String, value: Value) -> Self {
        self.filters.insert(key, value);
        self
    }

    /// 检查事件是否匹配订阅
    pub fn matches(&self, event: &RpcEvent) -> bool {
        // 简单的模式匹配，支持通配符 *
        if self.event_pattern == "*" {
            return true;
        }
        
        if self.event_pattern.ends_with("*") {
            let prefix = &self.event_pattern[..self.event_pattern.len() - 1];
            return event.event.starts_with(prefix);
        }
        
        event.event == self.event_pattern
    }
}

/// RPC 处理器 trait
#[async_trait::async_trait]
pub trait RpcHandler: Send + Sync {
    /// 处理 RPC 调用
    async fn handle(&self, method: &str, params: Option<Value>) -> Result<Value, RpcError>;

    /// 获取支持的方法列表
    fn methods(&self) -> Vec<String>;

    /// 获取方法描述
    fn describe(&self, method: &str) -> Option<String> {
        let _ = method;
        None
    }
}

/// 事件处理器 trait
#[async_trait::async_trait]
pub trait EventHandler: Send + Sync {
    /// 处理事件
    async fn handle_event(&self, event: &RpcEvent);
    
    /// 获取感兴趣的事件模式
    fn interested_events(&self) -> Vec<String>;
}

/// 简单的函数处理器
pub struct FunctionHandler {
    method: String,
    handler: Box<dyn Fn(Option<Value>) -> futures::future::BoxFuture<'static, Result<Value, RpcError>> + Send + Sync>,
}

impl FunctionHandler {
    /// 创建新的函数处理器
    pub fn new<F, Fut>(method: String, handler: F) -> Self
    where
        F: Fn(Option<Value>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<Value, RpcError>> + Send + 'static,
    {
        Self {
            method,
            handler: Box::new(move |params| Box::pin(handler(params))),
        }
    }
}

#[async_trait::async_trait]
impl RpcHandler for FunctionHandler {
    async fn handle(&self, method: &str, params: Option<Value>) -> Result<Value, RpcError> {
        if method != self.method {
            return Err(RpcError::method_not_found(method));
        }
        (self.handler)(params).await
    }

    fn methods(&self) -> Vec<String> {
        vec![self.method.clone()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_rpc_request_creation() {
        let req = RpcRequest::new("test_method".to_string(), Some(json!({"param": "value"})));
        assert_eq!(req.jsonrpc, "2.0");
        assert_eq!(req.method, "test_method");
        assert!(req.id.is_some());
        assert!(!req.is_notification());
    }

    #[test]
    fn test_rpc_notification() {
        let req = RpcRequest::notification("notify_method".to_string(), None);
        assert_eq!(req.jsonrpc, "2.0");
        assert_eq!(req.method, "notify_method");
        assert!(req.id.is_none());
        assert!(req.is_notification());
    }

    #[test]
    fn test_rpc_response_success() {
        let resp = RpcResponse::success(json!(1), json!({"result": "success"}));
        assert_eq!(resp.jsonrpc, "2.0");
        assert!(resp.is_success());
        assert!(!resp.is_error());
    }

    #[test]
    fn test_rpc_response_error() {
        let error = RpcError::method_not_found("unknown_method");
        let resp = RpcResponse::error(json!(1), error);
        assert_eq!(resp.jsonrpc, "2.0");
        assert!(!resp.is_success());
        assert!(resp.is_error());
    }
} 