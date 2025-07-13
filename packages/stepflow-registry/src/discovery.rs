//! Discovery service implementation

use std::sync::Arc;
use stepflow_core::*;
use stepflow_database::ToolRepository;
use crate::errors::*;

/// Discovery service implementation
pub struct DiscoveryService {
    tool_repository: Arc<ToolRepository>,
}

impl DiscoveryService {
    /// Create a new discovery service
    pub fn new(tool_repository: Arc<ToolRepository>) -> Self {
        Self { tool_repository }
    }
    
    /// Discover tools
    pub async fn discover_tools(&self) -> RegistryResult<Vec<ToolInfo>> {
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
    
    /// Get popular tools
    pub async fn get_popular_tools(&self, limit: usize) -> RegistryResult<Vec<ToolInfo>> {
        let mut tools = self.tool_repository.list_tools(None).await?;
        tools.truncate(limit);
        Ok(tools)
    }
    
    /// Get recent tools
    pub async fn get_recent_tools(&self, limit: usize) -> RegistryResult<Vec<ToolInfo>> {
        let mut tools = self.tool_repository.list_tools(None).await?;
        tools.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        tools.truncate(limit);
        Ok(tools)
    }
} 