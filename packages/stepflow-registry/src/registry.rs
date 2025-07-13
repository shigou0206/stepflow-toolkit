//! Registry trait definitions

use stepflow_core::*;
use crate::errors::*;

/// Registry trait for tool management
#[async_trait::async_trait]
pub trait Registry: Send + Sync {
    /// Register a new tool
    async fn register_tool(&self, tool: ToolInfo) -> RegistryResult<ToolId>;
    
    /// Get a tool by ID
    async fn get_tool(&self, tool_id: &ToolId) -> RegistryResult<ToolInfo>;
    
    /// List all tools
    async fn list_tools(&self) -> RegistryResult<Vec<ToolInfo>>;
    
    /// Search tools by query
    async fn search_tools(&self, query: &str) -> RegistryResult<Vec<ToolInfo>>;
    
    /// Update a tool
    async fn update_tool(&self, tool_id: &ToolId, tool: &ToolInfo) -> RegistryResult<()>;
    
    /// Delete a tool
    async fn delete_tool(&self, tool_id: &ToolId) -> RegistryResult<()>;
    
    /// Get tools by type
    async fn get_tools_by_type(&self, tool_type: &ToolType) -> RegistryResult<Vec<ToolInfo>>;
    
    /// Get tools by status
    async fn get_tools_by_status(&self, status: &ToolStatus) -> RegistryResult<Vec<ToolInfo>>;
    
    /// Check if a tool exists
    async fn tool_exists(&self, tool_id: &ToolId) -> RegistryResult<bool>;
    
    /// Get tool statistics
    async fn get_tool_stats(&self) -> RegistryResult<ToolStats>;
    
    /// Health check for the registry
    async fn health_check(&self) -> RegistryResult<bool>;
} 