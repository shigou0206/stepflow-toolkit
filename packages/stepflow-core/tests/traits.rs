use stepflow_core::traits::*;
use stepflow_core::types::*;
use stepflow_core::errors::*;
use stepflow_core::types::ToolStats; // Explicit import to resolve ambiguity
use async_trait::async_trait;
use std::collections::HashMap;
use chrono::Utc;

// Simple mock implementations for testing

#[derive(Debug)]
pub struct MockTool {
    pub id: ToolId,
    pub name: String,
    pub description: String,
}

#[async_trait]
impl Tool for MockTool {
    async fn get_info(&self) -> Result<ToolInfo, StepflowError> {
        Ok(ToolInfo {
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            version: ToolVersion::new(1, 0, 0),
            tool_type: ToolType::Custom("mock".to_string()),
            status: ToolStatus::Active,
            author: "test".to_string(),
            repository: None,
            documentation: None,
            tags: vec!["test".to_string()],
            capabilities: vec!["mock".to_string()],
            configuration_schema: None,
            examples: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    async fn execute(&self, _request: ToolRequest) -> Result<ToolResponse, StepflowError> {
        Ok(ToolResponse {
            success: true,
            output: Some(serde_json::json!({"result": "mock success"})),
            error: None,
            execution_time: 100,
            metadata: HashMap::new(),
        })
    }

    async fn validate_input(&self, _input: &serde_json::Value) -> Result<bool, StepflowError> {
        Ok(true)
    }

    async fn get_configuration_schema(&self) -> Result<Option<serde_json::Value>, StepflowError> {
        Ok(None)
    }

    async fn test(&self) -> Result<bool, StepflowError> {
        Ok(true)
    }
}

#[derive(Debug)]
pub struct MockToolExecutor;

#[async_trait]
impl ToolExecutor for MockToolExecutor {
    async fn execute_tool(&self, _request: ToolRequest) -> Result<ToolResponse, StepflowError> {
        Ok(ToolResponse {
            success: true,
            output: Some(serde_json::json!({"result": "mock execution"})),
            error: None,
            execution_time: 150,
            metadata: HashMap::new(),
        })
    }

    async fn execute_tool_with_timeout(
        &self,
        request: ToolRequest,
        _timeout: std::time::Duration,
    ) -> Result<ToolResponse, StepflowError> {
        self.execute_tool(request).await
    }

    async fn execute_tools(&self, requests: Vec<ToolRequest>) -> Result<Vec<ToolResponse>, StepflowError> {
        let mut responses = Vec::new();
        for request in requests {
            responses.push(self.execute_tool(request).await?);
        }
        Ok(responses)
    }

    async fn cancel_execution(&self, _execution_id: &ExecutionId) -> Result<(), StepflowError> {
        Ok(())
    }

    async fn get_execution_status(&self, _execution_id: &ExecutionId) -> Result<ExecutionStatus, StepflowError> {
        Ok(ExecutionStatus::Completed)
    }

    async fn get_execution_result(&self, _execution_id: &ExecutionId) -> Result<ExecutionResult, StepflowError> {
        Ok(ExecutionResult {
            success: true,
            output: Some(serde_json::json!({"result": "mock result"})),
            error: None,
            logs: vec![],
            metrics: HashMap::new(),
            metadata: HashMap::new(),
        })
    }
}

// Tests

#[tokio::test]
async fn test_mock_tool() {
    let tool = MockTool {
        id: ToolId::from_string("mock-tool".to_string()),
        name: "Mock Tool".to_string(),
        description: "A mock tool for testing".to_string(),
    };

    let info = tool.get_info().await.unwrap();
    assert_eq!(info.name, "Mock Tool");
    assert_eq!(info.tool_type, ToolType::Custom("mock".to_string()));

    let request = ToolRequest {
        tool_id: tool.id.clone(),
        input: serde_json::json!({}),
        configuration: None,
        metadata: HashMap::new(),
    };
    let response = tool.execute(request).await.unwrap();
    assert!(response.success);
    assert!(response.output.is_some());
}

#[tokio::test]
async fn test_mock_tool_executor() {
    let executor = MockToolExecutor;
    
    let request = ToolRequest {
        tool_id: ToolId::from_string("test-tool".to_string()),
        input: serde_json::json!({"test": "data"}),
        configuration: None,
        metadata: HashMap::new(),
    };

    let response = executor.execute_tool(request).await.unwrap();
    assert!(response.success);
    assert!(response.output.is_some());
}

#[tokio::test]
async fn test_tool_filter() {
    let filter = ToolFilter {
        tool_type: Some(ToolType::Custom("test".to_string())),
        status: Some(ToolStatus::Active),
        tags: vec!["test".to_string()],
        author: Some("test-author".to_string()),
        created_after: Some(Utc::now()),
        created_before: Some(Utc::now()),
    };

    assert!(filter.tool_type.is_some());
    assert!(filter.status.is_some());
    assert_eq!(filter.tags.len(), 1);
    assert!(filter.author.is_some());
}

#[tokio::test]
async fn test_execution_filter() {
    let filter = ExecutionFilter {
        tool_id: Some(ToolId::new()),
        tenant_id: Some(TenantId::new()),
        user_id: Some(UserId::new()),
        status: Some(ExecutionStatus::Pending),
        started_after: Some(Utc::now()),
        started_before: Some(Utc::now()),
    };

    assert!(filter.tool_id.is_some());
    assert!(filter.tenant_id.is_some());
    assert!(filter.user_id.is_some());
    assert!(filter.status.is_some());
}

#[tokio::test]
async fn test_credentials() {
    let credentials = Credentials {
        username: "test-user".to_string(),
        password: "test-password".to_string(),
        tenant_id: Some(TenantId::new()),
    };

    assert_eq!(credentials.username, "test-user");
    assert_eq!(credentials.password, "test-password");
    assert!(credentials.tenant_id.is_some());
}

#[tokio::test]
async fn test_auth_result() {
    let auth_result = AuthResult {
        user_id: UserId::new(),
        tenant_id: TenantId::new(),
        token: "test-token".to_string(),
        refresh_token: Some("refresh-token".to_string()),
        expires_at: Utc::now(),
        permissions: vec!["read".to_string(), "write".to_string()],
    };

    assert!(!auth_result.token.is_empty());
    assert!(auth_result.refresh_token.is_some());
    assert_eq!(auth_result.permissions.len(), 2);
}

#[tokio::test]
async fn test_permission() {
    let permission = Permission {
        resource: "tool".to_string(),
        action: "execute".to_string(),
        granted_at: Utc::now(),
        granted_by: UserId::new(),
    };

    assert_eq!(permission.resource, "tool");
    assert_eq!(permission.action, "execute");
}

#[tokio::test]
async fn test_event() {
    let event = Event {
        name: "tool_executed".to_string(),
        timestamp: Utc::now(),
        user_id: Some(UserId::new()),
        tenant_id: Some(TenantId::new()),
        data: HashMap::new(),
    };

    assert_eq!(event.name, "tool_executed");
    assert!(event.user_id.is_some());
    assert!(event.tenant_id.is_some());
}

#[tokio::test]
async fn test_metric_filter() {
    let filter = MetricFilter {
        name: Some("execution_time".to_string()),
        labels: HashMap::new(),
        start_time: Some(Utc::now()),
        end_time: Some(Utc::now()),
    };

    assert!(filter.name.is_some());
    assert!(filter.start_time.is_some());
    assert!(filter.end_time.is_some());
}

#[tokio::test]
async fn test_event_filter() {
    let filter = EventFilter {
        name: Some("tool_executed".to_string()),
        user_id: Some(UserId::new()),
        tenant_id: Some(TenantId::new()),
        start_time: Some(Utc::now()),
        end_time: Some(Utc::now()),
    };

    assert!(filter.name.is_some());
    assert!(filter.user_id.is_some());
    assert!(filter.tenant_id.is_some());
}

#[tokio::test]
async fn test_cache_stats() {
    let stats = CacheStats {
        hits: 100,
        misses: 50,
        size: 1000,
        max_size: 2000,
    };

    assert_eq!(stats.hits, 100);
    assert_eq!(stats.misses, 50);
    assert_eq!(stats.size, 1000);
    assert_eq!(stats.max_size, 2000);
}

#[tokio::test]
async fn test_database_stats() {
    let stats = DatabaseStats {
        connections: 10,
        active_connections: 5,
        idle_connections: 5,
        total_queries: 1000,
        slow_queries: 10,
        errors: 2,
    };

    assert_eq!(stats.connections, 10);
    assert_eq!(stats.active_connections, 5);
    assert_eq!(stats.idle_connections, 5);
    assert_eq!(stats.total_queries, 1000);
    assert_eq!(stats.slow_queries, 10);
    assert_eq!(stats.errors, 2);
}

#[tokio::test]
async fn test_tool_stats() {
    let stats = ToolStats {
        total_tools: 100,
        active_tools: 95,
        deprecated_tools: 3,
        error_tools: 2,
    };

    assert_eq!(stats.total_tools, 100);
    assert_eq!(stats.active_tools, 95);
    assert_eq!(stats.deprecated_tools, 3);
    assert_eq!(stats.error_tools, 2);
}

#[tokio::test]
async fn test_tenant_filter() {
    let filter = TenantFilter {
        domain: Some("example.com".to_string()),
        created_after: Some(Utc::now()),
        created_before: Some(Utc::now()),
    };

    assert!(filter.domain.is_some());
    assert!(filter.created_after.is_some());
    assert!(filter.created_before.is_some());
}

#[tokio::test]
async fn test_user_filter() {
    let filter = UserFilter {
        tenant_id: Some(TenantId::new()),
        role: Some(UserRole::Admin),
        created_after: Some(Utc::now()),
        created_before: Some(Utc::now()),
    };

    assert!(filter.tenant_id.is_some());
    assert!(filter.role.is_some());
    assert!(filter.created_after.is_some());
    assert!(filter.created_before.is_some());
} 