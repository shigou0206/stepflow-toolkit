//! Test database setup and initialization

use std::sync::Arc;
use stepflow_core::*;
use stepflow_database::{SqliteDatabase, MigrationManager};
use stepflow_executor::{ExecutionContext, ExecutionOptions, Priority, ResourceLimits};

/// Set up a test database with all necessary tables and migrations
pub async fn setup_test_database() -> StepflowResult<Arc<SqliteDatabase>> {
    // Create in-memory database for testing
    let db = Arc::new(SqliteDatabase::new(":memory:").await?);
    
    // Run migrations to create all necessary tables
    MigrationManager::run_migrations(&db).await?;
    
    // Insert test data
    populate_test_data(&db).await?;
    
    Ok(db)
}

/// Populate test database with sample data
async fn populate_test_data(db: &SqliteDatabase) -> StepflowResult<()> {
    // Insert sample tools
    let tools_sql = r#"
        INSERT INTO tools (
            id, name, description, version_major, version_minor, version_patch,
            tool_type, status, author, tags, capabilities, created_at, updated_at
        ) VALUES 
        ('test-tool-1', 'Test Tool 1', 'A test tool for basic operations', 1, 0, 0, 'python', 'active', 'test@example.com', '["test", "basic"]', '["execute", "validate"]', datetime('now'), datetime('now')),
        ('test-tool-2', 'Test Tool 2', 'A test tool for advanced operations', 2, 1, 0, 'rust', 'active', 'test@example.com', '["test", "advanced"]', '["execute", "transform"]', datetime('now'), datetime('now')),
        ('slow-tool', 'Slow Test Tool', 'A slow test tool for performance testing', 1, 0, 0, 'python', 'active', 'test@example.com', '["test", "performance"]', '["execute"]', datetime('now'), datetime('now'))
    "#;
    
    db.execute(tools_sql, &[]).await?;
    
    // Insert sample tenants
    let tenants_sql = r#"
        INSERT INTO tenants (id, name, description, created_at, updated_at)
        VALUES ('test-tenant', 'Test Tenant', 'A test tenant for testing', datetime('now'), datetime('now'))
    "#;
    
    db.execute(tenants_sql, &[]).await?;
    
    // Insert sample users
    let users_sql = r#"
        INSERT INTO users (id, username, email, password_hash, role, tenant_id, created_at, updated_at)
        VALUES ('test-user', 'testuser', 'test@example.com', 'hashed_password', 'user', 'test-tenant', datetime('now'), datetime('now'))
    "#;
    
    db.execute(users_sql, &[]).await?;
    
    Ok(())
}

/// Reset test database to clean state
pub async fn reset_test_database(db: &SqliteDatabase) -> StepflowResult<()> {
    // Clear all test data
    let tables = vec![
        "execution_results",
        "metrics", 
        "logs",
        "executions",
        "works",
        "tasks",
        "workers",
        "users",
        "tenants",
        "tools"
    ];
    
    for table in tables {
        let sql = format!("DELETE FROM {}", table);
        db.execute(&sql, &[]).await?;
    }
    
    // Re-populate with fresh test data
    populate_test_data(db).await?;
    
    Ok(())
}

/// Create test execution context
pub fn create_test_execution_context() -> ExecutionContext {
    ExecutionContext {
        user_id: "test-user".to_string(),
        tenant_id: "test-tenant".to_string(),
        session_id: "test-session-1".to_string(),
        request_id: "test-request-1".to_string(),
        parent_execution_id: None,
        environment: std::collections::HashMap::new(),
    }
}

/// Create test execution options
pub fn create_test_execution_options() -> ExecutionOptions {
    ExecutionOptions {
        timeout: Some(std::time::Duration::from_secs(30)),
        retry_count: 3,
        retry_delay: std::time::Duration::from_millis(100),
        priority: Priority::Normal,
        resource_limits: ResourceLimits::default(),
        logging_level: LogLevel::Info,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_setup_test_database() {
        let db = setup_test_database().await.unwrap();
        assert!(db.health_check().await.unwrap());
        
        // Verify tools table has data
        let result = db.execute("SELECT COUNT(*) as count FROM tools", &[]).await.unwrap();
        assert!(!result.rows.is_empty());
    }
    
    #[tokio::test]
    async fn test_reset_test_database() {
        let db = setup_test_database().await.unwrap();
        reset_test_database(&db).await.unwrap();
        
        // Verify data is still there after reset
        let result = db.execute("SELECT COUNT(*) as count FROM tools", &[]).await.unwrap();
        assert!(!result.rows.is_empty());
    }
} 