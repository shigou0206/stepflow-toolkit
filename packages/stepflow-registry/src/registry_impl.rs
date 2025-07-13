//! Registry implementation

use std::sync::Arc;
use stepflow_core::*;
use stepflow_database::{SqliteDatabase, ToolRepository};
use crate::errors::*;
use crate::registry::*;

/// Registry implementation
pub struct RegistryImpl {
    tool_repository: Arc<ToolRepository>,
}

impl RegistryImpl {
    /// Create a new registry implementation
    pub async fn new(db: Arc<SqliteDatabase>) -> RegistryResult<Self> {
        Ok(Self {
            tool_repository: Arc::new(ToolRepository::new(db.as_ref().clone())),
        })
    }
    
    /// Get tool repository
    pub fn tool_repository(&self) -> Arc<ToolRepository> {
        self.tool_repository.clone()
    }
}

#[async_trait::async_trait]
impl Registry for RegistryImpl {
    async fn register_tool(&self, tool: ToolInfo) -> RegistryResult<ToolId> {
        self.tool_repository.create_tool(&tool).await?;
        Ok(tool.id)
    }
    
    async fn get_tool(&self, tool_id: &ToolId) -> RegistryResult<ToolInfo> {
        self.tool_repository.get_tool(tool_id).await?
            .ok_or_else(|| RegistryError::ToolNotFound(tool_id.to_string()))
    }
    
    async fn list_tools(&self) -> RegistryResult<Vec<ToolInfo>> {
        self.tool_repository.list_tools(None).await.map_err(Into::into)
    }
    
    async fn search_tools(&self, query: &str) -> RegistryResult<Vec<ToolInfo>> {
        self.tool_repository.search_tools(query).await.map_err(Into::into)
    }
    
    async fn update_tool(&self, tool_id: &ToolId, tool: &ToolInfo) -> RegistryResult<()> {
        self.tool_repository.update_tool(tool_id, tool).await.map_err(Into::into)
    }
    
    async fn delete_tool(&self, tool_id: &ToolId) -> RegistryResult<()> {
        self.tool_repository.delete_tool(tool_id).await.map_err(Into::into)
    }
    
    async fn get_tools_by_type(&self, tool_type: &ToolType) -> RegistryResult<Vec<ToolInfo>> {
        self.tool_repository.get_tools_by_type(tool_type).await.map_err(Into::into)
    }
    
    async fn get_tools_by_status(&self, status: &ToolStatus) -> RegistryResult<Vec<ToolInfo>> {
        self.tool_repository.get_tools_by_status(status).await.map_err(Into::into)
    }
    
    async fn tool_exists(&self, tool_id: &ToolId) -> RegistryResult<bool> {
        self.tool_repository.tool_exists(tool_id).await.map_err(Into::into)
    }
    
    async fn get_tool_stats(&self) -> RegistryResult<ToolStats> {
        self.tool_repository.get_tool_stats().await.map_err(Into::into)
    }
    
    async fn health_check(&self) -> RegistryResult<bool> {
        // Simple health check - try to perform a basic database operation
        match self.tool_repository.list_tools(None).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
} 