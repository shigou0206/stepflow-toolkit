use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::cst::{CstParser, SourceType};
use super::error::{ProxyError, ProxyResult};

/// 代理服务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// 服务监听地址
    pub listen_addr: String,
    /// 服务监听端口
    pub listen_port: u16,
    /// OpenAPI 规范文档
    pub openapi_spec: OpenApiSpec,
    /// 方法映射配置
    pub method_mappings: HashMap<String, MethodMapping>,
    /// 默认的目标服务器
    pub default_server: String,
}

/// OpenAPI 规范配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenApiSpec {
    /// OpenAPI 文档内容（YAML 或 JSON）
    pub content: String,
    /// 文档格式
    pub format: OpenApiFormat,
    /// 解析后的文档结构
    #[serde(skip)]
    pub parsed: Option<Value>,
}

/// OpenAPI 文档格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OpenApiFormat {
    Json,
    Yaml,
}

/// 方法映射配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodMapping {
    /// JSON RPC 方法名
    pub rpc_method: String,
    /// HTTP 方法（GET, POST, PUT, DELETE 等）
    pub http_method: String,
    /// HTTP 路径
    pub http_path: String,
    /// 参数映射规则
    pub parameter_mapping: ParameterMapping,
    /// 目标服务器（如果不同于默认服务器）
    pub target_server: Option<String>,
}

/// 参数映射配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterMapping {
    /// 路径参数映射
    pub path_params: HashMap<String, String>,
    /// 查询参数映射
    pub query_params: HashMap<String, String>,
    /// 请求体参数映射
    pub body_params: HashMap<String, String>,
    /// 头部参数映射
    pub header_params: HashMap<String, String>,
}

/// 配置管理器
pub struct ConfigManager {
    config: ProxyConfig,
}

impl ConfigManager {
    /// 创建新的配置管理器
    pub fn new(config: ProxyConfig) -> ProxyResult<Self> {
        let mut manager = Self { config };
        manager.parse_openapi_spec()?;
        Ok(manager)
    }

    /// 从文件加载配置
    pub fn from_file(path: &str) -> ProxyResult<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ProxyError::OpenApiParseError(format!("Failed to read config file: {}", e)))?;
        
        let config: ProxyConfig = if path.ends_with(".yaml") || path.ends_with(".yml") {
            serde_yaml::from_str(&content)
                .map_err(|e| ProxyError::OpenApiParseError(format!("Failed to parse YAML config: {}", e)))?
        } else {
            serde_json::from_str(&content)
                .map_err(|e| ProxyError::OpenApiParseError(format!("Failed to parse JSON config: {}", e)))?
        };

        Self::new(config)
    }

    /// 解析 OpenAPI 规范
    fn parse_openapi_spec(&mut self) -> ProxyResult<()> {
        let source_type = match self.config.openapi_spec.format {
            OpenApiFormat::Json => SourceType::Json,
            OpenApiFormat::Yaml => SourceType::Yaml,
        };

        // 使用我们的 CST 解析器解析 OpenAPI 文档
        let cst = CstParser::parse_as(&self.config.openapi_spec.content, source_type);
        
        // 将 CST 转换为 JSON 值
        let parsed_value = self.cst_to_json_value(&cst)?;
        
        // 验证是否为有效的 OpenAPI 文档
        self.validate_openapi_spec(&parsed_value)?;
        
        self.config.openapi_spec.parsed = Some(parsed_value);
        Ok(())
    }

    /// 将 CST 转换为 JSON 值
    fn cst_to_json_value(&self, cst: &crate::TreeCursorSyntaxNode) -> ProxyResult<Value> {
        // 这里应该使用 AST fold 功能来转换
        // 暂时先简单解析 JSON 字符串
        let text = cst.text();
        serde_json::from_str(&text)
            .map_err(|e| ProxyError::OpenApiParseError(format!("Failed to convert CST to JSON: {}", e)))
    }

    /// 验证 OpenAPI 规范
    fn validate_openapi_spec(&self, spec: &Value) -> ProxyResult<()> {
        // 检查必需的字段
        if !spec.is_object() {
            return Err(ProxyError::OpenApiParseError("OpenAPI spec must be an object".to_string()));
        }

        let obj = spec.as_object().unwrap();
        
        // 检查 openapi 版本
        if !obj.contains_key("openapi") {
            return Err(ProxyError::OpenApiParseError("Missing 'openapi' field".to_string()));
        }

        // 检查 paths
        if !obj.contains_key("paths") {
            return Err(ProxyError::OpenApiParseError("Missing 'paths' field".to_string()));
        }

        Ok(())
    }

    /// 获取配置
    pub fn config(&self) -> &ProxyConfig {
        &self.config
    }

    /// 获取解析后的 OpenAPI 规范
    pub fn openapi_spec(&self) -> ProxyResult<&Value> {
        self.config.openapi_spec.parsed.as_ref()
            .ok_or_else(|| ProxyError::OpenApiParseError("OpenAPI spec not parsed".to_string()))
    }

    /// 根据 JSON RPC 方法名查找映射配置
    pub fn find_method_mapping(&self, method_name: &str) -> Option<&MethodMapping> {
        self.config.method_mappings.get(method_name)
    }

    /// 根据 HTTP 路径和方法查找 OpenAPI 操作
    pub fn find_openapi_operation(&self, http_method: &str, path: &str) -> ProxyResult<Option<&Value>> {
        let spec = self.openapi_spec()?;
        let paths = spec.get("paths")
            .and_then(|p| p.as_object())
            .ok_or_else(|| ProxyError::OpenApiParseError("Invalid paths object".to_string()))?;

        if let Some(path_item) = paths.get(path) {
            if let Some(operation) = path_item.get(http_method.to_lowercase()) {
                return Ok(Some(operation));
            }
        }

        Ok(None)
    }

    /// 获取服务器列表
    pub fn get_servers(&self) -> ProxyResult<Vec<String>> {
        let spec = self.openapi_spec()?;
        
        if let Some(servers) = spec.get("servers").and_then(|s| s.as_array()) {
            let mut server_urls = Vec::new();
            for server in servers {
                if let Some(url) = server.get("url").and_then(|u| u.as_str()) {
                    server_urls.push(url.to_string());
                }
            }
            Ok(server_urls)
        } else {
            // 如果没有定义服务器，使用默认服务器
            Ok(vec![self.config.default_server.clone()])
        }
    }
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            listen_addr: "127.0.0.1".to_string(),
            listen_port: 8080,
            openapi_spec: OpenApiSpec {
                content: "{}".to_string(),
                format: OpenApiFormat::Json,
                parsed: None,
            },
            method_mappings: HashMap::new(),
            default_server: "http://localhost:3000".to_string(),
        }
    }
}

impl Default for ParameterMapping {
    fn default() -> Self {
        Self {
            path_params: HashMap::new(),
            query_params: HashMap::new(),
            body_params: HashMap::new(),
            header_params: HashMap::new(),
        }
    }
} 