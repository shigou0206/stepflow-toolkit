//! Validation utilities

use stepflow_core::*;

/// Input validator implementation
pub struct InputValidator;

impl InputValidator {
    /// Create a new input validator
    pub fn new() -> Self {
        Self
    }
    
    /// Validate tool name
    pub fn validate_tool_name(&self, name: &str) -> Result<(), ValidationError> {
        if name.is_empty() {
            return Err(ValidationError::RequiredFieldMissing("name".to_string()));
        }
        
        if name.len() > 100 {
            return Err(ValidationError::StringTooLong(format!("name: max 100 characters, got {}", name.len())));
        }
        
        if name.contains(' ') {
            return Err(ValidationError::InvalidFormat("name: cannot contain spaces".to_string()));
        }
        
        Ok(())
    }
    
    /// Validate tool description
    pub fn validate_tool_description(&self, description: &str) -> Result<(), ValidationError> {
        if description.is_empty() {
            return Err(ValidationError::RequiredFieldMissing("description".to_string()));
        }
        
        if description.len() > 1000 {
            return Err(ValidationError::StringTooLong(format!("description: max 1000 characters, got {}", description.len())));
        }
        
        Ok(())
    }
    
    /// Validate tool
    pub fn validate_tool(&self, tool: &ToolInfo) -> Result<(), ValidationError> {
        self.validate_tool_name(&tool.name)?;
        self.validate_tool_description(&tool.description)?;
        Ok(())
    }
    
    /// Validate search query
    pub fn validate_search_query(&self, query: &str) -> Result<(), ValidationError> {
        if query.is_empty() {
            return Err(ValidationError::RequiredFieldMissing("query".to_string()));
        }
        
        if query.len() > 200 {
            return Err(ValidationError::StringTooLong(format!("query: max 200 characters, got {}", query.len())));
        }
        
        Ok(())
    }
} 