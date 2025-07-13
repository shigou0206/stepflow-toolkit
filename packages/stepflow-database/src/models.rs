//! Database models

use serde::{Deserialize, Serialize};
use stepflow_core::*;
use chrono::{DateTime, Utc};

/// Database model for tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolModel {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub version_major: u32,
    pub version_minor: u32,
    pub version_patch: u32,
    pub version_pre_release: Option<String>,
    pub version_build: Option<String>,
    pub tool_type: String,
    pub status: String,
    pub author: String,
    pub repository: Option<String>,
    pub documentation: Option<String>,
    pub tags: Option<String>, // JSON array
    pub capabilities: Option<String>, // JSON array
    pub configuration_schema: Option<String>, // JSON object
    pub examples: Option<String>, // JSON array
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ToolModel> for ToolInfo {
    fn from(model: ToolModel) -> Self {
        let version = ToolVersion {
            major: model.version_major,
            minor: model.version_minor,
            patch: model.version_patch,
            pre_release: model.version_pre_release,
            build: model.version_build,
        };

        let tool_type = match model.tool_type.as_str() {
            "openapi" => ToolType::OpenAPI,
            "asyncapi" => ToolType::AsyncAPI,
            "python" => ToolType::Python,
            "shell" => ToolType::Shell,
            "ai" => ToolType::AI,
            "system" => ToolType::System,
            custom if custom.starts_with("custom:") => {
                ToolType::Custom(custom[7..].to_string())
            }
            _ => ToolType::Custom(model.tool_type),
        };

        let status = match model.status.as_str() {
            "active" => ToolStatus::Active,
            "inactive" => ToolStatus::Inactive,
            "deprecated" => ToolStatus::Deprecated,
            "error" => ToolStatus::Error,
            _ => ToolStatus::Active,
        };

        let tags = model.tags
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default();

        let capabilities = model.capabilities
            .and_then(|c| serde_json::from_str(&c).ok())
            .unwrap_or_default();

        let configuration_schema = model.configuration_schema
            .and_then(|s| serde_json::from_str(&s).ok());

        let examples = model.examples
            .and_then(|e| serde_json::from_str(&e).ok())
            .unwrap_or_default();

        Self {
            id: ToolId::from_string(model.id),
            name: model.name,
            description: model.description.unwrap_or_default(),
            version,
            tool_type,
            status,
            author: model.author,
            repository: model.repository,
            documentation: model.documentation,
            tags,
            capabilities,
            configuration_schema,
            examples,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

impl From<ToolInfo> for ToolModel {
    fn from(info: ToolInfo) -> Self {
        let tool_type = match info.tool_type {
            ToolType::OpenAPI => "openapi".to_string(),
            ToolType::AsyncAPI => "asyncapi".to_string(),
            ToolType::Python => "python".to_string(),
            ToolType::Shell => "shell".to_string(),
            ToolType::AI => "ai".to_string(),
            ToolType::System => "system".to_string(),
            ToolType::Custom(s) => format!("custom:{}", s),
        };

        let status = match info.status {
            ToolStatus::Active => "active".to_string(),
            ToolStatus::Inactive => "inactive".to_string(),
            ToolStatus::Deprecated => "deprecated".to_string(),
            ToolStatus::Error => "error".to_string(),
        };

        Self {
            id: info.id.as_str().to_string(),
            name: info.name,
            description: Some(info.description),
            version_major: info.version.major,
            version_minor: info.version.minor,
            version_patch: info.version.patch,
            version_pre_release: info.version.pre_release,
            version_build: info.version.build,
            tool_type,
            status,
            author: info.author,
            repository: info.repository,
            documentation: info.documentation,
            tags: serde_json::to_string(&info.tags).ok(),
            capabilities: serde_json::to_string(&info.capabilities).ok(),
            configuration_schema: info.configuration_schema
                .map(|s| serde_json::to_string(&s))
                .transpose()
                .ok()
                .flatten(),
            examples: serde_json::to_string(&info.examples).ok(),
            created_at: info.created_at,
            updated_at: info.updated_at,
        }
    }
}

/// Database model for executions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionModel {
    pub id: String,
    pub tool_id: String,
    pub tenant_id: String,
    pub user_id: String,
    pub status: String,
    pub request_input: Option<String>, // JSON object
    pub request_configuration: Option<String>, // JSON object
    pub request_metadata: Option<String>, // JSON object
    pub result_success: Option<bool>,
    pub result_output: Option<String>, // JSON object
    pub result_error: Option<String>,
    pub result_logs: Option<String>, // JSON array
    pub result_metrics: Option<String>, // JSON object
    pub result_metadata: Option<String>, // JSON object
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ExecutionModel> for Execution {
    fn from(model: ExecutionModel) -> Self {
        let status = match model.status.as_str() {
            "pending" => ExecutionStatus::Pending,
            "running" => ExecutionStatus::Running,
            "completed" => ExecutionStatus::Completed,
            "failed" => ExecutionStatus::Failed,
            "cancelled" => ExecutionStatus::Cancelled,
            "timeout" => ExecutionStatus::Timeout,
            _ => ExecutionStatus::Pending,
        };

        let request = ToolRequest {
            tool_id: ToolId::from_string(model.tool_id.clone()),
            input: model.request_input
                .and_then(|i| serde_json::from_str(&i).ok())
                .unwrap_or_default(),
            configuration: model.request_configuration
                .and_then(|c| serde_json::from_str(&c).ok()),
            metadata: model.request_metadata
                .and_then(|m| serde_json::from_str(&m).ok())
                .unwrap_or_default(),
        };

        let result = if model.result_success.is_some() {
            Some(ExecutionResult {
                success: model.result_success.unwrap_or(false),
                output: model.result_output
                    .and_then(|o| serde_json::from_str(&o).ok()),
                error: model.result_error,
                logs: model.result_logs
                    .and_then(|l| serde_json::from_str(&l).ok())
                    .unwrap_or_default(),
                metrics: model.result_metrics
                    .and_then(|m| serde_json::from_str(&m).ok())
                    .unwrap_or_default(),
                metadata: model.result_metadata
                    .and_then(|m| serde_json::from_str(&m).ok())
                    .unwrap_or_default(),
            })
        } else {
            None
        };

        Self {
            id: ExecutionId::from_string(model.id),
            tool_id: ToolId::from_string(model.tool_id),
            tenant_id: TenantId::from_string(model.tenant_id),
            user_id: UserId::from_string(model.user_id),
            status,
            request,
            result,
            started_at: model.started_at.unwrap_or(model.created_at),
            completed_at: model.completed_at,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

impl From<Execution> for ExecutionModel {
    fn from(execution: Execution) -> Self {
        let status = match execution.status {
            ExecutionStatus::Pending => "pending".to_string(),
            ExecutionStatus::Running => "running".to_string(),
            ExecutionStatus::Completed => "completed".to_string(),
            ExecutionStatus::Failed => "failed".to_string(),
            ExecutionStatus::Cancelled => "cancelled".to_string(),
            ExecutionStatus::Timeout => "timeout".to_string(),
        };

        Self {
            id: execution.id.as_str().to_string(),
            tool_id: execution.tool_id.as_str().to_string(),
            tenant_id: execution.tenant_id.as_str().to_string(),
            user_id: execution.user_id.as_str().to_string(),
            status,
            request_input: serde_json::to_string(&execution.request.input).ok(),
            request_configuration: execution.request.configuration
                .as_ref()
                .map(|c| serde_json::to_string(c))
                .transpose()
                .ok()
                .flatten(),
            request_metadata: serde_json::to_string(&execution.request.metadata).ok(),
            result_success: execution.result.as_ref().map(|r| r.success),
            result_output: execution.result
                .as_ref()
                .and_then(|r| r.output.as_ref())
                .map(|o| serde_json::to_string(o))
                .transpose()
                .ok()
                .flatten(),
            result_error: execution.result.as_ref().and_then(|r| r.error.clone()),
            result_logs: execution.result
                .as_ref()
                .map(|r| serde_json::to_string(&r.logs))
                .transpose()
                .ok()
                .flatten(),
            result_metrics: execution.result
                .as_ref()
                .map(|r| serde_json::to_string(&r.metrics))
                .transpose()
                .ok()
                .flatten(),
            result_metadata: execution.result
                .as_ref()
                .map(|r| serde_json::to_string(&r.metadata))
                .transpose()
                .ok()
                .flatten(),
            started_at: Some(execution.started_at),
            completed_at: execution.completed_at,
            created_at: execution.created_at,
            updated_at: execution.updated_at,
        }
    }
}

/// Database model for tenants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantModel {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub domain: Option<String>,
    pub settings: Option<String>, // JSON object
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<TenantModel> for TenantInfo {
    fn from(model: TenantModel) -> Self {
        Self {
            id: TenantId::from_string(model.id),
            name: model.name,
            description: model.description.unwrap_or_default(),
            domain: model.domain,
            settings: model.settings
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default(),
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

impl From<TenantInfo> for TenantModel {
    fn from(info: TenantInfo) -> Self {
        Self {
            id: info.id.as_str().to_string(),
            name: info.name,
            description: Some(info.description),
            domain: info.domain,
            settings: serde_json::to_string(&info.settings).ok(),
            created_at: info.created_at,
            updated_at: info.updated_at,
        }
    }
}

/// Database model for users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserModel {
    pub id: String,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub role: String,
    pub tenant_id: String,
    pub settings: Option<String>, // JSON object
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<UserModel> for UserInfo {
    fn from(model: UserModel) -> Self {
        let role = match model.role.as_str() {
            "admin" => UserRole::Admin,
            "user" => UserRole::User,
            "guest" => UserRole::Guest,
            custom if custom.starts_with("custom:") => {
                UserRole::Custom(custom[7..].to_string())
            }
            _ => UserRole::User,
        };

        Self {
            id: UserId::from_string(model.id),
            username: model.username,
            email: model.email,
            role,
            tenant_id: TenantId::from_string(model.tenant_id),
            settings: model.settings
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default(),
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

impl From<UserInfo> for UserModel {
    fn from(info: UserInfo) -> Self {
        let role = match info.role {
            UserRole::Admin => "admin".to_string(),
            UserRole::User => "user".to_string(),
            UserRole::Guest => "guest".to_string(),
            UserRole::Custom(s) => format!("custom:{}", s),
        };

        Self {
            id: info.id.as_str().to_string(),
            username: info.username,
            email: info.email,
            password_hash: "".to_string(), // Password hash should be set separately
            role,
            tenant_id: info.tenant_id.as_str().to_string(),
            settings: serde_json::to_string(&info.settings).ok(),
            created_at: info.created_at,
            updated_at: info.updated_at,
        }
    }
} 