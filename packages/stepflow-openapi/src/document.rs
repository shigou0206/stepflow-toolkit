//! OpenAPI Document Management
//!
//! This module provides functionality for managing OpenAPI documents:
//! - Document upload and validation
//! - Storage and retrieval
//! - SRN generation for operations

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use thiserror::Error;

use crate::srn::{Srn, SrnError};

/// Document Manager Errors
#[derive(Debug, Error)]
pub enum DocumentError {
    #[error("Invalid OpenAPI format: {0}")]
    InvalidFormat(String),
    
    #[error("Document validation failed: {0}")]
    ValidationFailed(String),
    
    #[error("Document not found: {0}")]
    NotFound(String),
    
    #[error("Document already exists: {0}")]
    AlreadyExists(String),
    
    #[error("SRN generation failed: {0}")]
    SrnGeneration(#[from] SrnError),
    
    #[error("Parsing error: {0}")]
    ParseError(String),
    
    #[error("Storage error: {0}")]
    StorageError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Document format types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentFormat {
    Json,
    Yaml,
}

/// Document status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentStatus {
    Active,
    Inactive,
    Processing,
    Error,
}

/// OpenAPI Document metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMeta {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub tenant_id: String,
    pub namespace: String,
    pub format: DocumentFormat,
    pub status: DocumentStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub operations_count: usize,
    pub schemas_count: usize,
    pub servers: Vec<String>,
}

/// Full OpenAPI Document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenApiDocument {
    pub meta: DocumentMeta,
    pub content: String,
    pub parsed: Value,
    pub operations: Vec<OperationInfo>,
    pub schemas: Vec<SchemaInfo>,
}

/// Operation information extracted from OpenAPI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationInfo {
    pub srn: Srn,
    pub operation_id: String,
    pub method: String,
    pub path: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub parameters: Vec<ParameterInfo>,
    pub request_body: Option<RequestBodyInfo>,
    pub responses: HashMap<String, ResponseInfo>,
    pub tags: Vec<String>,
}

/// Parameter information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterInfo {
    pub name: String,
    pub location: ParameterLocation,
    pub required: bool,
    pub schema: Value,
    pub description: Option<String>,
}

/// Parameter location
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParameterLocation {
    Query,
    Path,
    Header,
    Cookie,
}

/// Request body information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestBodyInfo {
    pub required: bool,
    pub content: HashMap<String, MediaTypeInfo>,
    pub description: Option<String>,
}

/// Response information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseInfo {
    pub description: String,
    pub content: Option<HashMap<String, MediaTypeInfo>>,
    pub headers: Option<HashMap<String, Value>>,
}

/// Media type information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaTypeInfo {
    pub schema: Option<Value>,
}

/// Schema information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaInfo {
    pub srn: Srn,
    pub name: String,
    pub schema: Value,
    pub description: Option<String>,
}

/// Document upload request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentUploadRequest {
    pub name: String,
    pub namespace: String,
    pub tenant_id: String,
    pub content: String,
    pub format: DocumentFormat,
    pub description: Option<String>,
}

/// Document upload result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentUploadResult {
    pub document_id: String,
    pub operations: Vec<String>, // SRN strings
    pub schemas: Vec<String>,    // SRN strings
}

/// OpenAPI Document Manager
pub struct DocumentManager {
    storage: Box<dyn DocumentStorage>,
}

/// Document storage trait
#[async_trait::async_trait]
pub trait DocumentStorage: Send + Sync {
    async fn save_document(&self, document: &OpenApiDocument) -> Result<(), DocumentError>;
    async fn get_document(&self, id: &str) -> Result<Option<OpenApiDocument>, DocumentError>;
    async fn list_documents(&self, tenant_id: &str) -> Result<Vec<DocumentMeta>, DocumentError>;
    async fn delete_document(&self, id: &str) -> Result<(), DocumentError>;
    async fn update_document(&self, document: &OpenApiDocument) -> Result<(), DocumentError>;
}

impl DocumentManager {
    /// Create new document manager
    pub fn new(storage: Box<dyn DocumentStorage>) -> Self {
        Self { storage }
    }

    /// Upload and process OpenAPI document
    pub async fn upload_document(
        &self,
        request: DocumentUploadRequest,
    ) -> Result<DocumentUploadResult, DocumentError> {
        // Parse document
        let parsed = self.parse_document(&request.content, &request.format)?;
        
        // Validate OpenAPI structure
        self.validate_openapi_document(&parsed)?;
        
        // Create document metadata
        let document_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        // Extract operations and schemas
        let operations = self.extract_operations(&parsed, &request.tenant_id, &request.namespace)?;
        let schemas = self.extract_schemas(&parsed, &request.tenant_id, &request.namespace)?;
        
        // Get servers
        let servers = self.extract_servers(&parsed);
        
        let meta = DocumentMeta {
            id: document_id.clone(),
            name: request.name,
            description: request.description,
            version: self.extract_version(&parsed),
            tenant_id: request.tenant_id,
            namespace: request.namespace,
            format: request.format,
            status: DocumentStatus::Active,
            created_at: now,
            updated_at: now,
            operations_count: operations.len(),
            schemas_count: schemas.len(),
            servers,
        };

        let document = OpenApiDocument {
            meta,
            content: request.content,
            parsed,
            operations: operations.clone(),
            schemas: schemas.clone(),
        };

        // Save to storage
        self.storage.save_document(&document).await?;

        Ok(DocumentUploadResult {
            document_id,
            operations: operations.into_iter().map(|op| op.srn.to_string()).collect(),
            schemas: schemas.into_iter().map(|schema| schema.srn.to_string()).collect(),
        })
    }

    /// Get document by ID
    pub async fn get_document(&self, id: &str) -> Result<Option<OpenApiDocument>, DocumentError> {
        self.storage.get_document(id).await
    }

    /// List documents for tenant
    pub async fn list_documents(&self, tenant_id: &str) -> Result<Vec<DocumentMeta>, DocumentError> {
        self.storage.list_documents(tenant_id).await
    }

    /// Delete document
    pub async fn delete_document(&self, id: &str) -> Result<(), DocumentError> {
        self.storage.delete_document(id).await
    }

    /// Parse document content
    fn parse_document(&self, content: &str, format: &DocumentFormat) -> Result<Value, DocumentError> {
        match format {
            DocumentFormat::Json => {
                serde_json::from_str(content)
                    .map_err(|e| DocumentError::ParseError(format!("JSON parse error: {}", e)))
            }
            DocumentFormat::Yaml => {
                // Use direct YAML parsing instead of CST for now
                serde_yaml::from_str(content)
                    .map_err(|e| DocumentError::ParseError(format!("YAML parse error: {}", e)))
            }
        }
    }

    /// Convert CST to JSON Value (placeholder for future enhancement)
    fn cst_to_json_value(&self, cst: &crate::TreeCursorSyntaxNode) -> Result<Value, DocumentError> {
        // This is a simplified conversion - should be enhanced based on CST structure
        let content = cst.text();
        serde_yaml::from_str(&content)
            .map_err(|e| DocumentError::ParseError(format!("CST to JSON conversion error: {}", e)))
    }

    /// Validate OpenAPI document structure
    fn validate_openapi_document(&self, parsed: &Value) -> Result<(), DocumentError> {
        // Check required OpenAPI fields
        let openapi_version = parsed.get("openapi")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DocumentError::ValidationFailed("Missing 'openapi' field".to_string()))?;

        if !openapi_version.starts_with("3.") {
            return Err(DocumentError::ValidationFailed(
                format!("Unsupported OpenAPI version: {}", openapi_version)
            ));
        }

        // Check for required info section
        parsed.get("info")
            .ok_or_else(|| DocumentError::ValidationFailed("Missing 'info' section".to_string()))?;

        // Check for paths section
        parsed.get("paths")
            .ok_or_else(|| DocumentError::ValidationFailed("Missing 'paths' section".to_string()))?;

        Ok(())
    }

    /// Extract operations from OpenAPI document
    fn extract_operations(
        &self,
        parsed: &Value,
        tenant_id: &str,
        namespace: &str,
    ) -> Result<Vec<OperationInfo>, DocumentError> {
        let mut operations = Vec::new();
        
        let paths = parsed.get("paths")
            .and_then(|p| p.as_object())
            .ok_or_else(|| DocumentError::ValidationFailed("Invalid paths section".to_string()))?;

        for (path, path_item) in paths {
            if let Some(path_obj) = path_item.as_object() {
                for (method, operation) in path_obj {
                    if matches!(method.as_str(), "get" | "post" | "put" | "delete" | "patch" | "head" | "options") {
                        if let Some(op_obj) = operation.as_object() {
                            let operation_id = op_obj.get("operationId")
                                .and_then(|v| v.as_str())
                                .unwrap_or(&format!("{}_{}", method, path.replace('/', "_").trim_start_matches('_')))
                                .to_string();

                            let srn = Srn::openapi_operation(tenant_id, namespace, &operation_id)?;

                            let operation_info = OperationInfo {
                                srn,
                                operation_id,
                                method: method.to_uppercase(),
                                path: path.clone(),
                                summary: op_obj.get("summary").and_then(|v| v.as_str()).map(String::from),
                                description: op_obj.get("description").and_then(|v| v.as_str()).map(String::from),
                                parameters: self.extract_parameters(op_obj)?,
                                request_body: self.extract_request_body(op_obj)?,
                                responses: self.extract_responses(op_obj)?,
                                tags: self.extract_tags(op_obj),
                            };

                            operations.push(operation_info);
                        }
                    }
                }
            }
        }

        Ok(operations)
    }

    /// Extract schemas from OpenAPI document
    fn extract_schemas(
        &self,
        parsed: &Value,
        tenant_id: &str,
        namespace: &str,
    ) -> Result<Vec<SchemaInfo>, DocumentError> {
        let mut schemas = Vec::new();

        if let Some(components) = parsed.get("components").and_then(|c| c.as_object()) {
            if let Some(schemas_obj) = components.get("schemas").and_then(|s| s.as_object()) {
                for (schema_name, schema_value) in schemas_obj {
                    let srn = Srn::openapi_schema(tenant_id, namespace, schema_name)?;
                    
                    let schema_info = SchemaInfo {
                        srn,
                        name: schema_name.clone(),
                        schema: schema_value.clone(),
                        description: schema_value.get("description")
                            .and_then(|v| v.as_str())
                            .map(String::from),
                    };

                    schemas.push(schema_info);
                }
            }
        }

        Ok(schemas)
    }

    /// Extract parameters from operation
    fn extract_parameters(&self, operation: &serde_json::Map<String, Value>) -> Result<Vec<ParameterInfo>, DocumentError> {
        let mut parameters = Vec::new();

        if let Some(params) = operation.get("parameters").and_then(|p| p.as_array()) {
            for param in params {
                if let Some(param_obj) = param.as_object() {
                    let name = param_obj.get("name")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DocumentError::ValidationFailed("Parameter missing name".to_string()))?
                        .to_string();

                    let location_str = param_obj.get("in")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DocumentError::ValidationFailed("Parameter missing 'in' field".to_string()))?;

                    let location = match location_str {
                        "query" => ParameterLocation::Query,
                        "path" => ParameterLocation::Path,
                        "header" => ParameterLocation::Header,
                        "cookie" => ParameterLocation::Cookie,
                        _ => return Err(DocumentError::ValidationFailed(format!("Invalid parameter location: {}", location_str))),
                    };

                    let required = param_obj.get("required")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);

                    let schema = param_obj.get("schema")
                        .cloned()
                        .unwrap_or(Value::Object(serde_json::Map::new()));

                    let description = param_obj.get("description")
                        .and_then(|v| v.as_str())
                        .map(String::from);

                    parameters.push(ParameterInfo {
                        name,
                        location,
                        required,
                        schema,
                        description,
                    });
                }
            }
        }

        Ok(parameters)
    }

    /// Extract request body from operation
    fn extract_request_body(&self, operation: &serde_json::Map<String, Value>) -> Result<Option<RequestBodyInfo>, DocumentError> {
        if let Some(request_body) = operation.get("requestBody").and_then(|rb| rb.as_object()) {
            let required = request_body.get("required")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let description = request_body.get("description")
                .and_then(|v| v.as_str())
                .map(String::from);

            let mut content = HashMap::new();
            if let Some(content_obj) = request_body.get("content").and_then(|c| c.as_object()) {
                for (media_type, media_info) in content_obj {
                    let schema = media_info.get("schema").cloned();
                    content.insert(media_type.clone(), MediaTypeInfo { schema });
                }
            }

            Ok(Some(RequestBodyInfo {
                required,
                content,
                description,
            }))
        } else {
            Ok(None)
        }
    }

    /// Extract responses from operation
    fn extract_responses(&self, operation: &serde_json::Map<String, Value>) -> Result<HashMap<String, ResponseInfo>, DocumentError> {
        let mut responses = HashMap::new();

        if let Some(responses_obj) = operation.get("responses").and_then(|r| r.as_object()) {
            for (status_code, response) in responses_obj {
                if let Some(response_obj) = response.as_object() {
                    let description = response_obj.get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("No description")
                        .to_string();

                    let mut content = None;
                    if let Some(content_obj) = response_obj.get("content").and_then(|c| c.as_object()) {
                        let mut content_map = HashMap::new();
                        for (media_type, media_info) in content_obj {
                            let schema = media_info.get("schema").cloned();
                            content_map.insert(media_type.clone(), MediaTypeInfo { schema });
                        }
                        content = Some(content_map);
                    }

                    let headers = response_obj.get("headers")
                        .and_then(|h| h.as_object())
                        .map(|headers_obj| {
                            headers_obj.iter()
                                .map(|(k, v)| (k.clone(), v.clone()))
                                .collect::<HashMap<String, Value>>()
                        });

                    responses.insert(status_code.clone(), ResponseInfo {
                        description,
                        content,
                        headers,
                    });
                }
            }
        }

        Ok(responses)
    }

    /// Extract tags from operation
    fn extract_tags(&self, operation: &serde_json::Map<String, Value>) -> Vec<String> {
        operation.get("tags")
            .and_then(|t| t.as_array())
            .map(|tags| {
                tags.iter()
                    .filter_map(|tag| tag.as_str())
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Extract servers from document
    fn extract_servers(&self, parsed: &Value) -> Vec<String> {
        parsed.get("servers")
            .and_then(|s| s.as_array())
            .map(|servers| {
                servers.iter()
                    .filter_map(|server| {
                        server.get("url").and_then(|url| url.as_str())
                    })
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Extract version from document
    fn extract_version(&self, parsed: &Value) -> String {
        parsed.get("info")
            .and_then(|info| info.get("version"))
            .and_then(|v| v.as_str())
            .unwrap_or("1.0.0")
            .to_string()
    }
}

/// In-memory document storage implementation for testing/development
#[derive(Default)]
pub struct InMemoryDocumentStorage {
    documents: std::sync::RwLock<HashMap<String, OpenApiDocument>>,
}

#[async_trait::async_trait]
impl DocumentStorage for InMemoryDocumentStorage {
    async fn save_document(&self, document: &OpenApiDocument) -> Result<(), DocumentError> {
        let mut docs = self.documents.write().unwrap();
        docs.insert(document.meta.id.clone(), document.clone());
        Ok(())
    }

    async fn get_document(&self, id: &str) -> Result<Option<OpenApiDocument>, DocumentError> {
        let docs = self.documents.read().unwrap();
        Ok(docs.get(id).cloned())
    }

    async fn list_documents(&self, tenant_id: &str) -> Result<Vec<DocumentMeta>, DocumentError> {
        let docs = self.documents.read().unwrap();
        let filtered: Vec<DocumentMeta> = docs
            .values()
            .filter(|doc| doc.meta.tenant_id == tenant_id)
            .map(|doc| doc.meta.clone())
            .collect();
        Ok(filtered)
    }

    async fn delete_document(&self, id: &str) -> Result<(), DocumentError> {
        let mut docs = self.documents.write().unwrap();
        docs.remove(id);
        Ok(())
    }

    async fn update_document(&self, document: &OpenApiDocument) -> Result<(), DocumentError> {
        self.save_document(document).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_document_upload() {
        let storage = Box::new(InMemoryDocumentStorage::default());
        let manager = DocumentManager::new(storage);

        let request = DocumentUploadRequest {
            name: "Test API".to_string(),
            namespace: "test-api".to_string(),
            tenant_id: "tenant-123".to_string(),
            content: create_test_openapi_json(),
            format: DocumentFormat::Json,
            description: Some("Test description".to_string()),
        };

        let result = manager.upload_document(request).await.unwrap();
        assert!(!result.document_id.is_empty());
        assert!(!result.operations.is_empty());
    }

    fn create_test_openapi_json() -> String {
        r#"{
            "openapi": "3.0.0",
            "info": {
                "title": "Test API",
                "version": "1.0.0"
            },
            "paths": {
                "/users/{id}": {
                    "get": {
                        "operationId": "getUser",
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
                                "description": "User found"
                            }
                        }
                    }
                }
            }
        }"#.to_string()
    }
} 