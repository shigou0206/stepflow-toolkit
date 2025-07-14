use std::fmt;
use serde_json::Value;

/// 代理服务错误类型
#[derive(Debug, Clone)]
pub enum ProxyError {
    /// JSON RPC 解析错误
    JsonRpcParseError(String),
    /// OpenAPI 配置解析错误
    OpenApiParseError(String),
    /// HTTP 请求错误
    HttpRequestError(String),
    /// 参数转换错误
    ParameterConversionError(String),
    /// 方法不存在
    MethodNotFound(String),
    /// 内部服务器错误
    InternalError(String),
    /// 无效的请求格式
    InvalidRequest(String),
}

impl fmt::Display for ProxyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProxyError::JsonRpcParseError(msg) => write!(f, "JSON RPC parse error: {}", msg),
            ProxyError::OpenApiParseError(msg) => write!(f, "OpenAPI parse error: {}", msg),
            ProxyError::HttpRequestError(msg) => write!(f, "HTTP request error: {}", msg),
            ProxyError::ParameterConversionError(msg) => write!(f, "Parameter conversion error: {}", msg),
            ProxyError::MethodNotFound(msg) => write!(f, "Method not found: {}", msg),
            ProxyError::InternalError(msg) => write!(f, "Internal error: {}", msg),
            ProxyError::InvalidRequest(msg) => write!(f, "Invalid request: {}", msg),
        }
    }
}

impl std::error::Error for ProxyError {}

pub type ProxyResult<T> = Result<T, ProxyError>;

/// JSON RPC 错误码
pub mod json_rpc_errors {
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;
}

impl ProxyError {
    /// 转换为 JSON RPC 错误响应
    pub fn to_json_rpc_error(&self, id: Option<Value>) -> Value {
        use json_rpc_errors::*;
        
        let (code, message) = match self {
            ProxyError::JsonRpcParseError(_) => (PARSE_ERROR, "Parse error"),
            ProxyError::InvalidRequest(_) => (INVALID_REQUEST, "Invalid Request"),
            ProxyError::MethodNotFound(_) => (METHOD_NOT_FOUND, "Method not found"),
            ProxyError::ParameterConversionError(_) => (INVALID_PARAMS, "Invalid params"),
            _ => (INTERNAL_ERROR, "Internal error"),
        };

        serde_json::json!({
            "jsonrpc": "2.0",
            "error": {
                "code": code,
                "message": message,
                "data": self.to_string()
            },
            "id": id
        })
    }
} 