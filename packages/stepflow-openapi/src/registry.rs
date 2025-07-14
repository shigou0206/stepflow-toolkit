//! OpenAPI Tool Registry
//!
//! This module provides a comprehensive tool registry for managing,
//! executing, and monitoring OpenAPI tools.

use std::collections::HashMap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::RwLock;

use stepflow_core::types::{Tool, ToolRequest, ToolResponse, ToolInfo};
use stepflow_core::StepflowError;
use crate::srn::Srn;
use crate::generator::{GeneratedToolInfo, ToolGenerator, ToolGenerationRequest, GeneratorError};
use crate::document::DocumentManager;

/// Registry errors
#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("Tool not found: {0}")]
    ToolNotFound(String),
    
    #[error("Tool already registered: {0}")]
    ToolAlreadyRegistered(String),
    
    #[error("Invalid SRN: {0}")]
    InvalidSrn(String),
    
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    
    #[error("Generator error: {0}")]
    GeneratorError(#[from] GeneratorError),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Tool execution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionStats {
    /// Total number of executions
    pub total_executions: u64,
    /// Successful executions
    pub successful_executions: u64,
    /// Failed executions
    pub failed_executions: u64,
    /// Average execution time in milliseconds
    pub avg_execution_time_ms: f64,
    /// Last execution timestamp
    pub last_execution: Option<chrono::DateTime<chrono::Utc>>,
    /// Last execution success
    pub last_execution_success: Option<bool>,
}

impl Default for ToolExecutionStats {
    fn default() -> Self {
        Self {
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            avg_execution_time_ms: 0.0,
            last_execution: None,
            last_execution_success: None,
        }
    }
}

/// Registry tool entry
#[derive(Clone)]
pub struct RegistryToolEntry {
    /// Tool information
    pub info: GeneratedToolInfo,
    /// Execution statistics
    pub stats: Arc<RwLock<ToolExecutionStats>>,
    /// Whether the tool is enabled
    pub enabled: bool,
    /// Registration timestamp
    pub registered_at: chrono::DateTime<chrono::Utc>,
    /// Last access timestamp
    pub last_accessed: Arc<RwLock<chrono::DateTime<chrono::Utc>>>,
}

impl std::fmt::Debug for RegistryToolEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegistryToolEntry")
            .field("info", &self.info)
            .field("enabled", &self.enabled)
            .field("registered_at", &self.registered_at)
            .finish()
    }
}

/// Tool search criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSearchCriteria {
    /// SRN pattern (supports wildcards)
    pub srn_pattern: Option<String>,
    /// Tenant ID filter
    pub tenant_id: Option<String>,
    /// Namespace filter
    pub namespace: Option<String>,
    /// Operation method filter (GET, POST, etc.)
    pub method: Option<String>,
    /// Tags filter
    pub tags: Option<Vec<String>>,
    /// Only enabled tools
    pub enabled_only: bool,
}

impl Default for ToolSearchCriteria {
    fn default() -> Self {
        Self {
            srn_pattern: None,
            tenant_id: None,
            namespace: None,
            method: None,
            tags: None,
            enabled_only: true,
        }
    }
}

/// Tool registry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    /// Maximum number of tools to cache
    pub max_cached_tools: usize,
    /// Enable execution statistics collection
    pub collect_stats: bool,
    /// Auto-cleanup interval for unused tools (in seconds)
    pub cleanup_interval_seconds: u64,
    /// Maximum idle time before tool cleanup (in seconds)
    pub max_idle_seconds: u64,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            max_cached_tools: 1000,
            collect_stats: true,
            cleanup_interval_seconds: 300, // 5 minutes
            max_idle_seconds: 3600,        // 1 hour
        }
    }
}

/// OpenAPI Tool Registry
pub struct OpenApiToolRegistry {
    /// Registered tools
    tools: Arc<RwLock<HashMap<String, RegistryToolEntry>>>,
    /// Tool generator for creating new tools
    generator: Arc<ToolGenerator>,
    /// Registry configuration
    config: RegistryConfig,
    /// Statistics tracking
    global_stats: Arc<RwLock<GlobalRegistryStats>>,
}

/// Global registry statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GlobalRegistryStats {
    /// Total number of registered tools
    pub total_tools: usize,
    /// Total executions across all tools
    pub total_executions: u64,
    /// Registry uptime in seconds
    pub uptime_seconds: u64,
    /// Tools by tenant
    pub tools_by_tenant: HashMap<String, usize>,
    /// Tools by namespace
    pub tools_by_namespace: HashMap<String, usize>,
    /// Most executed tools
    pub most_executed_tools: Vec<(String, u64)>,
}

impl OpenApiToolRegistry {
    /// Create a new tool registry
    pub fn new(generator: Arc<ToolGenerator>, config: RegistryConfig) -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
            generator,
            config,
            global_stats: Arc::new(RwLock::new(GlobalRegistryStats::default())),
        }
    }

    /// Create with default configuration
    pub fn with_default_config(generator: Arc<ToolGenerator>) -> Self {
        Self::new(generator, RegistryConfig::default())
    }

    /// Register a tool from generation info
    pub async fn register_tool(&self, tool_info: GeneratedToolInfo) -> Result<(), RegistryError> {
        let srn = tool_info.srn.to_string();
        let mut tools = self.tools.write().await;

        if tools.contains_key(&srn) {
            return Err(RegistryError::ToolAlreadyRegistered(srn));
        }

        let entry = RegistryToolEntry {
            info: tool_info,
            stats: Arc::new(RwLock::new(ToolExecutionStats::default())),
            enabled: true,
            registered_at: chrono::Utc::now(),
            last_accessed: Arc::new(RwLock::new(chrono::Utc::now())),
        };

        tools.insert(srn, entry);

        // Update global stats
        self.update_global_stats().await;

        Ok(())
    }

    /// Generate and register tools from a document
    pub async fn generate_and_register_tools(
        &self,
        request: ToolGenerationRequest,
    ) -> Result<Vec<String>, RegistryError> {
        let result = self.generator.generate_tools(request).await?;
        let mut registered_srns = Vec::new();

        for srn in &result.tool_srns {
            if let Some(tool_info) = self.generator.get_tool_info(srn) {
                self.register_tool(tool_info).await?;
                registered_srns.push(srn.clone());
            }
        }

        Ok(registered_srns)
    }

    /// Unregister a tool
    pub async fn unregister_tool(&self, srn: &str) -> Result<(), RegistryError> {
        let mut tools = self.tools.write().await;
        tools.remove(srn);

        // Update global stats
        self.update_global_stats().await;

        Ok(())
    }

    /// Get a tool by SRN
    pub async fn get_tool(&self, srn: &str) -> Result<Arc<dyn Tool>, RegistryError> {
        let tools = self.tools.read().await;
        let entry = tools.get(srn)
            .ok_or_else(|| RegistryError::ToolNotFound(srn.to_string()))?;

        if !entry.enabled {
            return Err(RegistryError::PermissionDenied(format!("Tool {} is disabled", srn)));
        }

        // Update last accessed time
        *entry.last_accessed.write().await = chrono::Utc::now();

        Ok(entry.info.tool.clone())
    }

    /// Execute a tool by SRN
    pub async fn execute_tool(
        &self,
        srn: &str,
        request: ToolRequest,
    ) -> Result<ToolResponse, RegistryError> {
        let tool = self.get_tool(srn).await?;
        let start_time = std::time::Instant::now();

        // Execute the tool
        let result = tool.execute(request)
            .await
            .map_err(|e| RegistryError::ExecutionFailed(e.to_string()))?;

        // Update statistics if enabled
        if self.config.collect_stats {
            self.update_execution_stats(srn, &result, start_time.elapsed()).await?;
        }

        Ok(result)
    }

    /// Search tools by criteria
    pub async fn search_tools(&self, criteria: &ToolSearchCriteria) -> Result<Vec<String>, RegistryError> {
        let tools = self.tools.read().await;
        let mut matching_tools = Vec::new();

        for (srn, entry) in tools.iter() {
            // Check enabled filter
            if criteria.enabled_only && !entry.enabled {
                continue;
            }

            // Check SRN pattern
            if let Some(pattern) = &criteria.srn_pattern {
                if let Ok(parsed_srn) = Srn::parse(srn) {
                    if !parsed_srn.matches_pattern(pattern) {
                        continue;
                    }
                } else {
                    continue;
                }
            }

            // Check tenant filter
            if let Some(tenant) = &criteria.tenant_id {
                if entry.info.srn.tenant() != tenant {
                    continue;
                }
            }

            // Check namespace filter
            if let Some(namespace) = &criteria.namespace {
                if entry.info.srn.namespace() != namespace {
                    continue;
                }
            }

            // Check method filter
            if let Some(method) = &criteria.method {
                if entry.info.operation.method != *method {
                    continue;
                }
            }

            // Check tags filter
            if let Some(tags) = &criteria.tags {
                let operation_tags = &entry.info.operation.tags;
                if !tags.iter().any(|tag| operation_tags.contains(tag)) {
                    continue;
                }
            }

            matching_tools.push(srn.clone());
        }

        Ok(matching_tools)
    }

    /// List all registered tools
    pub async fn list_tools(&self) -> Result<Vec<String>, RegistryError> {
        let tools = self.tools.read().await;
        Ok(tools.keys().cloned().collect())
    }

    /// Get tool information
    pub async fn get_tool_info(&self, srn: &str) -> Result<ToolInfo, RegistryError> {
        let tools = self.tools.read().await;
        let entry = tools.get(srn)
            .ok_or_else(|| RegistryError::ToolNotFound(srn.to_string()))?;

        let tool_info = entry.info.tool.get_info()
            .await
            .map_err(|e| RegistryError::InternalError(e.to_string()))?;

        Ok(tool_info)
    }

    /// Get tool execution statistics
    pub async fn get_tool_stats(&self, srn: &str) -> Result<ToolExecutionStats, RegistryError> {
        let tools = self.tools.read().await;
        let entry = tools.get(srn)
            .ok_or_else(|| RegistryError::ToolNotFound(srn.to_string()))?;

        let stats = entry.stats.read().await;
        Ok(stats.clone())
    }

    /// Enable or disable a tool
    pub async fn set_tool_enabled(&self, srn: &str, enabled: bool) -> Result<(), RegistryError> {
        let mut tools = self.tools.write().await;
        let entry = tools.get_mut(srn)
            .ok_or_else(|| RegistryError::ToolNotFound(srn.to_string()))?;

        entry.enabled = enabled;
        Ok(())
    }

    /// Get global registry statistics
    pub async fn get_global_stats(&self) -> GlobalRegistryStats {
        let stats = self.global_stats.read().await;
        stats.clone()
    }

    /// Update execution statistics for a tool
    async fn update_execution_stats(
        &self,
        srn: &str,
        response: &ToolResponse,
        execution_time: std::time::Duration,
    ) -> Result<(), RegistryError> {
        let tools = self.tools.read().await;
        let entry = tools.get(srn)
            .ok_or_else(|| RegistryError::ToolNotFound(srn.to_string()))?;

        let mut stats = entry.stats.write().await;
        
        stats.total_executions += 1;
        if response.success {
            stats.successful_executions += 1;
        } else {
            stats.failed_executions += 1;
        }

        // Update average execution time
        let new_time_ms = execution_time.as_millis() as f64;
        if stats.total_executions == 1 {
            stats.avg_execution_time_ms = new_time_ms;
        } else {
            stats.avg_execution_time_ms = 
                (stats.avg_execution_time_ms * (stats.total_executions - 1) as f64 + new_time_ms) 
                / stats.total_executions as f64;
        }

        stats.last_execution = Some(chrono::Utc::now());
        stats.last_execution_success = Some(response.success);

        // Update global stats
        let mut global_stats = self.global_stats.write().await;
        global_stats.total_executions += 1;

        Ok(())
    }

    /// Update global registry statistics
    async fn update_global_stats(&self) {
        let tools = self.tools.read().await;
        let mut global_stats = self.global_stats.write().await;

        global_stats.total_tools = tools.len();
        global_stats.tools_by_tenant.clear();
        global_stats.tools_by_namespace.clear();

        for (_, entry) in tools.iter() {
            let tenant = entry.info.srn.tenant();
            let namespace = entry.info.srn.namespace();

            *global_stats.tools_by_tenant.entry(tenant.to_string()).or_insert(0) += 1;
            *global_stats.tools_by_namespace.entry(namespace.to_string()).or_insert(0) += 1;
        }

        // Update most executed tools
        let mut tool_executions: Vec<(String, u64)> = Vec::new();
        for (srn, entry) in tools.iter() {
            let stats = entry.stats.read().await;
            tool_executions.push((srn.clone(), stats.total_executions));
        }
        
        tool_executions.sort_by(|a, b| b.1.cmp(&a.1));
        tool_executions.truncate(10); // Top 10
        global_stats.most_executed_tools = tool_executions;
    }

    /// Cleanup unused tools
    pub async fn cleanup_unused_tools(&self) -> Result<usize, RegistryError> {
        let now = chrono::Utc::now();
        let max_idle_duration = chrono::Duration::seconds(self.config.max_idle_seconds as i64);
        
        let mut tools = self.tools.write().await;
        let mut to_remove = Vec::new();

        for (srn, entry) in tools.iter() {
            let last_accessed = entry.last_accessed.read().await;
            if now.signed_duration_since(*last_accessed) > max_idle_duration {
                to_remove.push(srn.clone());
            }
        }

        let removed_count = to_remove.len();
        for srn in to_remove {
            tools.remove(&srn);
        }

        if removed_count > 0 {
            drop(tools); // Release the lock before updating global stats
            self.update_global_stats().await;
        }

        Ok(removed_count)
    }

    /// Start background cleanup task
    pub fn start_cleanup_task(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        let cleanup_interval = std::time::Duration::from_secs(self.config.cleanup_interval_seconds);
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(cleanup_interval);
            
            loop {
                interval.tick().await;
                if let Err(e) = self.cleanup_unused_tools().await {
                    eprintln!("Cleanup error: {}", e);
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::InMemoryDocumentStorage;
    use crate::generator::ToolGenerator;

    async fn create_test_registry() -> Arc<OpenApiToolRegistry> {
        let storage = Box::new(InMemoryDocumentStorage::default());
        let doc_manager = Arc::new(DocumentManager::new(storage));
        let generator = Arc::new(ToolGenerator::with_default_config(doc_manager));
        Arc::new(OpenApiToolRegistry::with_default_config(generator))
    }

    #[tokio::test]
    async fn test_tool_registration() {
        let registry = create_test_registry().await;
        
        // This test would need actual tool generation, but for now we'll test the structure
        let tools = registry.list_tools().await.unwrap();
        assert!(tools.is_empty());
        
        let stats = registry.get_global_stats().await;
        assert_eq!(stats.total_tools, 0);
    }

    #[tokio::test]
    async fn test_tool_search() {
        let registry = create_test_registry().await;
        
        let criteria = ToolSearchCriteria {
            tenant_id: Some("tenant-123".to_string()),
            ..Default::default()
        };
        
        let results = registry.search_tools(&criteria).await.unwrap();
        assert!(results.is_empty()); // No tools registered yet
    }

    #[tokio::test]
    async fn test_tool_enabling() {
        let registry = create_test_registry().await;
        
        // Test enabling/disabling non-existent tool
        let result = registry.set_tool_enabled("non-existent", false).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cleanup() {
        let registry = create_test_registry().await;
        
        let removed = registry.cleanup_unused_tools().await.unwrap();
        assert_eq!(removed, 0); // No tools to clean up
    }
} 