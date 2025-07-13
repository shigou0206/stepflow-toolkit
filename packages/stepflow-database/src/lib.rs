//! Stepflow Database - Database layer for Stepflow Tool System
//!
//! This crate provides the database abstraction layer and SQLite implementation
//! for the Stepflow Tool System.

pub mod connection;
pub mod migrations;
pub mod repositories;
pub mod models;
pub mod utils;

pub use connection::*;
pub use migrations::*;
pub use repositories::*;
pub use models::*; 

#[cfg(test)]
mod tests {
    use super::*;
    use stepflow_core::*;
    use std::collections::HashMap;

    async fn create_test_database() -> StepflowResult<SqliteDatabase> {
        // 使用内存数据库避免文件权限问题
        let database = SqliteDatabase::new("sqlite::memory:").await?;
        
        // 运行迁移
        MigrationManager::run_migrations(&database).await?;
        
        Ok(database)
    }

    #[tokio::test]
    async fn test_database_connection() {
        let database = create_test_database().await.unwrap();
        assert!(database.health_check().await.unwrap());
    }

    #[tokio::test]
    async fn test_tool_repository() {
        let database = create_test_database().await.unwrap();
        let tool_repo = ToolRepository::new(database);

        // 创建测试工具
        let tool_info = ToolInfo {
            id: ToolId::new(),
            name: "test-tool".to_string(),
            description: "Test tool description".to_string(),
            version: ToolVersion::new(1, 0, 0),
            tool_type: ToolType::Custom("test".to_string()),
            status: ToolStatus::Active,
            author: "test-author".to_string(),
            repository: Some("https://github.com/test/test".to_string()),
            documentation: None,
            tags: vec!["test".to_string()],
            capabilities: vec!["test-capability".to_string()],
            configuration_schema: None,
            examples: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // 测试创建工具
        tool_repo.create_tool(&tool_info).await.unwrap();

        // 测试获取工具 - 添加调试信息
        let result = tool_repo.database().execute("SELECT * FROM tools WHERE id = ?", &[serde_json::Value::String(tool_info.id.as_str().to_string())]).await.unwrap();
        println!("Query result: {:?}", result);
        
        let retrieved_tool = tool_repo.get_tool(&tool_info.id).await.unwrap();
        assert!(retrieved_tool.is_some(), "Tool should be found after creation");
        
        let retrieved_tool = retrieved_tool.unwrap();
        assert_eq!(retrieved_tool.name, tool_info.name);
        assert_eq!(retrieved_tool.description, tool_info.description);

        // 测试列出工具
        let tools = tool_repo.list_tools(None).await.unwrap();
        assert_eq!(tools.len(), 1);

        // 测试搜索工具
        let search_results = tool_repo.search_tools("test").await.unwrap();
        assert_eq!(search_results.len(), 1);
    }

    #[tokio::test]
    async fn test_user_repository() {
        let database = create_test_database().await.unwrap();
        let tenant_repo = TenantRepository::new(database.clone());
        let user_repo = UserRepository::new(database);

        // 首先创建租户
        let tenant_id = TenantId::new();
        let tenant_info = TenantInfo {
            id: tenant_id.clone(),
            name: "Test Tenant".to_string(),
            description: "Test tenant description".to_string(),
            domain: Some("test.example.com".to_string()),
            settings: HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        tenant_repo.create_tenant(&tenant_info).await.unwrap();

        // 创建测试用户
        let user_info = UserInfo {
            id: UserId::new(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            role: UserRole::User,
            tenant_id: tenant_id.clone(),
            settings: HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // 测试用户注册
        user_repo.register_user(&user_info, "password123").await.unwrap();

        // 测试获取用户
        let retrieved_user = user_repo.get_user(&user_info.id).await.unwrap();
        assert!(retrieved_user.is_some());
        
        let retrieved_user = retrieved_user.unwrap();
        assert_eq!(retrieved_user.username, user_info.username);
        assert_eq!(retrieved_user.email, user_info.email);

        // 测试密码验证
        let password_valid = user_repo.verify_user_password(&user_info.id, "password123").await.unwrap();
        assert!(password_valid);

        let password_invalid = user_repo.verify_user_password(&user_info.id, "wrongpassword").await.unwrap();
        assert!(!password_invalid);

        // 测试通过用户名获取用户
        let user_by_username = user_repo.get_user_by_username("testuser").await.unwrap();
        assert!(user_by_username.is_some());
    }

    #[tokio::test]
    async fn test_database_stats() {
        let database = create_test_database().await.unwrap();
        
        // 执行一些查询来生成统计信息
        let _result = database.execute("SELECT 1", &[]).await.unwrap();
        let _result = database.execute("SELECT 2", &[]).await.unwrap();
        
        // 获取统计信息
        let stats = database.get_stats().await.unwrap();
        assert!(stats.total_queries >= 2);
    }

    #[tokio::test]
    async fn test_migration_system() {
        let database = create_test_database().await.unwrap();
        
        // 检查迁移状态
        let status = MigrationManager::get_migration_status(&database).await.unwrap();
        assert_eq!(status, MigrationStatus::Completed);

        // 获取迁移历史
        let history = MigrationManager::get_migration_history(&database).await.unwrap();
        assert!(!history.is_empty());
    }
} 