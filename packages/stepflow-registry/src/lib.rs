//! Stepflow Registry - Tool registry for Stepflow Tool System
//!
//! This crate provides the tool registry functionality for the Stepflow Tool System.

// Re-export core types
pub use stepflow_core::*;

// Module declarations
pub mod errors;
pub mod registry;
pub mod registry_impl;
pub mod tool_manager;
pub mod version_manager;
pub mod discovery;
pub mod cache;
pub mod validation;

// Re-export key types
pub use errors::{RegistryError, RegistryResult};
pub use registry::Registry;
pub use registry_impl::RegistryImpl;
pub use tool_manager::ToolManager as ToolManagerImpl;
pub use version_manager::VersionManager as VersionManagerImpl;
pub use discovery::DiscoveryService as DiscoveryServiceImpl;
pub use cache::Cache as CacheImpl;
pub use validation::InputValidator as InputValidatorImpl;

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Create a new registry with database
pub async fn create_registry(db: std::sync::Arc<stepflow_database::SqliteDatabase>) -> Result<RegistryImpl, RegistryError> {
    RegistryImpl::new(db).await
}

impl RegistryImpl {
    /// Get tool manager
    pub fn tool_manager(&self) -> ToolManagerImpl {
        ToolManagerImpl::new(self.tool_repository())
    }
    
    /// Get version manager
    pub fn version_manager(&self) -> VersionManagerImpl {
        VersionManagerImpl::new(self.tool_repository())
    }
    
    /// Get discovery service
    pub fn discovery_service(&self) -> DiscoveryServiceImpl {
        DiscoveryServiceImpl::new(self.tool_repository())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stepflow_database::{SqliteDatabase, MigrationManager};
    use std::sync::Arc;
    use chrono::Utc;
    
    async fn create_test_registry() -> Result<RegistryImpl, RegistryError> {
        let db = Arc::new(SqliteDatabase::new("sqlite::memory:").await
            .map_err(|e| RegistryError::DatabaseError(e.to_string()))?);
        
        // Run migrations to create tables
        MigrationManager::run_migrations(&db).await
            .map_err(|e| RegistryError::DatabaseError(e.to_string()))?;
        
        create_registry(db).await
    }
    
    #[tokio::test]
    async fn test_create_registry() {
        let registry = create_test_registry().await;
        assert!(registry.is_ok());
    }
    
    #[tokio::test]
    async fn test_registry_basic_operations() {
        let registry = create_test_registry().await.unwrap();
        
        let tool = ToolInfo {
            id: ToolId::new(),
            name: "test-tool".to_string(),
            description: "A test tool".to_string(),
            version: ToolVersion::new(1, 0, 0),
            tool_type: ToolType::Python,
            status: ToolStatus::Active,
            author: "test-author".to_string(),
            repository: None,
            documentation: None,
            tags: vec!["test".to_string()],
            capabilities: vec!["process".to_string()],
            configuration_schema: None,
            examples: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        // Test registration
        let tool_id = registry.register_tool(tool.clone()).await.unwrap();
        
        // Test retrieval
        let retrieved_tool = registry.get_tool(&tool_id).await.unwrap();
        assert_eq!(retrieved_tool.name, tool.name);
        assert_eq!(retrieved_tool.description, tool.description);
        
        // Test listing
        let tools = registry.list_tools().await.unwrap();
        assert_eq!(tools.len(), 1);
        
        // Test search
        let results = registry.search_tools("test").await.unwrap();
        assert_eq!(results.len(), 1);
    }
    
    #[tokio::test]
    async fn test_tool_manager() {
        let registry = create_test_registry().await.unwrap();
        let tool_manager = registry.tool_manager();
        
        let tool = ToolInfo {
            id: ToolId::new(),
            name: "test-tool".to_string(),
            description: "A test tool".to_string(),
            version: ToolVersion::new(1, 0, 0),
            tool_type: ToolType::Python,
            status: ToolStatus::Active,
            author: "test-author".to_string(),
            repository: None,
            documentation: None,
            tags: vec!["test".to_string()],
            capabilities: vec!["process".to_string()],
            configuration_schema: None,
            examples: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        // Test tool management
        let tool_id = tool_manager.register_tool(tool.clone()).await.unwrap();
        let retrieved_tool = tool_manager.get_tool(&tool_id).await.unwrap();
        assert_eq!(retrieved_tool.name, tool.name);
    }
    
    #[tokio::test]
    async fn test_version_manager() {
        let registry = create_test_registry().await.unwrap();
        let version_manager = registry.version_manager();
        
        let tool_id = ToolId::new();
        
        // Test version management
        let versions = version_manager.get_versions(&tool_id).await.unwrap();
        assert!(versions.is_empty());
    }
    
    #[tokio::test]
    async fn test_discovery_service() {
        let registry = create_test_registry().await.unwrap();
        let discovery = registry.discovery_service();
        
        // Test discovery
        let tools = discovery.discover_tools().await.unwrap();
        assert!(tools.is_empty());
        
        let results = discovery.search_tools("test").await.unwrap();
        assert!(results.is_empty());
    }
    
    #[tokio::test]
    async fn test_cache_system() {
        let cache = CacheImpl::new(100, std::time::Duration::from_secs(60));
        let key = "test-key".to_string();
        let value = "test-value".to_string();
        
        // Test cache operations
        cache.put(key.clone(), value.clone()).await;
        let retrieved = cache.get(&key).await;
        assert_eq!(retrieved, Some(value));
        
        cache.remove(&key).await;
        let retrieved = cache.get(&key).await;
        assert_eq!(retrieved, None);
    }
    
    #[tokio::test]
    async fn test_validation_system() {
        let validator = InputValidatorImpl::new();
        
        // Test validation
        let result = validator.validate_tool_name("valid-tool-name");
        assert!(result.is_ok());
        
        let result = validator.validate_tool_name("");
        assert!(result.is_err());
        
        let result = validator.validate_tool_name("invalid name with spaces");
        assert!(result.is_err());
    }
} 