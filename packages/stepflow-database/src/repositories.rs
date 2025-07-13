//! Database repositories

use stepflow_core::{
    ToolId, ToolInfo, ToolStatus, ToolType, ToolStats,
    TenantId, TenantInfo, UserId, UserInfo, UserRole,
    StepflowError, StepflowResult, Database,
};
use serde_json::Value;
use std::collections::HashMap;
use crate::SqliteDatabase;
use chrono::{DateTime, Utc};
use crate::utils::{hash_password, verify_password};
use crate::models::{ToolModel, TenantModel, UserModel};

/// Helper function to convert database row to ToolModel
fn row_to_tool_model(row: &HashMap<String, Value>) -> Option<ToolModel> {
    Some(ToolModel {
        id: row.get("id")?.as_str()?.to_string(),
        name: row.get("name")?.as_str()?.to_string(),
        description: row.get("description").and_then(|v| v.as_str()).map(|s| s.to_string()),
        version_major: row.get("version_major")?.as_i64()? as u32,
        version_minor: row.get("version_minor")?.as_i64()? as u32,
        version_patch: row.get("version_patch")?.as_i64()? as u32,
        version_pre_release: row.get("version_pre_release").and_then(|v| v.as_str()).map(|s| s.to_string()),
        version_build: row.get("version_build").and_then(|v| v.as_str()).map(|s| s.to_string()),
        tool_type: row.get("tool_type")?.as_str()?.to_string(),
        status: row.get("status")?.as_str()?.to_string(),
        author: row.get("author")?.as_str()?.to_string(),
        repository: row.get("repository").and_then(|v| v.as_str()).map(|s| s.to_string()),
        documentation: row.get("documentation").and_then(|v| v.as_str()).map(|s| s.to_string()),
        tags: row.get("tags").and_then(|v| v.as_str()).map(|s| s.to_string()),
        capabilities: row.get("capabilities").and_then(|v| v.as_str()).map(|s| s.to_string()),
        configuration_schema: row.get("configuration_schema").and_then(|v| v.as_str()).map(|s| s.to_string()),
        examples: row.get("examples").and_then(|v| v.as_str()).map(|s| s.to_string()),
        created_at: row.get("created_at")?.as_str()?.parse().ok()?,
        updated_at: row.get("updated_at")?.as_str()?.parse().ok()?,
    })
}

/// Helper function to convert database row to TenantModel
fn row_to_tenant_model(row: &HashMap<String, Value>) -> Option<TenantModel> {
    Some(TenantModel {
        id: row.get("id")?.as_str()?.to_string(),
        name: row.get("name")?.as_str()?.to_string(),
        description: row.get("description").and_then(|v| v.as_str()).map(|s| s.to_string()),
        domain: row.get("domain").and_then(|v| v.as_str()).map(|s| s.to_string()),
        settings: row.get("settings").and_then(|v| v.as_str()).map(|s| s.to_string()),
        created_at: row.get("created_at")?.as_str()?.parse().ok()?,
        updated_at: row.get("updated_at")?.as_str()?.parse().ok()?,
    })
}

/// Helper function to convert database row to UserModel
fn row_to_user_model(row: &HashMap<String, Value>) -> Option<UserModel> {
    Some(UserModel {
        id: row.get("id")?.as_str()?.to_string(),
        username: row.get("username")?.as_str()?.to_string(),
        email: row.get("email")?.as_str()?.to_string(),
        password_hash: row.get("password_hash")?.as_str()?.to_string(),
        role: row.get("role")?.as_str()?.to_string(),
        tenant_id: row.get("tenant_id")?.as_str()?.to_string(),
        settings: row.get("settings").and_then(|v| v.as_str()).map(|s| s.to_string()),
        created_at: row.get("created_at")?.as_str()?.parse().ok()?,
        updated_at: row.get("updated_at")?.as_str()?.parse().ok()?,
    })
}

/// Helper function to convert database row to ExecutionRecord
fn row_to_execution_record(row: &HashMap<String, Value>) -> Option<ExecutionRecord> {
    Some(ExecutionRecord {
        id: row.get("id")?.as_str()?.to_string(),
        tool_id: ToolId::from_string(row.get("tool_id")?.as_str()?.to_string()),
        tenant_id: TenantId::from_string(row.get("tenant_id")?.as_str()?.to_string()),
        user_id: UserId::from_string(row.get("user_id")?.as_str()?.to_string()),
        status: row.get("status")?.as_str()?.to_string(),
        request: row.get("request")?.clone(),
        result: row.get("result").and_then(|v| if v.is_null() { None } else { Some(v.clone()) }),
        started_at: row.get("started_at")?.as_str()?.parse().ok()?,
        completed_at: row.get("completed_at").and_then(|v| v.as_str()).and_then(|s| s.parse().ok()),
        created_at: row.get("created_at")?.as_str()?.parse().ok()?,
        updated_at: row.get("updated_at")?.as_str()?.parse().ok()?,
    })
}

/// Tool repository for managing tools in the database
pub struct ToolRepository {
    database: SqliteDatabase,
}

impl ToolRepository {
    /// Create a new tool repository
    pub fn new(database: SqliteDatabase) -> Self {
        Self { database }
    }

    /// Get database instance (for testing)
    pub fn database(&self) -> &SqliteDatabase {
        &self.database
    }

    /// Create a tool
    pub async fn create_tool(&self, tool: &ToolInfo) -> StepflowResult<()> {
        let sql = r#"
            INSERT INTO tools (
                id, name, description, version_major, version_minor, version_patch,
                version_pre_release, version_build, tool_type, status, author,
                repository, documentation, tags, capabilities, configuration_schema,
                examples, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#;

        let params = vec![
            Value::String(tool.id.as_str().to_string()),
            Value::String(tool.name.clone()),
            Value::String(tool.description.clone()),
            Value::Number(tool.version.major.into()),
            Value::Number(tool.version.minor.into()),
            Value::Number(tool.version.patch.into()),
            Value::String(tool.version.pre_release.clone().unwrap_or_default()),
            Value::String(tool.version.build.clone().unwrap_or_default()),
            Value::String(tool.tool_type.to_string()),
            Value::String(tool.status.to_string()),
            Value::String(tool.author.clone()),
            Value::String(tool.repository.clone().unwrap_or_default()),
            Value::String(tool.documentation.clone().unwrap_or_default()),
            Value::String(serde_json::to_string(&tool.tags)?),
            Value::String(serde_json::to_string(&tool.capabilities)?),
            Value::String(serde_json::to_string(&tool.configuration_schema)?),
            Value::String(serde_json::to_string(&tool.examples)?),
            Value::String(tool.created_at.to_rfc3339()),
            Value::String(tool.updated_at.to_rfc3339()),
        ];

        self.database.execute(sql, &params).await?;
        Ok(())
    }

    /// Get a tool by ID
    pub async fn get_tool(&self, tool_id: &ToolId) -> StepflowResult<Option<ToolInfo>> {
        let sql = "SELECT * FROM tools WHERE id = ?";
        let params = vec![Value::String(tool_id.as_str().to_string())];

        let result = self.database.execute(sql, &params).await?;
        
        if result.rows.is_empty() {
            return Ok(None);
        }

        if let Some(tool_model) = row_to_tool_model(&result.rows[0]) {
            Ok(Some(tool_model.into()))
        } else {
            Ok(None)
        }
    }

    /// Update a tool
    pub async fn update_tool(&self, tool_id: &ToolId, tool: &ToolInfo) -> StepflowResult<()> {
        let sql = r#"
            UPDATE tools SET
                name = ?, description = ?, version_major = ?, version_minor = ?,
                version_patch = ?, version_pre_release = ?, version_build = ?,
                tool_type = ?, status = ?, author = ?, repository = ?,
                documentation = ?, tags = ?, capabilities = ?, configuration_schema = ?,
                examples = ?, updated_at = ?
            WHERE id = ?
        "#;

        let params = vec![
            Value::String(tool.name.clone()),
            Value::String(tool.description.clone()),
            Value::Number(tool.version.major.into()),
            Value::Number(tool.version.minor.into()),
            Value::Number(tool.version.patch.into()),
            Value::String(tool.version.pre_release.clone().unwrap_or_default()),
            Value::String(tool.version.build.clone().unwrap_or_default()),
            Value::String(tool.tool_type.to_string()),
            Value::String(tool.status.to_string()),
            Value::String(tool.author.clone()),
            Value::String(tool.repository.clone().unwrap_or_default()),
            Value::String(tool.documentation.clone().unwrap_or_default()),
            Value::String(serde_json::to_string(&tool.tags)?),
            Value::String(serde_json::to_string(&tool.capabilities)?),
            Value::String(serde_json::to_string(&tool.configuration_schema)?),
            Value::String(serde_json::to_string(&tool.examples)?),
            Value::String(tool.updated_at.to_rfc3339()),
            Value::String(tool_id.as_str().to_string()),
        ];

        let _result = self.database.execute(sql, &params).await?;
        
        if _result.rows_affected == 0 {
            return Err(StepflowError::DatabaseError(stepflow_core::DatabaseError::QueryFailed(
                "Tool not found".to_string()
            )));
        }

        Ok(())
    }

    /// Delete a tool
    pub async fn delete_tool(&self, tool_id: &ToolId) -> StepflowResult<()> {
        let sql = "DELETE FROM tools WHERE id = ?";
        let params = vec![Value::String(tool_id.as_str().to_string())];

        let _result = self.database.execute(sql, &params).await?;
        
        if _result.rows_affected == 0 {
            return Err(StepflowError::DatabaseError(stepflow_core::DatabaseError::QueryFailed(
                "Tool not found".to_string()
            )));
        }

        Ok(())
    }

    /// List tools with optional filtering
    pub async fn list_tools(&self, filter: Option<HashMap<String, Value>>) -> StepflowResult<Vec<ToolInfo>> {
        let mut sql = "SELECT * FROM tools".to_string();
        let mut params = Vec::new();

        if let Some(filter_map) = filter {
            if !filter_map.is_empty() {
                sql.push_str(" WHERE ");
                let conditions: Vec<String> = filter_map.iter().map(|(k, v)| {
                    params.push(v.clone());
                    format!("{} = ?", k)
                }).collect();
                sql.push_str(&conditions.join(" AND "));
            }
        }

        let result = self.database.execute(&sql, &params).await?;
        
        let mut tools = Vec::new();
        for row in result.rows {
            if let Some(tool_model) = row_to_tool_model(&row) {
                tools.push(tool_model.into());
            }
        }
        
        Ok(tools)
    }

    /// Search tools by query
    pub async fn search_tools(&self, query: &str) -> StepflowResult<Vec<ToolInfo>> {
        let sql = "SELECT * FROM tools WHERE name LIKE ? OR description LIKE ?";
        let search_pattern = format!("%{}%", query);
        let params = vec![
            Value::String(search_pattern.clone()),
            Value::String(search_pattern),
        ];

        let result = self.database.execute(sql, &params).await?;
        
        let mut tools = Vec::new();
        for row in result.rows {
            if let Some(tool_model) = row_to_tool_model(&row) {
                tools.push(tool_model.into());
            }
        }
        
        Ok(tools)
    }

    /// Get tools by status
    pub async fn get_tools_by_status(&self, status: &ToolStatus) -> StepflowResult<Vec<ToolInfo>> {
        let sql = "SELECT * FROM tools WHERE status = ?";
        let params = vec![Value::String(status.to_string())];

        let result = self.database.execute(sql, &params).await?;
        
        let mut tools = Vec::new();
        for row in result.rows {
            if let Some(tool_model) = row_to_tool_model(&row) {
                tools.push(tool_model.into());
            }
        }
        
        Ok(tools)
    }

    /// Get tools by type
    pub async fn get_tools_by_type(&self, tool_type: &ToolType) -> StepflowResult<Vec<ToolInfo>> {
        let sql = "SELECT * FROM tools WHERE tool_type = ?";
        let params = vec![Value::String(tool_type.to_string())];

        let result = self.database.execute(sql, &params).await?;
        
        let mut tools = Vec::new();
        for row in result.rows {
            if let Some(tool_model) = row_to_tool_model(&row) {
                tools.push(tool_model.into());
            }
        }
        
        Ok(tools)
    }

    /// Check if a tool exists
    pub async fn tool_exists(&self, tool_id: &ToolId) -> StepflowResult<bool> {
        let sql = "SELECT 1 FROM tools WHERE id = ? LIMIT 1";
        let params = vec![Value::String(tool_id.as_str().to_string())];

        let result = self.database.execute(sql, &params).await?;
        Ok(!result.rows.is_empty())
    }

    /// Get tool statistics
    pub async fn get_tool_stats(&self) -> StepflowResult<ToolStats> {
        let sql = "SELECT COUNT(*) as total, COUNT(CASE WHEN status = 'active' THEN 1 END) as active FROM tools";
        let result = self.database.execute(sql, &[]).await?;
        
        if result.rows.is_empty() {
            return Ok(ToolStats {
                total_tools: 0,
                active_tools: 0,
                deprecated_tools: 0,
                error_tools: 0,
            });
        }

        let row = &result.rows[0];
        Ok(ToolStats {
            total_tools: row.get("total").and_then(|v| v.as_i64()).unwrap_or(0) as u64,
            active_tools: row.get("active").and_then(|v| v.as_i64()).unwrap_or(0) as u64,
            deprecated_tools: 0, // TODO: Add proper query
            error_tools: 0, // TODO: Add proper query
        })
    }
}

/// Tenant repository for managing tenants in the database
pub struct TenantRepository {
    database: SqliteDatabase,
}

impl TenantRepository {
    /// Create a new tenant repository
    pub fn new(database: SqliteDatabase) -> Self {
        Self { database }
    }

    /// Create a tenant
    pub async fn create_tenant(&self, tenant: &TenantInfo) -> StepflowResult<()> {
        let sql = r#"
            INSERT INTO tenants (
                id, name, description, domain, settings, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
        "#;

        let params = vec![
            Value::String(tenant.id.as_str().to_string()),
            Value::String(tenant.name.clone()),
            Value::String(tenant.description.clone()),
            Value::String(tenant.domain.clone().unwrap_or_default()),
            Value::String(serde_json::to_string(&tenant.settings)?),
            Value::String(tenant.created_at.to_rfc3339()),
            Value::String(tenant.updated_at.to_rfc3339()),
        ];

        self.database.execute(sql, &params).await?;
        Ok(())
    }

    /// Get a tenant by ID
    pub async fn get_tenant(&self, tenant_id: &TenantId) -> StepflowResult<Option<TenantInfo>> {
        let sql = "SELECT * FROM tenants WHERE id = ?";
        let params = vec![Value::String(tenant_id.as_str().to_string())];

        let result = self.database.execute(sql, &params).await?;
        
        if result.rows.is_empty() {
            return Ok(None);
        }

        if let Some(tenant_model) = row_to_tenant_model(&result.rows[0]) {
            Ok(Some(tenant_model.into()))
        } else {
            Ok(None)
        }
    }

    /// Update a tenant
    pub async fn update_tenant(&self, tenant_id: &TenantId, tenant: &TenantInfo) -> StepflowResult<()> {
        let sql = r#"
            UPDATE tenants SET
                name = ?, description = ?, domain = ?, settings = ?, updated_at = ?
            WHERE id = ?
        "#;

        let params = vec![
            Value::String(tenant.name.clone()),
            Value::String(tenant.description.clone()),
            Value::String(tenant.domain.clone().unwrap_or_default()),
            Value::String(serde_json::to_string(&tenant.settings)?),
            Value::String(tenant.updated_at.to_rfc3339()),
            Value::String(tenant_id.as_str().to_string()),
        ];

        let _result = self.database.execute(sql, &params).await?;
        
        if _result.rows_affected == 0 {
            return Err(StepflowError::DatabaseError(stepflow_core::DatabaseError::QueryFailed(
                "Tenant not found".to_string()
            )));
        }

        Ok(())
    }

    /// Delete a tenant
    pub async fn delete_tenant(&self, tenant_id: &TenantId) -> StepflowResult<()> {
        let sql = "DELETE FROM tenants WHERE id = ?";
        let params = vec![Value::String(tenant_id.as_str().to_string())];

        let _result = self.database.execute(sql, &params).await?;
        
        if _result.rows_affected == 0 {
            return Err(StepflowError::DatabaseError(stepflow_core::DatabaseError::QueryFailed(
                "Tenant not found".to_string()
            )));
        }

        Ok(())
    }

    /// List tenants with optional filtering
    pub async fn list_tenants(&self, filter: Option<HashMap<String, Value>>) -> StepflowResult<Vec<TenantInfo>> {
        let mut sql = "SELECT * FROM tenants".to_string();
        let mut params = Vec::new();

        if let Some(filter_map) = filter {
            if !filter_map.is_empty() {
                sql.push_str(" WHERE ");
                let conditions: Vec<String> = filter_map.iter().map(|(k, v)| {
                    params.push(v.clone());
                    format!("{} = ?", k)
                }).collect();
                sql.push_str(&conditions.join(" AND "));
            }
        }

        let result = self.database.execute(&sql, &params).await?;
        
        let mut tenants = Vec::new();
        for row in result.rows {
            if let Some(tenant_model) = row_to_tenant_model(&row) {
                tenants.push(tenant_model.into());
            }
        }
        
        Ok(tenants)
    }
}

/// User repository for managing users in the database
pub struct UserRepository {
    database: SqliteDatabase,
}

impl UserRepository {
    /// Create a new user repository
    pub fn new(database: SqliteDatabase) -> Self {
        Self { database }
    }

    /// Create a user
    pub async fn create_user(&self, user: &UserInfo, password_hash: &str) -> StepflowResult<()> {
        let sql = r#"
            INSERT INTO users (
                id, username, email, password_hash, role, tenant_id, settings,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#;

        let params = vec![
            Value::String(user.id.as_str().to_string()),
            Value::String(user.username.clone()),
            Value::String(user.email.clone()),
            Value::String(password_hash.to_string()),
            Value::String(user.role.to_string()),
            Value::String(user.tenant_id.as_str().to_string()),
            Value::String(serde_json::to_string(&user.settings)?),
            Value::String(user.created_at.to_rfc3339()),
            Value::String(user.updated_at.to_rfc3339()),
        ];

        self.database.execute(sql, &params).await?;
        Ok(())
    }

    /// Get a user by ID
    pub async fn get_user(&self, user_id: &UserId) -> StepflowResult<Option<UserInfo>> {
        let sql = "SELECT * FROM users WHERE id = ?";
        let params = vec![Value::String(user_id.as_str().to_string())];

        let result = self.database.execute(sql, &params).await?;
        
        if result.rows.is_empty() {
            return Ok(None);
        }

        if let Some(user_model) = row_to_user_model(&result.rows[0]) {
            Ok(Some(user_model.into()))
        } else {
            Ok(None)
        }
    }

    /// Get a user by username
    pub async fn get_user_by_username(&self, username: &str) -> StepflowResult<Option<UserInfo>> {
        let sql = "SELECT * FROM users WHERE username = ?";
        let params = vec![Value::String(username.to_string())];

        let result = self.database.execute(sql, &params).await?;
        
        if result.rows.is_empty() {
            return Ok(None);
        }

        if let Some(user_model) = row_to_user_model(&result.rows[0]) {
            Ok(Some(user_model.into()))
        } else {
            Ok(None)
        }
    }

    /// Get a user by email
    pub async fn get_user_by_email(&self, email: &str) -> StepflowResult<Option<UserInfo>> {
        let sql = "SELECT * FROM users WHERE email = ?";
        let params = vec![Value::String(email.to_string())];

        let result = self.database.execute(sql, &params).await?;
        
        if result.rows.is_empty() {
            return Ok(None);
        }

        if let Some(user_model) = row_to_user_model(&result.rows[0]) {
            Ok(Some(user_model.into()))
        } else {
            Ok(None)
        }
    }

    /// Update a user (optionally update password_hash)
    pub async fn update_user(&self, user_id: &UserId, user: &UserInfo, password_hash: Option<&str>) -> StepflowResult<()> {
        let (sql, params) = if let Some(hash) = password_hash {
            (
                r#"
                UPDATE users SET
                    username = ?, email = ?, password_hash = ?, role = ?,
                    tenant_id = ?, settings = ?, updated_at = ?
                WHERE id = ?
                "#,
                vec![
                    Value::String(user.username.clone()),
                    Value::String(user.email.clone()),
                    Value::String(hash.to_string()),
                    Value::String(user.role.to_string()),
                    Value::String(user.tenant_id.as_str().to_string()),
                    Value::String(serde_json::to_string(&user.settings)?),
                    Value::String(user.updated_at.to_rfc3339()),
                    Value::String(user_id.as_str().to_string()),
                ]
            )
        } else {
            (
                r#"
                UPDATE users SET
                    username = ?, email = ?, role = ?,
                    tenant_id = ?, settings = ?, updated_at = ?
                WHERE id = ?
                "#,
                vec![
                    Value::String(user.username.clone()),
                    Value::String(user.email.clone()),
                    Value::String(user.role.to_string()),
                    Value::String(user.tenant_id.as_str().to_string()),
                    Value::String(serde_json::to_string(&user.settings)?),
                    Value::String(user.updated_at.to_rfc3339()),
                    Value::String(user_id.as_str().to_string()),
                ]
            )
        };

        let _result = self.database.execute(sql, &params).await?;
        
        if _result.rows_affected == 0 {
            return Err(StepflowError::DatabaseError(stepflow_core::DatabaseError::QueryFailed(
                "User not found".to_string()
            )));
        }

        Ok(())
    }

    /// Delete a user
    pub async fn delete_user(&self, user_id: &UserId) -> StepflowResult<()> {
        let sql = "DELETE FROM users WHERE id = ?";
        let params = vec![Value::String(user_id.as_str().to_string())];

        let _result = self.database.execute(sql, &params).await?;
        
        if _result.rows_affected == 0 {
            return Err(StepflowError::DatabaseError(stepflow_core::DatabaseError::QueryFailed(
                "User not found".to_string()
            )));
        }

        Ok(())
    }

    /// List users by tenant
    pub async fn list_users_by_tenant(&self, tenant_id: &TenantId) -> StepflowResult<Vec<UserInfo>> {
        let sql = "SELECT * FROM users WHERE tenant_id = ?";
        let params = vec![Value::String(tenant_id.as_str().to_string())];

        let result = self.database.execute(sql, &params).await?;
        
        let mut users = Vec::new();
        for row in result.rows {
            if let Some(user_model) = row_to_user_model(&row) {
                users.push(user_model.into());
            }
        }
        
        Ok(users)
    }

    /// Get users by role
    pub async fn get_users_by_role(&self, role: &UserRole) -> StepflowResult<Vec<UserInfo>> {
        let sql = "SELECT * FROM users WHERE role = ?";
        let params = vec![Value::String(role.to_string())];

        let result = self.database.execute(sql, &params).await?;
        
        let mut users = Vec::new();
        for row in result.rows {
            if let Some(user_model) = row_to_user_model(&row) {
                users.push(user_model.into());
            }
        }
        
        Ok(users)
    }

    /// Register a new user (hashes password and stores it)
    pub async fn register_user(&self, user: &UserInfo, password: &str) -> StepflowResult<()> {
        let hash = hash_password(password).map_err(|e| StepflowError::DatabaseError(
            stepflow_core::DatabaseError::QueryFailed(e.to_string())
        ))?;
        self.create_user(user, &hash).await
    }

    /// Verify a user's password
    pub async fn verify_user_password(&self, user_id: &UserId, password: &str) -> StepflowResult<bool> {
        let sql = "SELECT password_hash FROM users WHERE id = ?";
        let params = vec![Value::String(user_id.as_str().to_string())];
        let result = self.database.execute(sql, &params).await?;
        
        // 安全地检查结果
        if result.rows.is_empty() {
            return Ok(false);
        }
        
        // 安全地获取password_hash
        let password_hash = result.rows[0]
            .get("password_hash")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        if password_hash.is_empty() {
            return Ok(false);
        }
        
        // 验证密码
        Ok(verify_password(password, password_hash).unwrap_or(false))
    }

    /// Change a user's password
    pub async fn change_user_password(&self, user_id: &UserId, new_password: &str) -> StepflowResult<()> {
        let hash = hash_password(new_password).map_err(|e| StepflowError::DatabaseError(
            stepflow_core::DatabaseError::QueryFailed(e.to_string())
        ))?;
        let sql = "UPDATE users SET password_hash = ?, updated_at = ? WHERE id = ?";
        let params = vec![
            Value::String(hash),
            Value::String(chrono::Utc::now().to_rfc3339()),
            Value::String(user_id.as_str().to_string()),
        ];
        let result = self.database.execute(sql, &params).await?;
        if result.rows_affected == 0 {
            return Err(StepflowError::DatabaseError(stepflow_core::DatabaseError::QueryFailed(
                "User not found".to_string()
            )));
        }
        Ok(())
    }
}

/// Execution repository for managing tool executions in the database
pub struct ExecutionRepository {
    database: SqliteDatabase,
}

impl ExecutionRepository {
    /// Create a new execution repository
    pub fn new(database: SqliteDatabase) -> Self {
        Self { database }
    }

    /// Create an execution record
    pub async fn create_execution(&self, execution: &ExecutionRecord) -> StepflowResult<()> {
        let sql = r#"
            INSERT INTO executions (
                id, tool_id, tenant_id, user_id, status, request, result,
                started_at, completed_at, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#;

        let params = vec![
            Value::String(execution.id.clone()),
            Value::String(execution.tool_id.as_str().to_string()),
            Value::String(execution.tenant_id.as_str().to_string()),
            Value::String(execution.user_id.as_str().to_string()),
            Value::String(execution.status.clone()),
            Value::String(serde_json::to_string(&execution.request)?),
            Value::String(serde_json::to_string(&execution.result)?),
            Value::String(execution.started_at.to_rfc3339()),
            Value::String(execution.completed_at.map(|t| t.to_rfc3339()).unwrap_or_default()),
            Value::String(execution.created_at.to_rfc3339()),
            Value::String(execution.updated_at.to_rfc3339()),
        ];

        self.database.execute(sql, &params).await?;
        Ok(())
    }

    /// Get an execution by ID
    pub async fn get_execution(&self, execution_id: &str) -> StepflowResult<Option<ExecutionRecord>> {
        let sql = "SELECT * FROM executions WHERE id = ?";
        let params = vec![Value::String(execution_id.to_string())];

        let result = self.database.execute(sql, &params).await?;
        
        if result.rows.is_empty() {
            return Ok(None);
        }

        if let Some(execution_record) = row_to_execution_record(&result.rows[0]) {
            Ok(Some(execution_record))
        } else {
            Ok(None)
        }
    }

    /// Update execution status
    pub async fn update_execution_status(&self, execution_id: &str, status: &str, result: Option<Value>) -> StepflowResult<()> {
        let sql = r#"
            UPDATE executions SET
                status = ?, result = ?, completed_at = ?, updated_at = ?
            WHERE id = ?
        "#;

        let params = vec![
            Value::String(status.to_string()),
            Value::String(serde_json::to_string(&result)?),
            Value::String(chrono::Utc::now().to_rfc3339()),
            Value::String(chrono::Utc::now().to_rfc3339()),
            Value::String(execution_id.to_string()),
        ];

        let _result = self.database.execute(sql, &params).await?;
        
        if _result.rows_affected == 0 {
            return Err(StepflowError::DatabaseError(stepflow_core::DatabaseError::QueryFailed(
                "Execution not found".to_string()
            )));
        }

        Ok(())
    }

    /// List executions by tool
    pub async fn list_executions_by_tool(&self, tool_id: &ToolId) -> StepflowResult<Vec<ExecutionRecord>> {
        let sql = "SELECT * FROM executions WHERE tool_id = ? ORDER BY started_at DESC";
        let params = vec![Value::String(tool_id.as_str().to_string())];

        let result = self.database.execute(sql, &params).await?;
        
        let mut executions = Vec::new();
        for row in result.rows {
            if let Some(execution_record) = row_to_execution_record(&row) {
                executions.push(execution_record);
            }
        }
        
        Ok(executions)
    }

    /// List executions by tenant
    pub async fn list_executions_by_tenant(&self, tenant_id: &TenantId) -> StepflowResult<Vec<ExecutionRecord>> {
        let sql = "SELECT * FROM executions WHERE tenant_id = ? ORDER BY started_at DESC";
        let params = vec![Value::String(tenant_id.as_str().to_string())];

        let result = self.database.execute(sql, &params).await?;
        
        let mut executions = Vec::new();
        for row in result.rows {
            if let Some(execution_record) = row_to_execution_record(&row) {
                executions.push(execution_record);
            }
        }
        
        Ok(executions)
    }

    /// List executions by user
    pub async fn list_executions_by_user(&self, user_id: &UserId) -> StepflowResult<Vec<ExecutionRecord>> {
        let sql = "SELECT * FROM executions WHERE user_id = ? ORDER BY started_at DESC";
        let params = vec![Value::String(user_id.as_str().to_string())];

        let result = self.database.execute(sql, &params).await?;
        
        let mut executions = Vec::new();
        for row in result.rows {
            if let Some(execution_record) = row_to_execution_record(&row) {
                executions.push(execution_record);
            }
        }
        
        Ok(executions)
    }

    /// Get execution statistics
    pub async fn get_execution_stats(&self, tenant_id: Option<&TenantId>) -> StepflowResult<ExecutionStats> {
        let mut sql = "SELECT COUNT(*) as total, COUNT(CASE WHEN status = 'completed' THEN 1 END) as completed, COUNT(CASE WHEN status = 'failed' THEN 1 END) as failed, COUNT(CASE WHEN status = 'running' THEN 1 END) as in_progress FROM executions".to_string();
        let mut params = Vec::new();

        if let Some(tenant) = tenant_id {
            sql.push_str(" WHERE tenant_id = ?");
            params.push(Value::String(tenant.as_str().to_string()));
        }

        let result = self.database.execute(&sql, &params).await?;
        
        if result.rows.is_empty() {
            return Ok(ExecutionStats {
                total: 0,
                completed: 0,
                failed: 0,
                in_progress: 0,
            });
        }

        let row = &result.rows[0];
        Ok(ExecutionStats {
            total: row.get("total").and_then(|v| v.as_i64()).unwrap_or(0) as u64,
            completed: row.get("completed").and_then(|v| v.as_i64()).unwrap_or(0) as u64,
            failed: row.get("failed").and_then(|v| v.as_i64()).unwrap_or(0) as u64,
            in_progress: row.get("in_progress").and_then(|v| v.as_i64()).unwrap_or(0) as u64,
        })
    }
}

/// Execution record
#[derive(Debug, Clone)]
pub struct ExecutionRecord {
    pub id: String,
    pub tool_id: ToolId,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub status: String,
    pub request: Value,
    pub result: Option<Value>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Execution statistics
#[derive(Debug, Clone)]
pub struct ExecutionStats {
    pub total: u64,
    pub completed: u64,
    pub failed: u64,
    pub in_progress: u64,
} 