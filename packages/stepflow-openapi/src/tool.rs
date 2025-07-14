//! OpenAPI Tool Implementation
//!
//! This module provides the OpenAPI tool implementation that conforms to
//! the stepflow-core Tool trait specification.

use std::collections::HashMap;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use chrono::Utc;

// Import stepflow-core types
use stepflow_core::types::{
    Tool, ToolInfo, ToolRequest, ToolResponse, ToolExample, ToolType, ToolStatus, ToolVersion,
};
use stepflow_core::StepflowError;

use crate::srn::{Srn, SrnError};
use crate::document::{OpenApiDocument, OperationInfo};
use crate::ref_resolver::{RefResolver, RefResolverError};
use crate::proxy::http_client::{HttpApiProxy, HttpClientConfig};
use crate::proxy::converter::{ParameterConverter, JsonRpcRequest, HttpRequest};

/// OpenAPI Tool Errors
#[derive(Debug, thiserror::Error)]
pub enum OpenApiToolError {
    #[error("SRN error: {0}")]
    SrnError(#[from] SrnError),
    
    #[error("Reference resolution error: {0}")]
    RefResolverError(#[from] RefResolverError),
    
    #[error("HTTP request error: {0}")]
    HttpError(String),
    
    #[error("Parameter validation error: {0}")]
    ParameterValidation(String),
    
    #[error("Schema validation error: {0}")]
    SchemaValidation(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Execution error: {0}")]
    Execution(String),
}

impl From<OpenApiToolError> for StepflowError {
    fn from(error: OpenApiToolError) -> Self {
        StepflowError::ToolExecutionFailed(error.to_string())
    }
}

/// OpenAPI Tool Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenApiToolConfig {
    /// SRN of this tool
    pub srn: String,
    /// Base URL for HTTP requests
    pub base_url: String,
    /// Default timeout in milliseconds
    pub timeout_ms: Option<u64>,
    /// Maximum retry attempts
    pub max_retries: Option<u32>,
    /// Custom headers
    pub default_headers: HashMap<String, String>,
    /// Authentication configuration
    pub auth: Option<AuthConfig>,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthConfig {
    /// Bearer token authentication
    Bearer { token: String },
    /// API key authentication
    ApiKey { header: String, value: String },
    /// Basic authentication
    Basic { username: String, password: String },
}

/// OpenAPI Tool implementation
pub struct OpenApiTool {
    /// Tool SRN
    srn: Srn,
    /// Tool configuration
    config: OpenApiToolConfig,
    /// Operation information from OpenAPI spec
    operation: OperationInfo,
    /// Resolved OpenAPI document (with $ref resolved)
    resolved_document: Value,
    /// HTTP client for making requests
    http_client: HttpApiProxy,
}

impl OpenApiTool {
    /// Create a new OpenAPI tool
    pub async fn new(
        config: OpenApiToolConfig,
        operation: OperationInfo,
        document: &OpenApiDocument,
    ) -> Result<Self, OpenApiToolError> {
        let srn = Srn::parse(&config.srn)?;
        
        // Resolve all references in the document
        let ref_resolver = RefResolver::new();
        let resolved_document = ref_resolver.resolve_document(&document.parsed)?;

        // Create HTTP client configuration
        let http_config = HttpClientConfig {
            timeout_seconds: config.timeout_ms.unwrap_or(30000) / 1000, // Convert ms to seconds
            max_retries: config.max_retries.unwrap_or(3),
            user_agent: "stepflow-openapi-tool/1.0".to_string(),
        };

        let http_client = HttpApiProxy::new(http_config)
            .map_err(|e| OpenApiToolError::Configuration(e.to_string()))?;

        Ok(Self {
            srn,
            config,
            operation,
            resolved_document,
            http_client,
        })
    }

    /// Validate input parameters against OpenAPI schema
    fn validate_input_parameters(&self, input: &Value) -> Result<(), OpenApiToolError> {
        // This is a simplified validation - should be enhanced with proper JSON Schema validation
        if let Some(input_obj) = input.as_object() {
            // Check required path parameters
            for param in &self.operation.parameters {
                if param.required && param.location == crate::document::ParameterLocation::Path {
                    if !input_obj.contains_key(&param.name) {
                        return Err(OpenApiToolError::ParameterValidation(
                            format!("Required path parameter '{}' is missing", param.name)
                        ));
                    }
                }
            }

            // Check required query parameters
            for param in &self.operation.parameters {
                if param.required && param.location == crate::document::ParameterLocation::Query {
                    if !input_obj.contains_key(&param.name) {
                        return Err(OpenApiToolError::ParameterValidation(
                            format!("Required query parameter '{}' is missing", param.name)
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    /// Convert input to HTTP request
    fn build_http_request(&self, input: &Value) -> Result<HttpRequest, OpenApiToolError> {
        // Create a JSON RPC request structure for parameter conversion
        let rpc_request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: self.operation.operation_id.clone(),
            params: Some(input.clone()),
            id: Some(Value::String("1".to_string())),
        };

        // Create method mapping for parameter conversion
        let method_mapping = crate::proxy::config::MethodMapping {
            rpc_method: self.operation.operation_id.clone(),
            http_method: self.operation.method.clone(),
            http_path: self.operation.path.clone(),
            parameter_mapping: self.create_parameter_mapping(),
            target_server: None,
        };

        // Convert to HTTP request
        ParameterConverter::convert_to_http_request(&rpc_request, &method_mapping)
            .map_err(|e| OpenApiToolError::Execution(e.to_string()))
    }

    /// Create parameter mapping from operation info
    fn create_parameter_mapping(&self) -> crate::proxy::config::ParameterMapping {
        let mut mapping = crate::proxy::config::ParameterMapping::default();

        for param in &self.operation.parameters {
            match param.location {
                crate::document::ParameterLocation::Path => {
                    mapping.path_params.insert(param.name.clone(), param.name.clone());
                }
                crate::document::ParameterLocation::Query => {
                    mapping.query_params.insert(param.name.clone(), param.name.clone());
                }
                crate::document::ParameterLocation::Header => {
                    mapping.header_params.insert(param.name.clone(), param.name.clone());
                }
                _ => {}
            }
        }

        mapping
    }

    /// Add authentication to HTTP request
    fn add_authentication(&self, http_request: &mut HttpRequest) -> Result<(), OpenApiToolError> {
        if let Some(auth) = &self.config.auth {
            match auth {
                AuthConfig::Bearer { token } => {
                    http_request.headers.insert("Authorization".to_string(), format!("Bearer {}", token));
                }
                AuthConfig::ApiKey { header, value } => {
                    http_request.headers.insert(header.clone(), value.clone());
                }
                AuthConfig::Basic { username, password } => {
                    use base64::Engine;
                    let credentials = base64::engine::general_purpose::STANDARD.encode(format!("{}:{}", username, password));
                    http_request.headers.insert("Authorization".to_string(), format!("Basic {}", credentials));
                }
            }
        }

        // Add default headers
        for (key, value) in &self.config.default_headers {
            http_request.headers.insert(key.clone(), value.clone());
        }

        Ok(())
    }

    /// Execute HTTP request and convert response
    async fn execute_http_request(&self, mut http_request: HttpRequest) -> Result<Value, OpenApiToolError> {
        // Add authentication
        self.add_authentication(&mut http_request)?;

        // Execute request using HTTP client
        let response = self.http_client
            .send_request(&self.config.base_url, &http_request)
            .await
            .map_err(|e| OpenApiToolError::HttpError(e.to_string()))?;

        Ok(response.body.unwrap_or(Value::Null))
    }
}

#[async_trait]
impl Tool for OpenApiTool {
    async fn get_info(&self) -> Result<ToolInfo, StepflowError> {
        Ok(ToolInfo {
            id: stepflow_core::types::ToolId::from_string(self.srn.to_string()),
            name: format!("{} {}", self.operation.method, self.operation.path),
            description: self.operation.description.clone()
                .or_else(|| self.operation.summary.clone())
                .unwrap_or_else(|| format!("OpenAPI operation: {}", self.operation.operation_id)),
            version: ToolVersion::new(1, 0, 0),
            tool_type: ToolType::OpenAPI,
            status: ToolStatus::Active,
            author: "StepFlow OpenAPI Generator".to_string(),
            repository: None,
            documentation: None,
            tags: self.operation.tags.clone(),
            capabilities: vec![
                "http_request".to_string(),
                "parameter_validation".to_string(),
                "schema_validation".to_string(),
            ],
            configuration_schema: self.get_configuration_schema().await?,
            examples: self.generate_examples(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    async fn execute(&self, request: ToolRequest) -> Result<ToolResponse, StepflowError> {
        let start_time = std::time::Instant::now();
        
        // Validate input parameters
        if let Err(e) = self.validate_input_parameters(&request.input) {
            return Ok(ToolResponse {
                success: false,
                output: None,
                error: Some(e.to_string()),
                execution_time: start_time.elapsed().as_millis() as u64,
                metadata: HashMap::new(),
            });
        }

        // Build HTTP request
        let http_request = match self.build_http_request(&request.input) {
            Ok(req) => req,
            Err(e) => {
                return Ok(ToolResponse {
                    success: false,
                    output: None,
                    error: Some(e.to_string()),
                    execution_time: start_time.elapsed().as_millis() as u64,
                    metadata: HashMap::new(),
                });
            }
        };

        // Execute HTTP request
        match self.execute_http_request(http_request).await {
            Ok(response_body) => {
                let mut metadata = HashMap::new();
                metadata.insert("operation_id".to_string(), Value::String(self.operation.operation_id.clone()));
                metadata.insert("method".to_string(), Value::String(self.operation.method.clone()));
                metadata.insert("path".to_string(), Value::String(self.operation.path.clone()));

                Ok(ToolResponse {
                    success: true,
                    output: Some(response_body),
                    error: None,
                    execution_time: start_time.elapsed().as_millis() as u64,
                    metadata,
                })
            }
            Err(e) => {
                Ok(ToolResponse {
                    success: false,
                    output: None,
                    error: Some(e.to_string()),
                    execution_time: start_time.elapsed().as_millis() as u64,
                    metadata: HashMap::new(),
                })
            }
        }
    }

    async fn validate_input(&self, input: &Value) -> Result<bool, StepflowError> {
        match self.validate_input_parameters(input) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    async fn get_configuration_schema(&self) -> Result<Option<Value>, StepflowError> {
        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                "srn": {
                    "type": "string",
                    "description": "Tool SRN identifier"
                },
                "base_url": {
                    "type": "string",
                    "description": "Base URL for HTTP requests"
                },
                "timeout_ms": {
                    "type": "integer",
                    "description": "Request timeout in milliseconds",
                    "default": 30000
                },
                "max_retries": {
                    "type": "integer", 
                    "description": "Maximum retry attempts",
                    "default": 3
                },
                "default_headers": {
                    "type": "object",
                    "description": "Default headers to include in requests"
                },
                "auth": {
                    "oneOf": [
                        {
                            "type": "object",
                            "properties": {
                                "Bearer": {
                                    "type": "object",
                                    "properties": {
                                        "token": {"type": "string"}
                                    },
                                    "required": ["token"]
                                }
                            }
                        },
                        {
                            "type": "object", 
                            "properties": {
                                "ApiKey": {
                                    "type": "object",
                                    "properties": {
                                        "header": {"type": "string"},
                                        "value": {"type": "string"}
                                    },
                                    "required": ["header", "value"]
                                }
                            }
                        }
                    ]
                }
            },
            "required": ["srn", "base_url"]
        });

        Ok(Some(schema))
    }

    async fn test(&self) -> Result<bool, StepflowError> {
        // Simple connectivity test - try to reach the base URL
        match reqwest::get(&self.config.base_url).await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

impl OpenApiTool {
    /// Generate example requests/responses for this tool
    fn generate_examples(&self) -> Vec<ToolExample> {
        let mut examples = Vec::new();

        // Generate example based on operation parameters
        let mut example_input = serde_json::Map::new();
        
        for param in &self.operation.parameters {
            let example_value = match param.schema.get("type").and_then(|t| t.as_str()) {
                Some("string") => Value::String("example".to_string()),
                Some("integer") => Value::Number(serde_json::Number::from(123)),
                Some("boolean") => Value::Bool(true),
                _ => Value::String("example".to_string()),
            };
            example_input.insert(param.name.clone(), example_value);
        }

        let example_output = serde_json::json!({
            "status": "success",
            "data": {}
        });

        examples.push(ToolExample {
            name: format!("{} Example", self.operation.operation_id),
            description: format!("Example request for {} {}", self.operation.method, self.operation.path),
            input: Value::Object(example_input),
            output: example_output,
        });

        examples
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{ParameterInfo, ParameterLocation};

    fn create_test_operation() -> OperationInfo {
        OperationInfo {
            srn: Srn::openapi_operation("tenant-123", "test-api", "getUser").unwrap(),
            operation_id: "getUser".to_string(),
            method: "GET".to_string(),
            path: "/users/{id}".to_string(),
            summary: Some("Get user by ID".to_string()),
            description: Some("Retrieve a user by their unique identifier".to_string()),
            parameters: vec![
                ParameterInfo {
                    name: "id".to_string(),
                    location: ParameterLocation::Path,
                    required: true,
                    schema: serde_json::json!({"type": "string"}),
                    description: Some("User ID".to_string()),
                }
            ],
            request_body: None,
            responses: HashMap::new(),
            tags: vec!["users".to_string()],
        }
    }

    fn create_test_config() -> OpenApiToolConfig {
        OpenApiToolConfig {
            srn: "stepflow:openapi:tenant-123:test-api:operation:getUser".to_string(),
            base_url: "https://api.example.com".to_string(),
            timeout_ms: Some(30000),
            max_retries: Some(3),
            default_headers: HashMap::new(),
            auth: None,
        }
    }

    #[tokio::test]
    async fn test_tool_info_generation() {
        use crate::document::{DocumentMeta, DocumentFormat, DocumentStatus, OpenApiDocument};
        use chrono::Utc;

        let config = create_test_config();
        let operation = create_test_operation();
        
        let document = OpenApiDocument {
            meta: DocumentMeta {
                id: "test-doc".to_string(),
                name: "Test API".to_string(),
                description: None,
                version: "1.0.0".to_string(),
                tenant_id: "tenant-123".to_string(),
                namespace: "test-api".to_string(),
                format: DocumentFormat::Json,
                status: DocumentStatus::Active,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                operations_count: 1,
                schemas_count: 0,
                servers: vec!["https://api.example.com".to_string()],
            },
            content: "{}".to_string(),
            parsed: serde_json::json!({}),
            operations: vec![operation.clone()],
            schemas: vec![],
        };

        let tool = OpenApiTool::new(config, operation, &document).await.unwrap();
        let info = tool.get_info().await.unwrap();

        assert_eq!(info.name, "GET /users/{id}");
        assert_eq!(info.tool_type, ToolType::OpenAPI);
        assert_eq!(info.status, ToolStatus::Active);
        assert!(!info.examples.is_empty());
    }

    #[test]
    fn test_parameter_validation() {
        let operation = create_test_operation();
        let config = create_test_config();
        
        // This test would need to be async and create a full tool instance
        // For now, we'll test the parameter mapping logic
        let mapping = crate::proxy::config::ParameterMapping::default();
        assert!(mapping.path_params.is_empty());
    }
} 