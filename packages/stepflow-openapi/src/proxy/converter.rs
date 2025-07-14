use std::collections::HashMap;
use serde_json::Value;
use serde::{Serialize, Deserialize};
use super::config::{MethodMapping, ParameterMapping};
use super::error::{ProxyError, ProxyResult};

/// HTTP 请求数据
#[derive(Debug, Clone)]
pub struct HttpRequest {
    /// HTTP 方法
    pub method: String,
    /// 请求路径
    pub path: String,
    /// 查询参数
    pub query_params: HashMap<String, String>,
    /// 路径参数
    pub path_params: HashMap<String, String>,
    /// 请求头
    pub headers: HashMap<String, String>,
    /// 请求体
    pub body: Option<Value>,
}

/// JSON RPC 请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// JSON RPC 版本
    pub jsonrpc: String,
    /// 方法名
    pub method: String,
    /// 参数
    pub params: Option<Value>,
    /// 请求ID
    pub id: Option<Value>,
}

/// JSON RPC 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// JSON RPC 版本
    pub jsonrpc: String,
    /// 结果
    pub result: Option<Value>,
    /// 错误
    pub error: Option<Value>,
    /// 请求ID
    pub id: Option<Value>,
}

/// HTTP 响应
#[derive(Debug, Clone)]
pub struct HttpResponse {
    /// 状态码
    pub status: u16,
    /// 响应头
    pub headers: HashMap<String, String>,
    /// 响应体
    pub body: Option<Value>,
}

/// 参数转换器
pub struct ParameterConverter;

impl ParameterConverter {
    /// 解析 JSON RPC 请求
    pub fn parse_json_rpc_request(request_body: &str) -> ProxyResult<JsonRpcRequest> {
        let value: Value = serde_json::from_str(request_body)
            .map_err(|e| ProxyError::JsonRpcParseError(format!("Invalid JSON: {}", e)))?;

        if !value.is_object() {
            return Err(ProxyError::InvalidRequest("Request must be an object".to_string()));
        }

        let obj = value.as_object().unwrap();

        // 检查 JSON RPC 版本
        let jsonrpc = obj.get("jsonrpc")
            .and_then(|v| v.as_str())
            .unwrap_or("2.0")
            .to_string();

        if jsonrpc != "2.0" {
            return Err(ProxyError::InvalidRequest("Unsupported JSON RPC version".to_string()));
        }

        // 获取方法名
        let method = obj.get("method")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ProxyError::InvalidRequest("Missing method field".to_string()))?
            .to_string();

        // 获取参数
        let params = obj.get("params").cloned();

        // 获取ID
        let id = obj.get("id").cloned();

        Ok(JsonRpcRequest {
            jsonrpc,
            method,
            params,
            id,
        })
    }

    /// 将 JSON RPC 请求转换为 HTTP 请求
    pub fn convert_to_http_request(
        rpc_request: &JsonRpcRequest,
        mapping: &MethodMapping,
    ) -> ProxyResult<HttpRequest> {
        let mut http_request = HttpRequest {
            method: mapping.http_method.clone(),
            path: mapping.http_path.clone(),
            query_params: HashMap::new(),
            path_params: HashMap::new(),
            headers: HashMap::new(),
            body: None,
        };

        // 如果有参数，进行转换
        if let Some(params) = &rpc_request.params {
            Self::convert_parameters(params, &mapping.parameter_mapping, &mut http_request)?;
        }

        // 处理路径参数替换
        Self::substitute_path_parameters(&mut http_request)?;

        Ok(http_request)
    }

    /// 转换参数
    fn convert_parameters(
        params: &Value,
        mapping: &ParameterMapping,
        http_request: &mut HttpRequest,
    ) -> ProxyResult<()> {
        match params {
            Value::Object(param_obj) => {
                for (key, value) in param_obj {
                    Self::map_parameter(key, value, mapping, http_request)?;
                }
            }
            Value::Array(param_array) => {
                // 对于数组参数，使用索引作为键
                for (index, value) in param_array.iter().enumerate() {
                    let key = index.to_string();
                    Self::map_parameter(&key, value, mapping, http_request)?;
                }
            }
            _ => {
                // 单个值参数，作为请求体
                http_request.body = Some(params.clone());
            }
        }
        Ok(())
    }

    /// 映射单个参数
    fn map_parameter(
        key: &str,
        value: &Value,
        mapping: &ParameterMapping,
        http_request: &mut HttpRequest,
    ) -> ProxyResult<()> {
        let value_str = match value {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
            _ => serde_json::to_string(value)
                .map_err(|e| ProxyError::ParameterConversionError(format!("Failed to serialize parameter: {}", e)))?,
        };

        // 检查路径参数映射
        if let Some(path_param) = mapping.path_params.get(key) {
            http_request.path_params.insert(path_param.clone(), value_str);
            return Ok(());
        }

        // 检查查询参数映射
        if let Some(query_param) = mapping.query_params.get(key) {
            http_request.query_params.insert(query_param.clone(), value_str);
            return Ok(());
        }

        // 检查头部参数映射
        if let Some(header_param) = mapping.header_params.get(key) {
            http_request.headers.insert(header_param.clone(), value_str);
            return Ok(());
        }

        // 检查请求体参数映射
        if mapping.body_params.contains_key(key) {
            // 对于请求体参数，我们需要构建一个对象
            match &mut http_request.body {
                Some(Value::Object(body_obj)) => {
                    if let Some(body_key) = mapping.body_params.get(key) {
                        body_obj.insert(body_key.clone(), value.clone());
                    }
                }
                Some(_) => {
                    return Err(ProxyError::ParameterConversionError(
                        "Cannot mix body parameter types".to_string()
                    ));
                }
                None => {
                    let mut body_obj = serde_json::Map::new();
                    if let Some(body_key) = mapping.body_params.get(key) {
                        body_obj.insert(body_key.clone(), value.clone());
                    }
                    http_request.body = Some(Value::Object(body_obj));
                }
            }
            return Ok(());
        }

        // 如果没有明确的映射，默认作为查询参数
        http_request.query_params.insert(key.to_string(), value_str);
        Ok(())
    }

    /// 替换路径中的参数
    fn substitute_path_parameters(http_request: &mut HttpRequest) -> ProxyResult<()> {
        let mut path = http_request.path.clone();
        
        for (param_name, param_value) in &http_request.path_params {
            let placeholder = format!("{{{}}}", param_name);
            if path.contains(&placeholder) {
                path = path.replace(&placeholder, param_value);
            } else {
                return Err(ProxyError::ParameterConversionError(
                    format!("Path parameter '{}' not found in path template", param_name)
                ));
            }
        }
        
        http_request.path = path;
        Ok(())
    }

    /// 将 HTTP 响应转换为 JSON RPC 响应
    pub fn convert_to_json_rpc_response(
        http_response: &HttpResponse,
        request_id: Option<Value>,
    ) -> ProxyResult<JsonRpcResponse> {
        let mut rpc_response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: None,
            id: request_id,
        };

        if http_response.status >= 200 && http_response.status < 300 {
            // 成功响应
            rpc_response.result = http_response.body.clone();
        } else {
            // 错误响应
            let error_message = match &http_response.body {
                Some(body) => {
                    if let Some(msg) = body.get("message").and_then(|m| m.as_str()) {
                        msg.to_string()
                    } else {
                        format!("HTTP {} error", http_response.status)
                    }
                }
                None => format!("HTTP {} error", http_response.status),
            };

            rpc_response.error = Some(serde_json::json!({
                "code": Self::http_status_to_rpc_error_code(http_response.status),
                "message": error_message,
                "data": http_response.body
            }));
        }

        Ok(rpc_response)
    }

    /// 将 HTTP 状态码转换为 JSON RPC 错误码
    fn http_status_to_rpc_error_code(status: u16) -> i32 {
        match status {
            400 => super::error::json_rpc_errors::INVALID_PARAMS,
            404 => super::error::json_rpc_errors::METHOD_NOT_FOUND,
            500..=599 => super::error::json_rpc_errors::INTERNAL_ERROR,
            _ => super::error::json_rpc_errors::INTERNAL_ERROR,
        }
    }

    /// 序列化 JSON RPC 响应
    pub fn serialize_json_rpc_response(response: &JsonRpcResponse) -> ProxyResult<String> {
        serde_json::to_string(response)
            .map_err(|e| ProxyError::InternalError(format!("Failed to serialize response: {}", e)))
    }

    /// 构建 HTTP URL
    pub fn build_http_url(base_url: &str, http_request: &HttpRequest) -> String {
        let mut url = format!("{}{}", base_url.trim_end_matches('/'), http_request.path);
        
        if !http_request.query_params.is_empty() {
            let query_string: Vec<String> = http_request.query_params
                .iter()
                .map(|(k, v)| format!("{}={}", 
                    urlencoding::encode(k), 
                    urlencoding::encode(v)
                ))
                .collect();
            url.push('?');
            url.push_str(&query_string.join("&"));
        }
        
        url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_json_rpc_request() {
        let request_body = r#"
        {
            "jsonrpc": "2.0",
            "method": "getUser",
            "params": {"id": 123},
            "id": 1
        }
        "#;

        let request = ParameterConverter::parse_json_rpc_request(request_body).unwrap();
        assert_eq!(request.method, "getUser");
        assert_eq!(request.id, Some(Value::Number(serde_json::Number::from(1))));
    }

    #[test]
    fn test_convert_to_http_request() {
        let rpc_request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "getUser".to_string(),
            params: Some(serde_json::json!({"userId": 123, "format": "json"})),
            id: Some(Value::Number(serde_json::Number::from(1))),
        };

        let mut mapping = MethodMapping {
            rpc_method: "getUser".to_string(),
            http_method: "GET".to_string(),
            http_path: "/users/{id}".to_string(),
            parameter_mapping: ParameterMapping::default(),
            target_server: None,
        };

        mapping.parameter_mapping.path_params.insert("userId".to_string(), "id".to_string());
        mapping.parameter_mapping.query_params.insert("format".to_string(), "format".to_string());

        let http_request = ParameterConverter::convert_to_http_request(&rpc_request, &mapping).unwrap();
        
        assert_eq!(http_request.method, "GET");
        assert_eq!(http_request.path, "/users/123");
        assert_eq!(http_request.query_params.get("format"), Some(&"json".to_string()));
    }
} 