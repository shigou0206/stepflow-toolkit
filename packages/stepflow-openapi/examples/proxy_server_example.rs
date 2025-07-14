use std::collections::HashMap;
use stepflow_openapi::proxy::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化简单日志（可选）
    // tracing_subscriber::init();

    println!("=== StepFlow OpenAPI JSON RPC Proxy Server Example ===\n");

    // 创建示例配置
    let config = create_example_config();
    
    // 显示配置信息
    println!("📋 Proxy Configuration:");
    println!("   Listen Address: {}:{}", config.listen_addr, config.listen_port);
    println!("   Default Server: {}", config.default_server);
    println!("   Methods Mapped: {}", config.method_mappings.len());
    
    for (method, mapping) in &config.method_mappings {
        println!("     • {} -> {} {}", method, mapping.http_method, mapping.http_path);
    }
    println!();

    // 创建配置管理器
    let config_manager = match ConfigManager::new(config) {
        Ok(manager) => {
            println!("✅ Configuration loaded successfully");
            manager
        }
        Err(e) => {
            eprintln!("❌ Failed to load configuration: {}", e);
            return Err(e.into());
        }
    };

    // 创建代理服务器
    let server = match JsonRpcProxyServer::new(config_manager) {
        Ok(server) => {
            println!("✅ Proxy server created successfully");
            server
        }
        Err(e) => {
            eprintln!("❌ Failed to create proxy server: {}", e);
            return Err(e.into());
        }
    };

    println!("\n🚀 Starting JSON RPC Proxy Server...");
    println!("📡 You can now send JSON RPC requests to: http://127.0.0.1:8080");
    println!("\n📖 Example JSON RPC requests:");
    println!("   Get User:");
    println!("   {{\n     \"jsonrpc\": \"2.0\",\n     \"method\": \"getUser\",\n     \"params\": {{\"userId\": 123}},\n     \"id\": 1\n   }}");
    println!("\n   Create User:");
    println!("   {{\n     \"jsonrpc\": \"2.0\",\n     \"method\": \"createUser\",\n     \"params\": {{\"name\": \"John\", \"email\": \"john@example.com\"}},\n     \"id\": 2\n   }}");
    println!("\n⚠️  Note: Make sure your target API server is running at http://localhost:3000");
    println!("🔄 Press Ctrl+C to stop the server\n");

    // 启动服务器
    if let Err(e) = server.start().await {
        eprintln!("❌ Server error: {}", e);
        return Err(e.into());
    }

    Ok(())
}

/// 创建示例配置
fn create_example_config() -> ProxyConfig {
    let mut method_mappings = HashMap::new();
    
    // 用户管理 API 映射
    let mut get_user_mapping = MethodMapping {
        rpc_method: "getUser".to_string(),
        http_method: "GET".to_string(),
        http_path: "/api/users/{id}".to_string(),
        parameter_mapping: ParameterMapping::default(),
        target_server: None,
    };
    get_user_mapping.parameter_mapping.path_params.insert("userId".to_string(), "id".to_string());
    method_mappings.insert("getUser".to_string(), get_user_mapping);

    let mut create_user_mapping = MethodMapping {
        rpc_method: "createUser".to_string(),
        http_method: "POST".to_string(),
        http_path: "/api/users".to_string(),
        parameter_mapping: ParameterMapping::default(),
        target_server: None,
    };
    create_user_mapping.parameter_mapping.body_params.insert("name".to_string(), "name".to_string());
    create_user_mapping.parameter_mapping.body_params.insert("email".to_string(), "email".to_string());
    method_mappings.insert("createUser".to_string(), create_user_mapping);

    let mut update_user_mapping = MethodMapping {
        rpc_method: "updateUser".to_string(),
        http_method: "PUT".to_string(),
        http_path: "/api/users/{id}".to_string(),
        parameter_mapping: ParameterMapping::default(),
        target_server: None,
    };
    update_user_mapping.parameter_mapping.path_params.insert("userId".to_string(), "id".to_string());
    update_user_mapping.parameter_mapping.body_params.insert("name".to_string(), "name".to_string());
    update_user_mapping.parameter_mapping.body_params.insert("email".to_string(), "email".to_string());
    method_mappings.insert("updateUser".to_string(), update_user_mapping);

    let mut delete_user_mapping = MethodMapping {
        rpc_method: "deleteUser".to_string(),
        http_method: "DELETE".to_string(),
        http_path: "/api/users/{id}".to_string(),
        parameter_mapping: ParameterMapping::default(),
        target_server: None,
    };
    delete_user_mapping.parameter_mapping.path_params.insert("userId".to_string(), "id".to_string());
    method_mappings.insert("deleteUser".to_string(), delete_user_mapping);

    // 列表查询映射
    let mut list_users_mapping = MethodMapping {
        rpc_method: "listUsers".to_string(),
        http_method: "GET".to_string(),
        http_path: "/api/users".to_string(),
        parameter_mapping: ParameterMapping::default(),
        target_server: None,
    };
    list_users_mapping.parameter_mapping.query_params.insert("page".to_string(), "page".to_string());
    list_users_mapping.parameter_mapping.query_params.insert("limit".to_string(), "limit".to_string());
    list_users_mapping.parameter_mapping.query_params.insert("search".to_string(), "search".to_string());
    method_mappings.insert("listUsers".to_string(), list_users_mapping);

    ProxyConfig {
        listen_addr: "127.0.0.1".to_string(),
        listen_port: 8080,
        openapi_spec: OpenApiSpec {
            content: include_str!("../examples/sample_openapi.json").to_string(),
            format: OpenApiFormat::Json,
            parsed: None,
        },
        method_mappings,
        default_server: "http://localhost:3000".to_string(),
    }
} 