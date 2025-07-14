//! StepFlow Resource Name (SRN) System
//!
//! SRN Format: stepflow:<tool-type>:<tenant>:<namespace>:<resource-type>:<resource-id>
//!
//! Examples:
//! - stepflow:openapi:tenant-123:user-api:operation:getUserById
//! - stepflow:openapi:global:payment-api:operation:processPayment

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// SRN Error types
#[derive(Debug, Error)]
pub enum SrnError {
    #[error("Invalid SRN format: expected 6 components separated by ':', got {0}")]
    InvalidFormat(usize),
    
    #[error("Invalid prefix: expected 'stepflow', got '{0}'")]
    InvalidPrefix(String),
    
    #[error("Empty component at position {0}")]
    EmptyComponent(usize),
    
    #[error("Invalid tool type: '{0}'")]
    InvalidToolType(String),
    
    #[error("Invalid resource type: '{0}'")]
    InvalidResourceType(String),
    
    #[error("Invalid characters in component: '{0}'")]
    InvalidCharacters(String),
}

/// StepFlow Resource Name (SRN)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Srn {
    /// Full SRN string
    srn: String,
    /// Parsed components
    components: SrnComponents,
}

/// SRN Components
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SrnComponents {
    /// Fixed prefix: "stepflow"
    pub prefix: String,
    /// Tool type: openapi, asyncapi, python, shell, etc.
    pub tool_type: String,
    /// Tenant identifier
    pub tenant: String,
    /// Namespace (document/service name)
    pub namespace: String,
    /// Resource type: operation, schema, server, etc.
    pub resource_type: String,
    /// Resource identifier
    pub resource_id: String,
}

/// Supported tool types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolType {
    OpenApi,
    AsyncApi,
    Python,
    Shell,
    AI,
    System,
    Custom(String),
}

/// Supported resource types for OpenAPI tools
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OpenApiResourceType {
    Operation,
    Schema,
    Server,
    Component,
}

impl Srn {
    /// Create a new SRN from components
    pub fn new(
        tool_type: &str,
        tenant: &str,
        namespace: &str,
        resource_type: &str,
        resource_id: &str,
    ) -> Result<Self, SrnError> {
        let components = SrnComponents {
            prefix: "stepflow".to_string(),
            tool_type: tool_type.to_string(),
            tenant: tenant.to_string(),
            namespace: namespace.to_string(),
            resource_type: resource_type.to_string(),
            resource_id: resource_id.to_string(),
        };

        // Validate components
        Self::validate_components(&components)?;

        let srn = format!(
            "stepflow:{}:{}:{}:{}:{}",
            tool_type, tenant, namespace, resource_type, resource_id
        );

        Ok(Self { srn, components })
    }

    /// Parse SRN from string
    pub fn parse(srn: &str) -> Result<Self, SrnError> {
        let parts: Vec<&str> = srn.split(':').collect();
        
        if parts.len() != 6 {
            return Err(SrnError::InvalidFormat(parts.len()));
        }

        // Check prefix
        if parts[0] != "stepflow" {
            return Err(SrnError::InvalidPrefix(parts[0].to_string()));
        }

        // Check for empty components
        for (i, part) in parts.iter().enumerate() {
            if part.is_empty() {
                return Err(SrnError::EmptyComponent(i));
            }
        }

        let components = SrnComponents {
            prefix: parts[0].to_string(),
            tool_type: parts[1].to_string(),
            tenant: parts[2].to_string(),
            namespace: parts[3].to_string(),
            resource_type: parts[4].to_string(),
            resource_id: parts[5].to_string(),
        };

        // Validate components
        Self::validate_components(&components)?;

        Ok(Self {
            srn: srn.to_string(),
            components,
        })
    }

    /// Validate SRN components
    fn validate_components(components: &SrnComponents) -> Result<(), SrnError> {
        // Validate tool type
        if !Self::is_valid_tool_type(&components.tool_type) {
            return Err(SrnError::InvalidToolType(components.tool_type.clone()));
        }

        // Validate characters in all components
        for (component, name) in [
            (&components.tool_type, "tool_type"),
            (&components.tenant, "tenant"),
            (&components.namespace, "namespace"),
            (&components.resource_type, "resource_type"),
            (&components.resource_id, "resource_id"),
        ] {
            if !Self::is_valid_component(component) {
                return Err(SrnError::InvalidCharacters(format!("{}: {}", name, component)));
            }
        }

        Ok(())
    }

    /// Check if tool type is valid
    fn is_valid_tool_type(tool_type: &str) -> bool {
        matches!(
            tool_type,
            "openapi" | "asyncapi" | "python" | "shell" | "ai" | "system"
        ) || tool_type.starts_with("custom:")
    }

    /// Check if component contains valid characters
    fn is_valid_component(component: &str) -> bool {
        component
            .chars()
            .all(|c| c.is_alphanumeric() || matches!(c, '-' | '_' | '.'))
    }

    /// Get SRN string
    pub fn as_str(&self) -> &str {
        &self.srn
    }

    /// Get SRN components
    pub fn components(&self) -> &SrnComponents {
        &self.components
    }

    /// Get tool type
    pub fn tool_type(&self) -> &str {
        &self.components.tool_type
    }

    /// Get tenant
    pub fn tenant(&self) -> &str {
        &self.components.tenant
    }

    /// Get namespace
    pub fn namespace(&self) -> &str {
        &self.components.namespace
    }

    /// Get resource type
    pub fn resource_type(&self) -> &str {
        &self.components.resource_type
    }

    /// Get resource ID
    pub fn resource_id(&self) -> &str {
        &self.components.resource_id
    }

    /// Check if SRN matches pattern
    pub fn matches_pattern(&self, pattern: &str) -> bool {
        // Simple wildcard matching (* for any component)
        let pattern_parts: Vec<&str> = pattern.split(':').collect();
        let srn_parts: Vec<&str> = self.srn.split(':').collect();

        if pattern_parts.len() != srn_parts.len() {
            return false;
        }

        pattern_parts
            .iter()
            .zip(srn_parts.iter())
            .all(|(pattern_part, srn_part)| pattern_part == &"*" || pattern_part == srn_part)
    }

    /// Create OpenAPI operation SRN
    pub fn openapi_operation(
        tenant: &str,
        namespace: &str,
        operation_id: &str,
    ) -> Result<Self, SrnError> {
        Self::new("openapi", tenant, namespace, "operation", operation_id)
    }

    /// Create OpenAPI schema SRN
    pub fn openapi_schema(
        tenant: &str,
        namespace: &str,
        schema_name: &str,
    ) -> Result<Self, SrnError> {
        Self::new("openapi", tenant, namespace, "schema", schema_name)
    }
}

impl fmt::Display for Srn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.srn)
    }
}

impl From<Srn> for String {
    fn from(srn: Srn) -> Self {
        srn.srn
    }
}

impl TryFrom<String> for Srn {
    type Error = SrnError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::parse(&s)
    }
}

impl TryFrom<&str> for Srn {
    type Error = SrnError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::parse(s)
    }
}

/// SRN Builder for fluent construction
pub struct SrnBuilder {
    tool_type: Option<String>,
    tenant: Option<String>,
    namespace: Option<String>,
    resource_type: Option<String>,
    resource_id: Option<String>,
}

impl SrnBuilder {
    /// Create new builder
    pub fn new() -> Self {
        Self {
            tool_type: None,
            tenant: None,
            namespace: None,
            resource_type: None,
            resource_id: None,
        }
    }

    /// Set tool type
    pub fn tool_type(mut self, tool_type: &str) -> Self {
        self.tool_type = Some(tool_type.to_string());
        self
    }

    /// Set tenant
    pub fn tenant(mut self, tenant: &str) -> Self {
        self.tenant = Some(tenant.to_string());
        self
    }

    /// Set namespace
    pub fn namespace(mut self, namespace: &str) -> Self {
        self.namespace = Some(namespace.to_string());
        self
    }

    /// Set resource type
    pub fn resource_type(mut self, resource_type: &str) -> Self {
        self.resource_type = Some(resource_type.to_string());
        self
    }

    /// Set resource ID
    pub fn resource_id(mut self, resource_id: &str) -> Self {
        self.resource_id = Some(resource_id.to_string());
        self
    }

    /// Build SRN
    pub fn build(self) -> Result<Srn, SrnError> {
        let tool_type = self.tool_type.ok_or_else(|| SrnError::EmptyComponent(1))?;
        let tenant = self.tenant.ok_or_else(|| SrnError::EmptyComponent(2))?;
        let namespace = self.namespace.ok_or_else(|| SrnError::EmptyComponent(3))?;
        let resource_type = self.resource_type.ok_or_else(|| SrnError::EmptyComponent(4))?;
        let resource_id = self.resource_id.ok_or_else(|| SrnError::EmptyComponent(5))?;

        Srn::new(&tool_type, &tenant, &namespace, &resource_type, &resource_id)
    }
}

impl Default for SrnBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_srn_creation() {
        let srn = Srn::new("openapi", "tenant-123", "user-api", "operation", "getUserById").unwrap();
        assert_eq!(srn.as_str(), "stepflow:openapi:tenant-123:user-api:operation:getUserById");
        assert_eq!(srn.tool_type(), "openapi");
        assert_eq!(srn.tenant(), "tenant-123");
        assert_eq!(srn.namespace(), "user-api");
        assert_eq!(srn.resource_type(), "operation");
        assert_eq!(srn.resource_id(), "getUserById");
    }

    #[test]
    fn test_srn_parsing() {
        let srn_str = "stepflow:openapi:tenant-123:user-api:operation:getUserById";
        let srn = Srn::parse(srn_str).unwrap();
        assert_eq!(srn.as_str(), srn_str);
    }

    #[test]
    fn test_srn_validation() {
        // Test invalid format
        assert!(Srn::parse("invalid:format").is_err());
        
        // Test invalid prefix
        assert!(Srn::parse("invalid:openapi:tenant:ns:op:id").is_err());
        
        // Test empty component
        assert!(Srn::parse("stepflow::tenant:ns:op:id").is_err());
        
        // Test invalid tool type
        assert!(Srn::new("invalid", "tenant", "ns", "op", "id").is_err());
    }

    #[test]
    fn test_srn_pattern_matching() {
        let srn = Srn::parse("stepflow:openapi:tenant-123:user-api:operation:getUserById").unwrap();
        
        assert!(srn.matches_pattern("stepflow:openapi:tenant-123:user-api:operation:getUserById"));
        assert!(srn.matches_pattern("stepflow:openapi:*:user-api:operation:*"));
        assert!(srn.matches_pattern("stepflow:*:*:*:*:*"));
        assert!(!srn.matches_pattern("stepflow:asyncapi:*:*:*:*"));
    }

    #[test]
    fn test_srn_builder() {
        let srn = SrnBuilder::new()
            .tool_type("openapi")
            .tenant("tenant-123")
            .namespace("user-api")
            .resource_type("operation")
            .resource_id("getUserById")
            .build()
            .unwrap();
        
        assert_eq!(srn.as_str(), "stepflow:openapi:tenant-123:user-api:operation:getUserById");
    }

    #[test]
    fn test_openapi_helpers() {
        let operation_srn = Srn::openapi_operation("tenant-123", "user-api", "getUser").unwrap();
        assert_eq!(operation_srn.as_str(), "stepflow:openapi:tenant-123:user-api:operation:getUser");
        
        let schema_srn = Srn::openapi_schema("tenant-123", "user-api", "User").unwrap();
        assert_eq!(schema_srn.as_str(), "stepflow:openapi:tenant-123:user-api:schema:User");
    }
} 