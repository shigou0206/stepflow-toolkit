//! Tests for types module

use stepflow_core::types::*;
use chrono::Utc;

#[tokio::test]
async fn test_tool_id() {
    let id1 = ToolId::new();
    let id2 = ToolId::new();
    
    assert_ne!(id1, id2);
    assert_eq!(id1.as_str(), id1.as_str());
    
    let id3 = ToolId::from_string("test-id".to_string());
    assert_eq!(id3.as_str(), "test-id");
}

#[tokio::test]
async fn test_tool_version() {
    let version = ToolVersion::new(1, 2, 3);
    assert_eq!(version.to_string(), "1.2.3");
    
    let version_with_pre = version.with_pre_release("alpha".to_string());
    assert_eq!(version_with_pre.to_string(), "1.2.3-alpha");
    
    let version_with_build = version_with_pre.with_build("123".to_string());
    assert_eq!(version_with_build.to_string(), "1.2.3-alpha+123");
}

#[tokio::test]
async fn test_tool_type() {
    assert_eq!(ToolType::OpenAPI.to_string(), "openapi");
    assert_eq!(ToolType::Python.to_string(), "python");
    assert_eq!(ToolType::Custom("my-tool".to_string()).to_string(), "custom:my-tool");
}

#[tokio::test]
async fn test_tool_status() {
    assert_eq!(ToolStatus::Active.to_string(), "active");
    assert_eq!(ToolStatus::Error.to_string(), "error");
}

#[tokio::test]
async fn test_tool_info() {
    let info = ToolInfo {
        id: ToolId::new(),
        name: "test-tool".to_string(),
        description: "A test tool".to_string(),
        version: ToolVersion::new(1, 0, 0),
        tool_type: ToolType::Python,
        status: ToolStatus::Active,
        author: "test-author".to_string(),
        repository: Some("https://github.com/test/tool".to_string()),
        documentation: Some("https://docs.test.com".to_string()),
        tags: vec!["test".to_string(), "example".to_string()],
        capabilities: vec!["process".to_string()],
        configuration_schema: None,
        examples: vec![],
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    assert_eq!(info.name, "test-tool");
    assert_eq!(info.tool_type, ToolType::Python);
    assert_eq!(info.status, ToolStatus::Active);
}

#[tokio::test]
async fn test_execution_id() {
    let id1 = ExecutionId::new();
    let id2 = ExecutionId::new();
    
    assert_ne!(id1, id2);
    
    let id3 = ExecutionId::from_string("exec-123".to_string());
    assert_eq!(id3.as_str(), "exec-123");
}

#[tokio::test]
async fn test_execution_status() {
    assert_eq!(ExecutionStatus::Pending.to_string(), "pending");
    assert_eq!(ExecutionStatus::Completed.to_string(), "completed");
    assert_eq!(ExecutionStatus::Failed.to_string(), "failed");
}

#[tokio::test]
async fn test_user_id() {
    let id1 = UserId::new();
    let id2 = UserId::new();
    
    assert_ne!(id1, id2);
    
    let id3 = UserId::from_string("user-123".to_string());
    assert_eq!(id3.as_str(), "user-123");
}

#[tokio::test]
async fn test_user_role() {
    assert_eq!(UserRole::Admin.to_string(), "admin");
    assert_eq!(UserRole::User.to_string(), "user");
    assert_eq!(UserRole::Custom("moderator".to_string()).to_string(), "custom:moderator");
}

#[tokio::test]
async fn test_tenant_id() {
    let id1 = TenantId::new();
    let id2 = TenantId::new();
    
    assert_ne!(id1, id2);
    
    let id3 = TenantId::from_string("tenant-123".to_string());
    assert_eq!(id3.as_str(), "tenant-123");
}

#[tokio::test]
async fn test_query_default() {
    let query = Query::default();
    assert_eq!(query.filters.len(), 0);
    assert_eq!(query.sorts.len(), 0);
    assert_eq!(query.pagination.page, 1);
    assert_eq!(query.pagination.size, 20);
}

#[tokio::test]
async fn test_pagination_default() {
    let pagination = Pagination::default();
    assert_eq!(pagination.page, 1);
    assert_eq!(pagination.size, 20);
    assert_eq!(pagination.total, None);
}

#[tokio::test]
async fn test_filter_operator() {
    assert_eq!(FilterOperator::Equal.to_string(), "eq");
    assert_eq!(FilterOperator::Contains.to_string(), "contains");
    assert_eq!(FilterOperator::IsNull.to_string(), "is_null");
}

#[tokio::test]
async fn test_sort_direction() {
    assert_eq!(SortDirection::Asc.to_string(), "asc");
    assert_eq!(SortDirection::Desc.to_string(), "desc");
} 