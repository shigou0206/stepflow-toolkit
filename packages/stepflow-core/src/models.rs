use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc, Duration};
use validator::{Validate};
use crate::types::*;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ToolSpec {
    pub id: ToolId,
    #[validate(length(min = 1, max = 200))]
    pub name: String,
    #[validate(length(min = 1, max = 1000))]
    pub description: String,
    pub tool_type: ToolType,
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
    pub config: serde_json::Value,
    #[validate(length(max = 20))]
    pub tags: Vec<String>,
    pub category: Option<String>,
    pub version: String,
    pub tenant_id: TenantId,
    pub registered_at: DateTime<Utc>,
    pub execution_config: ExecutionConfig,
}

impl ToolSpec {
    pub fn validate(&self) -> Result<(), validator::ValidationError> {
        // 手动校验 ToolId
        // 手动校验 Duration
        // TODO: schema/config 校验
        Ok(())
    }
    pub fn generate_id(namespace: &str, name: &str, version: &str) -> ToolId {
        ToolId::from_string(format!("tool:{}:{}/{}", namespace, name, version))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ExecutionConfig {
    pub timeout: Duration,
    pub memory_limit: Option<u64>,
    pub cpu_limit: Option<f64>,
    pub sandbox_level: SandboxLevel,
    pub retry_config: RetryConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SandboxLevel {
    None,
    Basic,
    Strict,
    Process,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RetryConfig {
    #[validate(range(min = 0, max = 10))]
    pub max_retries: u32,
    pub retry_delay: Duration,
    pub backoff_strategy: BackoffStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackoffStrategy {
    Fixed,
    Exponential,
    Linear,
} 