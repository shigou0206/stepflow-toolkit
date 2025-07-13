//! Tool manager implementation

use std::sync::Arc;
use stepflow_core::*;
use stepflow_database::ToolRepository;
use crate::errors::*;

/// Tool manager implementation
pub struct ToolManager {
    tool_repository: Arc<ToolRepository>,
}

impl ToolManager {
    /// Create a new tool manager
    pub fn new(tool_repository: Arc<ToolRepository>) -> Self {
        Self { tool_repository }
    }
    
    /// Register a tool
    pub async fn register_tool(&self, tool: ToolInfo) -> RegistryResult<ToolId> {
        self.tool_repository.create_tool(&tool).await?;
        Ok(tool.id)
    }
    
    /// Get a tool by ID
    pub async fn get_tool(&self, tool_id: &ToolId) -> RegistryResult<ToolInfo> {
        self.tool_repository.get_tool(tool_id).await?
            .ok_or_else(|| RegistryError::ToolNotFound(tool_id.to_string()))
    }
    
    /// Update a tool
    pub async fn update_tool(&self, tool_id: &ToolId, tool: &ToolInfo) -> RegistryResult<()> {
        self.tool_repository.update_tool(tool_id, tool).await.map_err(Into::into)
    }
    
    /// Delete a tool
    pub async fn delete_tool(&self, tool_id: &ToolId) -> RegistryResult<()> {
        self.tool_repository.delete_tool(tool_id).await.map_err(Into::into)
    }
    
    /// List all tools
    pub async fn list_tools(&self) -> RegistryResult<Vec<ToolInfo>> {
        self.tool_repository.list_tools(None).await.map_err(Into::into)
    }
    
    /// Search tools
    pub async fn search_tools(&self, query: &str) -> RegistryResult<Vec<ToolInfo>> {
        self.tool_repository.search_tools(query).await.map_err(Into::into)
    }
    
    /// Get tools by type
    pub async fn get_tools_by_type(&self, tool_type: &ToolType) -> RegistryResult<Vec<ToolInfo>> {
        self.tool_repository.get_tools_by_type(tool_type).await.map_err(Into::into)
    }
    
    /// Get tools by status
    pub async fn get_tools_by_status(&self, status: &ToolStatus) -> RegistryResult<Vec<ToolInfo>> {
        self.tool_repository.get_tools_by_status(status).await.map_err(Into::into)
    }
    
    /// Check if tool exists
    pub async fn tool_exists(&self, tool_id: &ToolId) -> RegistryResult<bool> {
        match self.tool_repository.get_tool(tool_id).await {
            Ok(Some(_)) => Ok(true),
            Ok(None) => Ok(false),
            Err(e) => Err(e.into()),
        }
    }
} 