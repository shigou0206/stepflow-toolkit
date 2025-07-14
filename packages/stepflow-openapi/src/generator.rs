//! OpenAPI Tool Generator
//!
//! This module provides functionality for automatically generating Tool instances
//! from OpenAPI documents and operation information.

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use stepflow_core::types::Tool;
use crate::srn::Srn;
use crate::document::{OpenApiDocument, OperationInfo, DocumentManager};
use crate::tool::{OpenApiTool, OpenApiToolConfig, OpenApiToolError, AuthConfig};

/// Tool generator errors
#[derive(Debug, Error)]
pub enum GeneratorError {
    #[error("Document not found: {0}")]
    DocumentNotFound(String),
    
    #[error("Operation not found: {0}")]
    OperationNotFound(String),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
    
    #[error("Tool creation failed: {0}")]
    ToolCreationFailed(#[from] OpenApiToolError),
    
    #[error("SRN generation failed: {0}")]
    SrnGenerationFailed(String),
    
    #[error("Missing required field: {0}")]
    MissingRequiredField(String),
}

/// Tool generation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolGenerationRequest {
    /// Document ID
    pub document_id: String,
    /// Operation ID (optional, if not provided, generate for all operations)
    pub operation_id: Option<String>,
    /// Base URL for the API
    pub base_url: String,
    /// Default timeout in milliseconds
    pub timeout_ms: Option<u64>,
    /// Maximum retry attempts
    pub max_retries: Option<u32>,
    /// Default headers to include in requests
    pub default_headers: Option<HashMap<String, String>>,
    /// Authentication configuration
    pub auth: Option<AuthConfig>,
    /// Custom tool configuration overrides
    pub tool_config_overrides: Option<HashMap<String, serde_json::Value>>,
}

/// Tool generation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolGenerationResult {
    /// Number of tools generated
    pub tools_generated: usize,
    /// List of generated tool SRNs
    pub tool_srns: Vec<String>,
    /// Any warnings or issues encountered during generation
    pub warnings: Vec<String>,
    /// Generation metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Individual tool generation info
#[derive(Clone)]
pub struct GeneratedToolInfo {
    /// Tool SRN
    pub srn: Srn,
    /// Tool instance
    pub tool: Arc<dyn Tool>,
    /// Configuration used to create the tool
    pub config: OpenApiToolConfig,
    /// Original operation info
    pub operation: OperationInfo,
}

impl std::fmt::Debug for GeneratedToolInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GeneratedToolInfo")
            .field("srn", &self.srn)
            .field("config", &self.config)
            .field("operation", &self.operation)
            .field("tool", &"<Tool instance>")
            .finish()
    }
}

/// Tool generator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorConfig {
    /// Whether to generate tools for all operations or only explicitly requested ones
    pub generate_all_operations: bool,
    /// Default base URL if not provided in requests
    pub default_base_url: Option<String>,
    /// Default timeout for generated tools
    pub default_timeout_ms: u64,
    /// Default max retries for generated tools
    pub default_max_retries: u32,
    /// Whether to include operation tags in tool metadata
    pub include_tags: bool,
    /// Whether to generate examples for tools
    pub generate_examples: bool,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            generate_all_operations: true,
            default_base_url: None,
            default_timeout_ms: 30000,
            default_max_retries: 3,
            include_tags: true,
            generate_examples: true,
        }
    }
}

/// OpenAPI Tool Generator
pub struct ToolGenerator {
    /// Document manager for retrieving OpenAPI documents
    document_manager: Arc<DocumentManager>,
    /// Generator configuration
    config: GeneratorConfig,
    /// Generated tools cache
    generated_tools: std::sync::RwLock<HashMap<String, GeneratedToolInfo>>,
}

impl ToolGenerator {
    /// Create a new tool generator
    pub fn new(document_manager: Arc<DocumentManager>, config: GeneratorConfig) -> Self {
        Self {
            document_manager,
            config,
            generated_tools: std::sync::RwLock::new(HashMap::new()),
        }
    }

    /// Create with default configuration
    pub fn with_default_config(document_manager: Arc<DocumentManager>) -> Self {
        Self::new(document_manager, GeneratorConfig::default())
    }

    /// Generate tools from a document
    pub async fn generate_tools(
        &self,
        request: ToolGenerationRequest,
    ) -> Result<ToolGenerationResult, GeneratorError> {
        // Get the document
        let document = self.document_manager
            .get_document(&request.document_id)
            .await
            .map_err(|e| GeneratorError::DocumentNotFound(format!("Failed to load document: {}", e)))?
            .ok_or_else(|| GeneratorError::DocumentNotFound(request.document_id.clone()))?;

        let mut generated_tools = Vec::new();
        let mut warnings = Vec::new();
        let mut metadata = HashMap::new();

        // Determine which operations to generate tools for
        let operations_to_generate = if let Some(operation_id) = &request.operation_id {
            // Generate for specific operation
            let operation = document.operations.iter()
                .find(|op| op.operation_id == *operation_id)
                .ok_or_else(|| GeneratorError::OperationNotFound(operation_id.clone()))?;
            vec![operation.clone()]
        } else if self.config.generate_all_operations {
            // Generate for all operations
            document.operations.clone()
        } else {
            return Err(GeneratorError::InvalidConfiguration(
                "No operation specified and generate_all_operations is false".to_string()
            ));
        };

        // Generate tools for each operation
        for operation in operations_to_generate {
            match self.generate_tool_for_operation(&request, &document, &operation).await {
                Ok(tool_info) => {
                    // Cache the generated tool
                    let mut cache = self.generated_tools.write().unwrap();
                    cache.insert(tool_info.srn.to_string(), tool_info.clone());
                    
                    generated_tools.push(tool_info);
                }
                Err(e) => {
                    warnings.push(format!("Failed to generate tool for operation '{}': {}", operation.operation_id, e));
                }
            }
        }

        // Add metadata
        metadata.insert("document_id".to_string(), serde_json::Value::String(request.document_id));
        metadata.insert("document_name".to_string(), serde_json::Value::String(document.meta.name));
        metadata.insert("document_version".to_string(), serde_json::Value::String(document.meta.version));
        metadata.insert("total_operations".to_string(), serde_json::Value::Number(document.operations.len().into()));

        Ok(ToolGenerationResult {
            tools_generated: generated_tools.len(),
            tool_srns: generated_tools.iter().map(|t| t.srn.to_string()).collect(),
            warnings,
            metadata,
        })
    }

    /// Generate a tool for a specific operation
    async fn generate_tool_for_operation(
        &self,
        request: &ToolGenerationRequest,
        document: &OpenApiDocument,
        operation: &OperationInfo,
    ) -> Result<GeneratedToolInfo, GeneratorError> {
        // Create tool configuration
        let tool_config = self.create_tool_config(request, operation)?;
        
        // Create the tool instance
        let tool = OpenApiTool::new(tool_config.clone(), operation.clone(), document).await?;
        
        Ok(GeneratedToolInfo {
            srn: operation.srn.clone(),
            tool: Arc::new(tool),
            config: tool_config,
            operation: operation.clone(),
        })
    }

    /// Create tool configuration for an operation
    fn create_tool_config(
        &self,
        request: &ToolGenerationRequest,
        operation: &OperationInfo,
    ) -> Result<OpenApiToolConfig, GeneratorError> {
        let base_url = if request.base_url.is_empty() {
            self.config.default_base_url.clone()
                .ok_or_else(|| GeneratorError::MissingRequiredField("base_url".to_string()))?
        } else {
            request.base_url.clone()
        };

        let mut config = OpenApiToolConfig {
            srn: operation.srn.to_string(),
            base_url,
            timeout_ms: request.timeout_ms.or(Some(self.config.default_timeout_ms)),
            max_retries: request.max_retries.or(Some(self.config.default_max_retries)),
            default_headers: request.default_headers.clone().unwrap_or_default(),
            auth: request.auth.clone(),
        };

        // Apply any configuration overrides
        if let Some(overrides) = &request.tool_config_overrides {
            if let Some(timeout) = overrides.get("timeout_ms").and_then(|v| v.as_u64()) {
                config.timeout_ms = Some(timeout);
            }
            if let Some(retries) = overrides.get("max_retries").and_then(|v| v.as_u64()) {
                config.max_retries = Some(retries as u32);
            }
            if let Some(headers) = overrides.get("default_headers").and_then(|v| v.as_object()) {
                for (k, v) in headers {
                    if let Some(header_value) = v.as_str() {
                        config.default_headers.insert(k.clone(), header_value.to_string());
                    }
                }
            }
        }

        Ok(config)
    }

    /// Get a cached tool by SRN
    pub fn get_tool(&self, srn: &str) -> Option<Arc<dyn Tool>> {
        let cache = self.generated_tools.read().unwrap();
        cache.get(srn).map(|info| info.tool.clone())
    }

    /// Get all cached tools
    pub fn get_all_tools(&self) -> Vec<Arc<dyn Tool>> {
        let cache = self.generated_tools.read().unwrap();
        cache.values().map(|info| info.tool.clone()).collect()
    }

    /// Get tool info by SRN
    pub fn get_tool_info(&self, srn: &str) -> Option<GeneratedToolInfo> {
        let cache = self.generated_tools.read().unwrap();
        cache.get(srn).cloned()
    }

    /// List all generated tool SRNs
    pub fn list_tool_srns(&self) -> Vec<String> {
        let cache = self.generated_tools.read().unwrap();
        cache.keys().cloned().collect()
    }

    /// Remove a tool from cache
    pub fn remove_tool(&self, srn: &str) -> bool {
        let mut cache = self.generated_tools.write().unwrap();
        cache.remove(srn).is_some()
    }

    /// Clear all cached tools
    pub fn clear_cache(&self) {
        let mut cache = self.generated_tools.write().unwrap();
        cache.clear();
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> CacheStats {
        let cache = self.generated_tools.read().unwrap();
        let total_tools = cache.len();
        let mut by_namespace = HashMap::new();
        let mut by_tenant = HashMap::new();

        for (_, info) in cache.iter() {
            // Count by namespace
            let namespace = info.srn.namespace();
            *by_namespace.entry(namespace.to_string()).or_insert(0) += 1;

            // Count by tenant
            let tenant = info.srn.tenant();
            *by_tenant.entry(tenant.to_string()).or_insert(0) += 1;
        }

        CacheStats {
            total_tools,
            by_namespace,
            by_tenant,
        }
    }

    /// Validate that a tool can be executed
    pub async fn validate_tool(&self, srn: &str) -> Result<bool, GeneratorError> {
        let tool = self.get_tool(srn)
            .ok_or_else(|| GeneratorError::OperationNotFound(srn.to_string()))?;

        match tool.test().await {
            Ok(result) => Ok(result),
            Err(_) => Ok(false),
        }
    }

    /// Batch generate tools for multiple documents
    pub async fn batch_generate_tools(
        &self,
        requests: Vec<ToolGenerationRequest>,
    ) -> Vec<Result<ToolGenerationResult, GeneratorError>> {
        let mut results = Vec::new();
        
        for request in requests {
            let result = self.generate_tools(request).await;
            results.push(result);
        }

        results
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// Total number of cached tools
    pub total_tools: usize,
    /// Tools grouped by namespace
    pub by_namespace: HashMap<String, usize>,
    /// Tools grouped by tenant
    pub by_tenant: HashMap<String, usize>,
}

/// Tool registry trait for managing generated tools
#[async_trait]
pub trait ToolRegistry: Send + Sync {
    /// Register a tool
    async fn register_tool(&self, tool_info: GeneratedToolInfo) -> Result<(), GeneratorError>;
    
    /// Unregister a tool
    async fn unregister_tool(&self, srn: &str) -> Result<(), GeneratorError>;
    
    /// Get a tool by SRN
    async fn get_tool(&self, srn: &str) -> Result<Option<Arc<dyn Tool>>, GeneratorError>;
    
    /// List all tools
    async fn list_tools(&self) -> Result<Vec<String>, GeneratorError>;
    
    /// Search tools by pattern
    async fn search_tools(&self, pattern: &str) -> Result<Vec<String>, GeneratorError>;
}

/// In-memory tool registry implementation
pub struct InMemoryToolRegistry {
    tools: std::sync::RwLock<HashMap<String, GeneratedToolInfo>>,
}

impl InMemoryToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: std::sync::RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolRegistry for InMemoryToolRegistry {
    async fn register_tool(&self, tool_info: GeneratedToolInfo) -> Result<(), GeneratorError> {
        let mut tools = self.tools.write().unwrap();
        tools.insert(tool_info.srn.to_string(), tool_info);
        Ok(())
    }

    async fn unregister_tool(&self, srn: &str) -> Result<(), GeneratorError> {
        let mut tools = self.tools.write().unwrap();
        tools.remove(srn);
        Ok(())
    }

    async fn get_tool(&self, srn: &str) -> Result<Option<Arc<dyn Tool>>, GeneratorError> {
        let tools = self.tools.read().unwrap();
        Ok(tools.get(srn).map(|info| info.tool.clone()))
    }

    async fn list_tools(&self) -> Result<Vec<String>, GeneratorError> {
        let tools = self.tools.read().unwrap();
        Ok(tools.keys().cloned().collect())
    }

    async fn search_tools(&self, pattern: &str) -> Result<Vec<String>, GeneratorError> {
        let tools = self.tools.read().unwrap();
        let matching_tools: Vec<String> = tools.keys()
            .filter(|srn| {
                if let Ok(parsed_srn) = Srn::parse(srn) {
                    parsed_srn.matches_pattern(pattern)
                } else {
                    false
                }
            })
            .cloned()
            .collect();
        Ok(matching_tools)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{InMemoryDocumentStorage, DocumentUploadRequest, DocumentFormat};

    async fn create_test_generator() -> ToolGenerator {
        let storage = Box::new(InMemoryDocumentStorage::default());
        let doc_manager = Arc::new(DocumentManager::new(storage));
        ToolGenerator::with_default_config(doc_manager)
    }

    async fn create_test_document(doc_manager: &DocumentManager) -> String {
        let request = DocumentUploadRequest {
            name: "Test API".to_string(),
            namespace: "test-api".to_string(),
            tenant_id: "tenant-123".to_string(),
            content: create_test_openapi_json(),
            format: DocumentFormat::Json,
            description: Some("Test API for tool generation".to_string()),
        };

        doc_manager.upload_document(request).await.unwrap().document_id
    }

    fn create_test_openapi_json() -> String {
        r#"{
            "openapi": "3.0.0",
            "info": {
                "title": "Test API",
                "version": "1.0.0"
            },
            "servers": [
                {
                    "url": "https://api.example.com"
                }
            ],
            "paths": {
                "/users/{id}": {
                    "get": {
                        "operationId": "getUser",
                        "summary": "Get user by ID",
                        "parameters": [
                            {
                                "name": "id",
                                "in": "path",
                                "required": true,
                                "schema": {"type": "string"}
                            }
                        ],
                        "responses": {
                            "200": {
                                "description": "User found",
                                "content": {
                                    "application/json": {
                                        "schema": {
                                            "type": "object",
                                            "properties": {
                                                "id": {"type": "string"},
                                                "name": {"type": "string"}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                "/users": {
                    "post": {
                        "operationId": "createUser",
                        "summary": "Create new user",
                        "requestBody": {
                            "required": true,
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "name": {"type": "string"},
                                            "email": {"type": "string"}
                                        },
                                        "required": ["name", "email"]
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
        }"#.to_string()
    }

    #[tokio::test]
    async fn test_tool_generation() {
        let generator = create_test_generator().await;
        let doc_id = create_test_document(&generator.document_manager).await;

        let request = ToolGenerationRequest {
            document_id: doc_id,
            operation_id: None, // Generate for all operations
            base_url: "https://api.example.com".to_string(),
            timeout_ms: Some(30000),
            max_retries: Some(3),
            default_headers: None,
            auth: None,
            tool_config_overrides: None,
        };

        let result = generator.generate_tools(request).await.unwrap();
        
        assert_eq!(result.tools_generated, 2); // getUser and createUser
        assert_eq!(result.tool_srns.len(), 2);
        assert!(result.warnings.is_empty());
    }

    #[tokio::test]
    async fn test_specific_operation_generation() {
        let generator = create_test_generator().await;
        let doc_id = create_test_document(&generator.document_manager).await;

        let request = ToolGenerationRequest {
            document_id: doc_id,
            operation_id: Some("getUser".to_string()),
            base_url: "https://api.example.com".to_string(),
            timeout_ms: Some(30000),
            max_retries: Some(3),
            default_headers: None,
            auth: None,
            tool_config_overrides: None,
        };

        let result = generator.generate_tools(request).await.unwrap();
        
        assert_eq!(result.tools_generated, 1);
        assert_eq!(result.tool_srns.len(), 1);
        assert!(result.tool_srns[0].contains("getUser"));
    }

    #[tokio::test]
    async fn test_tool_caching() {
        let generator = create_test_generator().await;
        let doc_id = create_test_document(&generator.document_manager).await;

        let request = ToolGenerationRequest {
            document_id: doc_id,
            operation_id: Some("getUser".to_string()),
            base_url: "https://api.example.com".to_string(),
            timeout_ms: Some(30000),
            max_retries: Some(3),
            default_headers: None,
            auth: None,
            tool_config_overrides: None,
        };

        let result = generator.generate_tools(request).await.unwrap();
        let srn = &result.tool_srns[0];
        
        // Tool should be cached
        let cached_tool = generator.get_tool(srn);
        assert!(cached_tool.is_some());
        
        // Get cache stats
        let stats = generator.get_cache_stats();
        assert_eq!(stats.total_tools, 1);
    }

    #[tokio::test]
    async fn test_tool_registry() {
        let registry = InMemoryToolRegistry::new();
        
        // This would normally be a real tool, but for testing we'll create a minimal one
        let generator = create_test_generator().await;
        let doc_id = create_test_document(&generator.document_manager).await;
        
        let request = ToolGenerationRequest {
            document_id: doc_id,
            operation_id: Some("getUser".to_string()),
            base_url: "https://api.example.com".to_string(),
            timeout_ms: Some(30000),
            max_retries: Some(3),
            default_headers: None,
            auth: None,
            tool_config_overrides: None,
        };

        generator.generate_tools(request).await.unwrap();
        let tools = generator.list_tool_srns();
        
        assert!(!tools.is_empty());
        
        // Test registry operations
        let all_tools = registry.list_tools().await.unwrap();
        assert!(all_tools.is_empty()); // Registry is empty initially
    }
} 