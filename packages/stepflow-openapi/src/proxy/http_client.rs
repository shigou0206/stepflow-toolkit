use std::time::Duration;
use serde_json::Value;
use super::converter::{HttpRequest, HttpResponse};
use super::error::{ProxyError, ProxyResult};

/// HTTP 客户端配置
#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    /// 请求超时时间（秒）
    pub timeout_seconds: u64,
    /// 最大重试次数
    pub max_retries: u32,
    /// 用户代理字符串
    pub user_agent: String,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 30,
            max_retries: 3,
            user_agent: "stepflow-openapi-proxy/1.0".to_string(),
        }
    }
}

/// HTTP API 代理客户端
pub struct HttpApiProxy {
    client: reqwest::Client,
    config: HttpClientConfig,
}

impl HttpApiProxy {
    /// 创建新的 HTTP 代理客户端
    pub fn new(config: HttpClientConfig) -> ProxyResult<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .user_agent(&config.user_agent)
            .build()
            .map_err(|e| ProxyError::HttpRequestError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { client, config })
    }

    /// 创建默认配置的 HTTP 代理客户端
    pub fn default() -> ProxyResult<Self> {
        Self::new(HttpClientConfig::default())
    }

    /// 发送 HTTP 请求
    pub async fn send_request(
        &self, 
        base_url: &str, 
        request: &HttpRequest
    ) -> ProxyResult<HttpResponse> {
        let url = super::converter::ParameterConverter::build_http_url(base_url, request);
        
        let mut retries = 0;
        loop {
            match self.try_send_request(&url, request).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    retries += 1;
                    if retries > self.config.max_retries {
                        return Err(e);
                    }
                    
                    // 只对特定类型的错误进行重试
                    if !self.should_retry(&e) {
                        return Err(e);
                    }
                    
                    // 简单的指数退避
                    let delay = Duration::from_millis(100 * (1 << retries.min(5)));
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }

    /// 尝试发送单次 HTTP 请求
    async fn try_send_request(&self, url: &str, request: &HttpRequest) -> ProxyResult<HttpResponse> {
        let mut req_builder = match request.method.to_uppercase().as_str() {
            "GET" => self.client.get(url),
            "POST" => self.client.post(url),
            "PUT" => self.client.put(url),
            "DELETE" => self.client.delete(url),
            "PATCH" => self.client.patch(url),
            "HEAD" => self.client.head(url),
            "OPTIONS" => {
                // reqwest 没有直接的 options 方法，使用 request
                self.client.request(reqwest::Method::OPTIONS, url)
            }
            _ => {
                return Err(ProxyError::HttpRequestError(
                    format!("Unsupported HTTP method: {}", request.method)
                ));
            }
        };

        // 添加头部
        for (key, value) in &request.headers {
            req_builder = req_builder.header(key, value);
        }

        // 添加请求体
        if let Some(body) = &request.body {
            req_builder = req_builder
                .header("Content-Type", "application/json")
                .json(body);
        }

        // 发送请求
        let response = req_builder
            .send()
            .await
            .map_err(|e| ProxyError::HttpRequestError(format!("Failed to send request: {}", e)))?;

        // 提取响应信息
        let status = response.status().as_u16();
        let mut headers = std::collections::HashMap::new();
        
        for (key, value) in response.headers() {
            if let Ok(value_str) = value.to_str() {
                headers.insert(key.to_string(), value_str.to_string());
            }
        }

        // 读取响应体
        let body_text = response
            .text()
            .await
            .map_err(|e| ProxyError::HttpRequestError(format!("Failed to read response body: {}", e)))?;

        let body = if body_text.is_empty() {
            None
        } else {
            // 尝试解析为 JSON，如果失败则保存为字符串
            match serde_json::from_str::<Value>(&body_text) {
                Ok(json) => Some(json),
                Err(_) => Some(Value::String(body_text)),
            }
        };

        Ok(HttpResponse {
            status,
            headers,
            body,
        })
    }

    /// 判断是否应该重试请求
    fn should_retry(&self, error: &ProxyError) -> bool {
        match error {
            ProxyError::HttpRequestError(msg) => {
                // 对网络错误和超时进行重试
                msg.contains("timeout") || 
                msg.contains("connection") || 
                msg.contains("network")
            }
            _ => false,
        }
    }

    /// 健康检查
    pub async fn health_check(&self, base_url: &str) -> ProxyResult<bool> {
        let health_url = format!("{}/health", base_url.trim_end_matches('/'));
        
        match self.client.get(&health_url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
        {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => {
                // 如果 /health 端点不存在，尝试根路径
                let root_url = base_url.trim_end_matches('/');
                match self.client.get(root_url)
                    .timeout(Duration::from_secs(5))
                    .send()
                    .await
                {
                    Ok(response) => Ok(response.status().as_u16() < 500),
                    Err(_) => Ok(false),
                }
            }
        }
    }

    /// 获取 API 信息（尝试获取 OpenAPI 规范）
    pub async fn get_api_info(&self, base_url: &str) -> ProxyResult<Option<Value>> {
        let common_openapi_paths = [
            "/openapi.json",
            "/openapi.yaml", 
            "/swagger.json",
            "/swagger.yaml",
            "/api-docs",
            "/docs/openapi.json",
        ];

        for path in &common_openapi_paths {
            let url = format!("{}{}", base_url.trim_end_matches('/'), path);
            
            if let Ok(response) = self.client.get(&url)
                .timeout(Duration::from_secs(10))
                .send()
                .await
            {
                if response.status().is_success() {
                    if let Ok(text) = response.text().await {
                        // 尝试解析为 JSON
                        if let Ok(json) = serde_json::from_str::<Value>(&text) {
                            return Ok(Some(json));
                        }
                        // 尝试解析为 YAML
                        if let Ok(yaml) = serde_yaml::from_str::<Value>(&text) {
                            return Ok(Some(yaml));
                        }
                    }
                }
            }
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_http_client_creation() {
        let client = HttpApiProxy::default();
        assert!(client.is_ok());
    }

    #[test]
    fn test_should_retry() {
        let client = HttpApiProxy::default().unwrap();
        
        let timeout_error = ProxyError::HttpRequestError("timeout occurred".to_string());
        assert!(client.should_retry(&timeout_error));
        
        let parse_error = ProxyError::JsonRpcParseError("invalid json".to_string());
        assert!(!client.should_retry(&parse_error));
    }

    #[test]
    fn test_build_url() {
        let request = HttpRequest {
            method: "GET".to_string(),
            path: "/users/123".to_string(),
            query_params: {
                let mut params = HashMap::new();
                params.insert("format".to_string(), "json".to_string());
                params.insert("include".to_string(), "profile".to_string());
                params
            },
            path_params: HashMap::new(),
            headers: HashMap::new(),
            body: None,
        };

        let url = super::super::converter::ParameterConverter::build_http_url("http://api.example.com", &request);
        assert!(url.contains("/users/123"));
        assert!(url.contains("format=json"));
        assert!(url.contains("include=profile"));
    }
} 