use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use super::config::ConfigManager;
use super::converter::{ParameterConverter, JsonRpcResponse};
use super::http_client::{HttpApiProxy, HttpClientConfig};
use super::error::{ProxyError, ProxyResult};

/// JSON RPC 代理服务器
pub struct JsonRpcProxyServer {
    config_manager: Arc<ConfigManager>,
    http_client: Arc<HttpApiProxy>,
    listen_addr: SocketAddr,
}

impl JsonRpcProxyServer {
    /// 创建新的代理服务器
    pub fn new(config_manager: ConfigManager) -> ProxyResult<Self> {
        let listen_addr = format!("{}:{}", 
            config_manager.config().listen_addr,
            config_manager.config().listen_port
        ).parse()
        .map_err(|e| ProxyError::InternalError(format!("Invalid listen address: {}", e)))?;

        let http_client = HttpApiProxy::new(HttpClientConfig::default())?;

        Ok(Self {
            config_manager: Arc::new(config_manager),
            http_client: Arc::new(http_client),
            listen_addr,
        })
    }

    /// 启动服务器
    pub async fn start(&self) -> ProxyResult<()> {
        let listener = TcpListener::bind(self.listen_addr)
            .await
            .map_err(|e| ProxyError::InternalError(format!("Failed to bind to {}: {}", self.listen_addr, e)))?;

        println!("JSON RPC Proxy Server listening on {}", self.listen_addr);

        // 进行健康检查
        if let Err(e) = self.perform_startup_checks().await {
            eprintln!("Startup checks failed: {}", e);
        }

        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    println!("New connection from: {}", addr);
                    
                    let config_manager = Arc::clone(&self.config_manager);
                    let http_client = Arc::clone(&self.http_client);
                    
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_connection(stream, config_manager, http_client).await {
                            eprintln!("Error handling connection from {}: {}", addr, e);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Failed to accept connection: {}", e);
                }
            }
        }
    }

    /// 处理单个连接
    async fn handle_connection(
        mut stream: TcpStream,
        config_manager: Arc<ConfigManager>,
        http_client: Arc<HttpApiProxy>,
    ) -> ProxyResult<()> {
        let mut buffer = vec![0; 8192];
        
        loop {
            match stream.read(&mut buffer).await {
                Ok(0) => {
                    // 连接关闭
                    break;
                }
                Ok(n) => {
                    let request_data = String::from_utf8_lossy(&buffer[..n]);
                    
                    // 处理可能的HTTP请求头
                    let json_body = Self::extract_json_from_request(&request_data)?;
                    
                    let response = Self::process_json_rpc_request(
                        &json_body,
                        &config_manager,
                        &http_client,
                    ).await;

                    let response_text = match response {
                        Ok(rpc_response) => {
                            ParameterConverter::serialize_json_rpc_response(&rpc_response)?
                        }
                        Err(e) => {
                            serde_json::to_string(&e.to_json_rpc_error(None))
                                .unwrap_or_else(|_| r#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"Internal error"},"id":null}"#.to_string())
                        }
                    };

                    // 构建 HTTP 响应
                    let http_response = Self::build_http_response(&response_text);
                    
                    if let Err(e) = stream.write_all(http_response.as_bytes()).await {
                        eprintln!("Failed to write response: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Failed to read from stream: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    /// 从HTTP请求中提取JSON内容
    fn extract_json_from_request(request_data: &str) -> ProxyResult<String> {
        // 检查是否是HTTP请求
        if request_data.starts_with("POST") || request_data.starts_with("GET") {
            // HTTP请求，需要提取body
            if let Some(body_start) = request_data.find("\r\n\r\n") {
                Ok(request_data[body_start + 4..].to_string())
            } else if let Some(body_start) = request_data.find("\n\n") {
                Ok(request_data[body_start + 2..].to_string())
            } else {
                Err(ProxyError::InvalidRequest("No body found in HTTP request".to_string()))
            }
        } else {
            // 直接的JSON-RPC请求
            Ok(request_data.to_string())
        }
    }

    /// 构建HTTP响应
    fn build_http_response(json_content: &str) -> String {
        format!(
            "HTTP/1.1 200 OK\r\n\
             Content-Type: application/json\r\n\
             Content-Length: {}\r\n\
             Access-Control-Allow-Origin: *\r\n\
             Access-Control-Allow-Methods: POST, GET, OPTIONS\r\n\
             Access-Control-Allow-Headers: Content-Type\r\n\
             \r\n\
             {}",
            json_content.len(),
            json_content
        )
    }

    /// 处理JSON RPC请求
    async fn process_json_rpc_request(
        request_body: &str,
        config_manager: &ConfigManager,
        http_client: &HttpApiProxy,
    ) -> ProxyResult<JsonRpcResponse> {
        // 解析JSON RPC请求
        let rpc_request = ParameterConverter::parse_json_rpc_request(request_body)?;

        // 查找方法映射
        let method_mapping = config_manager
            .find_method_mapping(&rpc_request.method)
            .ok_or_else(|| ProxyError::MethodNotFound(format!("Method '{}' not found", rpc_request.method)))?;

        // 转换为HTTP请求
        let http_request = ParameterConverter::convert_to_http_request(&rpc_request, method_mapping)?;

        // 确定目标服务器
        let target_server = method_mapping.target_server
            .as_ref()
            .unwrap_or(&config_manager.config().default_server);

        // 发送HTTP请求
        let http_response = http_client.send_request(target_server, &http_request).await?;

        // 转换回JSON RPC响应
        let rpc_response = ParameterConverter::convert_to_json_rpc_response(&http_response, rpc_request.id)?;

        Ok(rpc_response)
    }

    /// 启动时的健康检查
    async fn perform_startup_checks(&self) -> ProxyResult<()> {
        println!("Performing startup checks...");

        // 检查OpenAPI规范是否有效
        let _spec = self.config_manager.openapi_spec()?;
        println!("✓ OpenAPI specification is valid");

        // 检查服务器连接
        let servers = self.config_manager.get_servers()?;
        for server in &servers {
            match self.http_client.health_check(server).await {
                Ok(true) => println!("✓ Server {} is healthy", server),
                Ok(false) => println!("⚠ Server {} returned non-success status", server),
                Err(e) => println!("✗ Server {} health check failed: {}", server, e),
            }
        }

        // 尝试获取远程API信息
        for server in &servers {
            match self.http_client.get_api_info(server).await {
                Ok(Some(_)) => println!("✓ Found OpenAPI spec at {}", server),
                Ok(None) => println!("⚠ No OpenAPI spec found at {}", server),
                Err(e) => println!("✗ Failed to get API info from {}: {}", server, e),
            }
        }

        Ok(())
    }

    /// 创建示例配置
    pub fn create_example_config() -> super::config::ProxyConfig {
        use std::collections::HashMap;
        use super::config::*;

        let mut method_mappings = HashMap::new();
        
        // 示例方法映射：获取用户信息
        let mut get_user_mapping = MethodMapping {
            rpc_method: "getUser".to_string(),
            http_method: "GET".to_string(),
            http_path: "/users/{id}".to_string(),
            parameter_mapping: ParameterMapping::default(),
            target_server: None,
        };
        get_user_mapping.parameter_mapping.path_params.insert("userId".to_string(), "id".to_string());
        method_mappings.insert("getUser".to_string(), get_user_mapping);

        // 示例方法映射：创建用户
        let mut create_user_mapping = MethodMapping {
            rpc_method: "createUser".to_string(),
            http_method: "POST".to_string(),
            http_path: "/users".to_string(),
            parameter_mapping: ParameterMapping::default(),
            target_server: None,
        };
        create_user_mapping.parameter_mapping.body_params.insert("name".to_string(), "name".to_string());
        create_user_mapping.parameter_mapping.body_params.insert("email".to_string(), "email".to_string());
        method_mappings.insert("createUser".to_string(), create_user_mapping);

        ProxyConfig {
            listen_addr: "127.0.0.1".to_string(),
            listen_port: 8080,
            openapi_spec: OpenApiSpec {
                content: r#"{
                    "openapi": "3.0.0",
                    "info": {
                        "title": "Example API",
                        "version": "1.0.0"
                    },
                    "paths": {
                        "/users/{id}": {
                            "get": {
                                "parameters": [
                                    {
                                        "name": "id",
                                        "in": "path",
                                        "required": true,
                                        "schema": {"type": "integer"}
                                    }
                                ],
                                "responses": {
                                    "200": {
                                        "description": "User found",
                                        "content": {
                                            "application/json": {
                                                "schema": {"type": "object"}
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        "/users": {
                            "post": {
                                "requestBody": {
                                    "content": {
                                        "application/json": {
                                            "schema": {
                                                "type": "object",
                                                "properties": {
                                                    "name": {"type": "string"},
                                                    "email": {"type": "string"}
                                                }
                                            }
                                        }
                                    }
                                },
                                "responses": {
                                    "201": {
                                        "description": "User created"
                                    }
                                }
                            }
                        }
                    }
                }"#.to_string(),
                format: OpenApiFormat::Json,
                parsed: None,
            },
            method_mappings,
            default_server: "http://localhost:3000".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_from_http_request() {
        let http_request = "POST /jsonrpc HTTP/1.1\r\n\
                           Host: localhost:8080\r\n\
                           Content-Type: application/json\r\n\
                           Content-Length: 50\r\n\
                           \r\n\
                           {\"jsonrpc\":\"2.0\",\"method\":\"test\",\"id\":1}";
        
        let json_body = JsonRpcProxyServer::extract_json_from_request(http_request).unwrap();
        assert_eq!(json_body, "{\"jsonrpc\":\"2.0\",\"method\":\"test\",\"id\":1}");
    }

    #[test]
    fn test_extract_json_from_direct_request() {
        let direct_request = "{\"jsonrpc\":\"2.0\",\"method\":\"test\",\"id\":1}";
        
        let json_body = JsonRpcProxyServer::extract_json_from_request(direct_request).unwrap();
        assert_eq!(json_body, "{\"jsonrpc\":\"2.0\",\"method\":\"test\",\"id\":1}");
    }

    #[test]
    fn test_build_http_response() {
        let json_content = "{\"jsonrpc\":\"2.0\",\"result\":\"ok\",\"id\":1}";
        let response = JsonRpcProxyServer::build_http_response(json_content);
        
        assert!(response.contains("HTTP/1.1 200 OK"));
        assert!(response.contains("Content-Type: application/json"));
        assert!(response.contains("Access-Control-Allow-Origin: *"));
        assert!(response.contains(json_content));
    }

    #[test]
    fn test_create_example_config() {
        let config = JsonRpcProxyServer::create_example_config();
        
        assert_eq!(config.listen_port, 8080);
        assert!(config.method_mappings.contains_key("getUser"));
        assert!(config.method_mappings.contains_key("createUser"));
        
        let get_user = &config.method_mappings["getUser"];
        assert_eq!(get_user.http_method, "GET");
        assert_eq!(get_user.http_path, "/users/{id}");
    }
} 