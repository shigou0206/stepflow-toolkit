//! Version manager implementation

use std::sync::Arc;
use stepflow_core::*;
use stepflow_database::ToolRepository;
use crate::errors::*;

/// Version manager implementation
pub struct VersionManager {
    tool_repository: Arc<ToolRepository>,
}

impl VersionManager {
    /// Create a new version manager
    pub fn new(tool_repository: Arc<ToolRepository>) -> Self {
        Self { tool_repository }
    }
    
    /// Get versions for a tool
    pub async fn get_versions(&self, tool_id: &ToolId) -> RegistryResult<Vec<ToolVersion>> {
        match self.tool_repository.get_tool(tool_id).await? {
            Some(tool) => Ok(vec![tool.version]),
            None => Ok(vec![]),
        }
    }
    
    /// Get latest version for a tool
    pub async fn get_latest_version(&self, tool_id: &ToolId) -> RegistryResult<Option<ToolVersion>> {
        match self.tool_repository.get_tool(tool_id).await? {
            Some(tool) => Ok(Some(tool.version)),
            None => Ok(None),
        }
    }
    
    /// Check if version exists
    pub async fn version_exists(&self, tool_id: &ToolId, version: &ToolVersion) -> RegistryResult<bool> {
        match self.tool_repository.get_tool(tool_id).await? {
            Some(tool) => Ok(tool.version == *version),
            None => Ok(false),
        }
    }
} 