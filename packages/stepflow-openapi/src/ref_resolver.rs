//! OpenAPI $ref Reference Resolver
//!
//! This module provides functionality for resolving OpenAPI $ref references:
//! - Recursive reference resolution
//! - Circular reference detection
//! - Complete schema generation

use serde_json::{Value, Map};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

/// Reference resolution errors
#[derive(Debug, Error)]
pub enum RefResolverError {
    #[error("Reference not found: {0}")]
    ReferenceNotFound(String),
    
    #[error("Circular reference detected: {0}")]
    CircularReference(String),
    
    #[error("Invalid reference format: {0}")]
    InvalidReference(String),
    
    #[error("JSON path error: {0}")]
    JsonPathError(String),
    
    #[error("Resolution depth exceeded: max {0}")]
    MaxDepthExceeded(usize),
}

/// Reference resolver configuration
#[derive(Debug, Clone)]
pub struct RefResolverConfig {
    /// Maximum resolution depth to prevent infinite loops
    pub max_depth: usize,
    /// Whether to preserve original $ref for debugging
    pub preserve_refs: bool,
    /// Whether to inline all references
    pub inline_all: bool,
}

impl Default for RefResolverConfig {
    fn default() -> Self {
        Self {
            max_depth: 100,
            preserve_refs: false,
            inline_all: true,
        }
    }
}

/// Reference tracking information
#[derive(Debug, Clone)]
struct RefContext {
    /// Current resolution depth
    depth: usize,
    /// References being resolved (for circular detection)
    resolving: HashSet<String>,
    /// Already resolved references (caching)
    resolved: HashMap<String, Value>,
}

/// OpenAPI Reference Resolver
pub struct RefResolver {
    config: RefResolverConfig,
}

impl RefResolver {
    /// Create new reference resolver with default config
    pub fn new() -> Self {
        Self {
            config: RefResolverConfig::default(),
        }
    }

    /// Create reference resolver with custom config
    pub fn with_config(config: RefResolverConfig) -> Self {
        Self { config }
    }

    /// Resolve all references in OpenAPI document
    pub fn resolve_document(&self, document: &Value) -> Result<Value, RefResolverError> {
        let mut context = RefContext {
            depth: 0,
            resolving: HashSet::new(),
            resolved: HashMap::new(),
        };

        self.resolve_value(document, document, &mut context)
    }

    /// Resolve references in a specific value
    fn resolve_value(
        &self,
        value: &Value,
        root_document: &Value,
        context: &mut RefContext,
    ) -> Result<Value, RefResolverError> {
        if context.depth > self.config.max_depth {
            return Err(RefResolverError::MaxDepthExceeded(self.config.max_depth));
        }

        match value {
            Value::Object(obj) => self.resolve_object(obj, root_document, context),
            Value::Array(arr) => self.resolve_array(arr, root_document, context),
            _ => Ok(value.clone()),
        }
    }

    /// Resolve references in an object
    fn resolve_object(
        &self,
        obj: &Map<String, Value>,
        root_document: &Value,
        context: &mut RefContext,
    ) -> Result<Value, RefResolverError> {
        // Check if this object contains a $ref
        if let Some(ref_value) = obj.get("$ref") {
            if let Some(ref_str) = ref_value.as_str() {
                return self.resolve_reference(ref_str, root_document, context);
            }
        }

        // Resolve all values in the object
        let mut resolved_obj = Map::new();
        context.depth += 1;

        for (key, value) in obj {
            let resolved_value = self.resolve_value(value, root_document, context)?;
            resolved_obj.insert(key.clone(), resolved_value);
        }

        context.depth -= 1;
        Ok(Value::Object(resolved_obj))
    }

    /// Resolve references in an array
    fn resolve_array(
        &self,
        arr: &Vec<Value>,
        root_document: &Value,
        context: &mut RefContext,
    ) -> Result<Value, RefResolverError> {
        context.depth += 1;
        let mut resolved_arr = Vec::new();

        for item in arr {
            let resolved_item = self.resolve_value(item, root_document, context)?;
            resolved_arr.push(resolved_item);
        }

        context.depth -= 1;
        Ok(Value::Array(resolved_arr))
    }

    /// Resolve a specific reference
    fn resolve_reference(
        &self,
        ref_str: &str,
        root_document: &Value,
        context: &mut RefContext,
    ) -> Result<Value, RefResolverError> {
        // Check for circular reference
        if context.resolving.contains(ref_str) {
            return Err(RefResolverError::CircularReference(ref_str.to_string()));
        }

        // Check if already resolved
        if let Some(cached) = context.resolved.get(ref_str) {
            return Ok(cached.clone());
        }

        // Add to resolving set
        context.resolving.insert(ref_str.to_string());

        let result = match self.resolve_reference_path(ref_str, root_document) {
            Ok(referenced_value) => {
                // Recursively resolve the referenced value
                self.resolve_value(&referenced_value, root_document, context)
            }
            Err(e) => Err(e),
        };

        // Remove from resolving set
        context.resolving.remove(ref_str);

        // Cache the result if successful
        if let Ok(ref resolved_value) = result {
            context.resolved.insert(ref_str.to_string(), resolved_value.clone());
        }

        result
    }

    /// Resolve a reference path to its value
    fn resolve_reference_path(
        &self,
        ref_str: &str,
        root_document: &Value,
    ) -> Result<Value, RefResolverError> {
        // Handle different reference formats
        if ref_str.starts_with("#/") {
            // JSON Pointer reference (e.g., "#/components/schemas/User")
            self.resolve_json_pointer(&ref_str[2..], root_document)
        } else if ref_str.starts_with("#") {
            // Fragment reference (e.g., "#User")
            self.resolve_fragment(&ref_str[1..], root_document)
        } else {
            // External reference - not supported yet
            Err(RefResolverError::InvalidReference(format!(
                "External references not supported: {}",
                ref_str
            )))
        }
    }

    /// Resolve a JSON Pointer path
    fn resolve_json_pointer(
        &self,
        path: &str,
        document: &Value,
    ) -> Result<Value, RefResolverError> {
        let parts: Vec<&str> = path.split('/').collect();
        let mut current = document;

        for part in parts {
            if part.is_empty() {
                continue;
            }

            // Decode JSON Pointer special characters
            let decoded_part = part
                .replace("~1", "/")
                .replace("~0", "~");

            match current {
                Value::Object(obj) => {
                    current = obj.get(&decoded_part).ok_or_else(|| {
                        RefResolverError::ReferenceNotFound(format!(
                            "Property '{}' not found in path '{}'",
                            decoded_part, path
                        ))
                    })?;
                }
                Value::Array(arr) => {
                    let index: usize = decoded_part.parse().map_err(|_| {
                        RefResolverError::JsonPathError(format!(
                            "Invalid array index '{}' in path '{}'",
                            decoded_part, path
                        ))
                    })?;
                    current = arr.get(index).ok_or_else(|| {
                        RefResolverError::ReferenceNotFound(format!(
                            "Array index {} out of bounds in path '{}'",
                            index, path
                        ))
                    })?;
                }
                _ => {
                    return Err(RefResolverError::JsonPathError(format!(
                        "Cannot traverse into non-object/array at '{}' in path '{}'",
                        decoded_part, path
                    )));
                }
            }
        }

        Ok(current.clone())
    }

    /// Resolve a fragment reference (simple name lookup)
    fn resolve_fragment(
        &self,
        fragment: &str,
        document: &Value,
    ) -> Result<Value, RefResolverError> {
        // Try common OpenAPI locations for the fragment
        let search_paths = [
            format!("components/schemas/{}", fragment),
            format!("components/responses/{}", fragment),
            format!("components/parameters/{}", fragment),
            format!("components/examples/{}", fragment),
            format!("components/requestBodies/{}", fragment),
            format!("components/headers/{}", fragment),
            format!("components/securitySchemes/{}", fragment),
            format!("components/links/{}", fragment),
            format!("components/callbacks/{}", fragment),
            format!("definitions/{}", fragment), // OpenAPI 2.0 compatibility
        ];

        for path in &search_paths {
            if let Ok(value) = self.resolve_json_pointer(path, document) {
                return Ok(value);
            }
        }

        Err(RefResolverError::ReferenceNotFound(format!(
            "Fragment '{}' not found in document",
            fragment
        )))
    }

    /// Get all references in a document
    pub fn collect_references(&self, document: &Value) -> Vec<String> {
        let mut refs = Vec::new();
        self.collect_refs_recursive(document, &mut refs);
        refs
    }

    /// Recursively collect all references
    fn collect_refs_recursive(&self, value: &Value, refs: &mut Vec<String>) {
        match value {
            Value::Object(obj) => {
                if let Some(ref_value) = obj.get("$ref") {
                    if let Some(ref_str) = ref_value.as_str() {
                        refs.push(ref_str.to_string());
                    }
                }
                for (_, v) in obj {
                    self.collect_refs_recursive(v, refs);
                }
            }
            Value::Array(arr) => {
                for item in arr {
                    self.collect_refs_recursive(item, refs);
                }
            }
            _ => {}
        }
    }

    /// Validate that all references can be resolved
    pub fn validate_references(&self, document: &Value) -> Result<(), RefResolverError> {
        let refs = self.collect_references(document);
        
        for ref_str in refs {
            self.resolve_reference_path(&ref_str, document)?;
        }

        Ok(())
    }
}

impl Default for RefResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to resolve all references in a document
pub fn resolve_refs(document: &Value) -> Result<Value, RefResolverError> {
    RefResolver::new().resolve_document(document)
}

/// Convenience function to resolve references with custom config
pub fn resolve_refs_with_config(
    document: &Value,
    config: RefResolverConfig,
) -> Result<Value, RefResolverError> {
    RefResolver::with_config(config).resolve_document(document)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_simple_reference_resolution() {
        let document = json!({
            "components": {
                "schemas": {
                    "User": {
                        "type": "object",
                        "properties": {
                            "id": {"type": "string"},
                            "name": {"type": "string"}
                        }
                    }
                }
            },
            "paths": {
                "/users/{id}": {
                    "get": {
                        "responses": {
                            "200": {
                                "content": {
                                    "application/json": {
                                        "schema": {"$ref": "#/components/schemas/User"}
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        let resolver = RefResolver::new();
        let resolved = resolver.resolve_document(&document).unwrap();

        // Check that reference was resolved
        let schema = &resolved["paths"]["/users/{id}"]["get"]["responses"]["200"]["content"]["application/json"]["schema"];
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["id"].is_object());
    }

    #[test]
    fn test_circular_reference_detection() {
        let document = json!({
            "components": {
                "schemas": {
                    "Node": {
                        "type": "object",
                        "properties": {
                            "value": {"type": "string"},
                            "child": {"$ref": "#/components/schemas/Node"}
                        }
                    }
                }
            }
        });

        let resolver = RefResolver::new();
        let result = resolver.resolve_document(&document);
        
        // Should detect circular reference
        assert!(matches!(result, Err(RefResolverError::CircularReference(_))));
    }

    #[test]
    fn test_nested_reference_resolution() {
        let document = json!({
            "components": {
                "schemas": {
                    "Address": {
                        "type": "object",
                        "properties": {
                            "street": {"type": "string"},
                            "city": {"type": "string"}
                        }
                    },
                    "User": {
                        "type": "object",
                        "properties": {
                            "id": {"type": "string"},
                            "address": {"$ref": "#/components/schemas/Address"}
                        }
                    }
                }
            },
            "example": {"$ref": "#/components/schemas/User"}
        });

        let resolver = RefResolver::new();
        let resolved = resolver.resolve_document(&document).unwrap();

        // Check nested resolution
        let address_schema = &resolved["example"]["properties"]["address"];
        assert_eq!(address_schema["type"], "object");
        assert!(address_schema["properties"]["street"].is_object());
    }

    #[test]
    fn test_reference_collection() {
        let document = json!({
            "components": {
                "schemas": {
                    "User": {"type": "object"}
                }
            },
            "paths": {
                "/users": {
                    "get": {
                        "responses": {
                            "200": {
                                "schema": {"$ref": "#/components/schemas/User"}
                            }
                        }
                    },
                    "post": {
                        "requestBody": {
                            "schema": {"$ref": "#/components/schemas/User"}
                        }
                    }
                }
            }
        });

        let resolver = RefResolver::new();
        let refs = resolver.collect_references(&document);
        
        assert_eq!(refs.len(), 2);
        assert!(refs.contains(&"#/components/schemas/User".to_string()));
    }

    #[test]
    fn test_reference_validation() {
        let valid_document = json!({
            "components": {
                "schemas": {
                    "User": {"type": "object"}
                }
            },
            "example": {"$ref": "#/components/schemas/User"}
        });

        let invalid_document = json!({
            "example": {"$ref": "#/components/schemas/NonExistent"}
        });

        let resolver = RefResolver::new();
        
        assert!(resolver.validate_references(&valid_document).is_ok());
        assert!(resolver.validate_references(&invalid_document).is_err());
    }

    #[test]
    fn test_json_pointer_decoding() {
        let document = json!({
            "components": {
                "schemas": {
                    "User/Info": {
                        "~special": {"type": "string"}
                    }
                }
            }
        });

        let resolver = RefResolver::new();
        
        // Test encoded JSON Pointer
        let result = resolver.resolve_json_pointer("components/schemas/User~1Info/~0special", &document);
        assert!(result.is_ok());
        assert_eq!(result.unwrap()["type"], "string");
    }
} 