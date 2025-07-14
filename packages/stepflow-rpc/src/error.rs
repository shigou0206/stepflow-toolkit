//! JSON-RPC Error Handling

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// JSON-RPC 2.0 标准错误代码
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ErrorCode {
    ParseError = -32700,
    InvalidRequest = -32600,
    MethodNotFound = -32601,
    InvalidParams = -32602,
    InternalError = -32603,
    // ServerError 使用函数而不是变体
}

impl ErrorCode {
    pub fn code(&self) -> i32 {
        match self {
            ErrorCode::ParseError => -32700,
            ErrorCode::InvalidRequest => -32600,
            ErrorCode::MethodNotFound => -32601,
            ErrorCode::InvalidParams => -32602,
            ErrorCode::InternalError => -32603,
        }
    }

    pub fn message(&self) -> &'static str {
        match self {
            ErrorCode::ParseError => "Parse error",
            ErrorCode::InvalidRequest => "Invalid Request",
            ErrorCode::MethodNotFound => "Method not found",
            ErrorCode::InvalidParams => "Invalid params",
            ErrorCode::InternalError => "Internal error",
        }
    }

    /// 创建服务器错误
    pub fn server_error(code: i32) -> i32 {
        if code >= -32099 && code <= -32000 {
            code
        } else {
            -32000 // 默认服务器错误
        }
    }
}

/// JSON-RPC 错误对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl RpcError {
    pub fn new(code: ErrorCode, data: Option<serde_json::Value>) -> Self {
        Self {
            code: code.code(),
            message: code.message().to_string(),
            data,
        }
    }

    pub fn parse_error() -> Self {
        Self::new(ErrorCode::ParseError, None)
    }

    pub fn invalid_request() -> Self {
        Self::new(ErrorCode::InvalidRequest, None)
    }

    pub fn method_not_found(method: &str) -> Self {
        Self::new(
            ErrorCode::MethodNotFound,
            Some(serde_json::json!({"method": method})),
        )
    }

    pub fn invalid_params(message: &str) -> Self {
        Self::new(
            ErrorCode::InvalidParams,
            Some(serde_json::json!({"message": message})),
        )
    }

    pub fn internal_error(message: &str) -> Self {
        Self::new(
            ErrorCode::InternalError,
            Some(serde_json::json!({"message": message})),
        )
    }

    pub fn server_error(code: i32, message: &str) -> Self {
        let validated_code = ErrorCode::server_error(code);
        Self {
            code: validated_code,
            message: message.to_string(),
            data: None,
        }
    }
}

/// RPC 框架错误类型
#[derive(Error, Debug)]
pub enum RpcFrameworkError {
    #[error("JSON parse error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("RPC error: {0:?}")]
    RpcError(RpcError),

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Timeout error: {0}")]
    TimeoutError(String),

    #[error("Service not found: {0}")]
    ServiceNotFound(String),

    #[error("Invalid address: {0}")]
    InvalidAddress(String),
}

impl From<RpcError> for RpcFrameworkError {
    fn from(error: RpcError) -> Self {
        RpcFrameworkError::RpcError(error)
    }
}

pub type RpcResult<T> = Result<T, RpcFrameworkError>; 