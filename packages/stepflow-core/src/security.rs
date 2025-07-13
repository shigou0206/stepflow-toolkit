//! Security types and traits for Stepflow Tool System

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Security context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    pub user_id: crate::UserId,
    pub tenant_id: crate::TenantId,
    pub permissions: Vec<String>,
    pub roles: Vec<String>,
    pub session_id: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

/// Security validator trait
#[async_trait]
pub trait SecurityValidator: Send + Sync {
    /// Validate input
    async fn validate_input(&self, input: &str) -> Result<bool, crate::StepflowError>;

    /// Validate path
    async fn validate_path(&self, path: &str) -> Result<bool, crate::StepflowError>;

    /// Validate command
    async fn validate_command(&self, command: &str) -> Result<bool, crate::StepflowError>;

    /// Validate URL
    async fn validate_url(&self, url: &str) -> Result<bool, crate::StepflowError>;

    /// Validate JSON
    async fn validate_json(&self, json: &str) -> Result<bool, crate::StepflowError>;

    /// Sanitize input
    async fn sanitize_input(&self, input: &str) -> Result<String, crate::StepflowError>;

    /// Sanitize path
    async fn sanitize_path(&self, path: &str) -> Result<String, crate::StepflowError>;
}

/// Rate limiter trait
#[async_trait]
pub trait RateLimiter: Send + Sync {
    /// Check rate limit
    async fn check_rate_limit(&self, key: &str, limit: u32, window: std::time::Duration) -> Result<bool, crate::StepflowError>;

    /// Increment counter
    async fn increment(&self, key: &str) -> Result<u32, crate::StepflowError>;

    /// Get current count
    async fn get_count(&self, key: &str) -> Result<u32, crate::StepflowError>;

    /// Reset counter
    async fn reset(&self, key: &str) -> Result<(), crate::StepflowError>;
}

/// Encryption trait
#[async_trait]
pub trait Encryptor: Send + Sync {
    /// Encrypt data
    async fn encrypt(&self, data: &[u8], key: &str) -> Result<Vec<u8>, crate::StepflowError>;

    /// Decrypt data
    async fn decrypt(&self, data: &[u8], key: &str) -> Result<Vec<u8>, crate::StepflowError>;

    /// Generate key
    async fn generate_key(&self) -> Result<String, crate::StepflowError>;

    /// Hash data
    async fn hash(&self, data: &[u8], salt: &str) -> Result<String, crate::StepflowError>;

    /// Verify hash
    async fn verify_hash(&self, data: &[u8], hash: &str, salt: &str) -> Result<bool, crate::StepflowError>;
}

/// Token manager trait
#[async_trait]
pub trait TokenManager: Send + Sync {
    /// Generate token
    async fn generate_token(&self, claims: &TokenClaims) -> Result<String, crate::StepflowError>;

    /// Validate token
    async fn validate_token(&self, token: &str) -> Result<TokenClaims, crate::StepflowError>;

    /// Refresh token
    async fn refresh_token(&self, token: &str) -> Result<String, crate::StepflowError>;

    /// Revoke token
    async fn revoke_token(&self, token: &str) -> Result<(), crate::StepflowError>;

    /// Get token info
    async fn get_token_info(&self, token: &str) -> Result<TokenInfo, crate::StepflowError>;
}

/// Token claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenClaims {
    pub user_id: crate::UserId,
    pub tenant_id: crate::TenantId,
    pub permissions: Vec<String>,
    pub roles: Vec<String>,
    pub issued_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub not_before: Option<chrono::DateTime<chrono::Utc>>,
    pub issuer: Option<String>,
    pub audience: Option<String>,
    pub subject: Option<String>,
}

/// Token info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub token_id: String,
    pub user_id: crate::UserId,
    pub tenant_id: crate::TenantId,
    pub issued_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub is_revoked: bool,
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
}

/// Session manager trait
#[async_trait]
pub trait SessionManager: Send + Sync {
    /// Create session
    async fn create_session(&self, user_id: &crate::UserId, tenant_id: &crate::TenantId) -> Result<Session, crate::StepflowError>;

    /// Get session
    async fn get_session(&self, session_id: &str) -> Result<Session, crate::StepflowError>;

    /// Update session
    async fn update_session(&self, session_id: &str, data: HashMap<String, serde_json::Value>) -> Result<(), crate::StepflowError>;

    /// Delete session
    async fn delete_session(&self, session_id: &str) -> Result<(), crate::StepflowError>;

    /// List sessions
    async fn list_sessions(&self, user_id: &crate::UserId) -> Result<Vec<Session>, crate::StepflowError>;

    /// Clean up expired sessions
    async fn cleanup_sessions(&self) -> Result<u64, crate::StepflowError>;
}

/// Session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub session_id: String,
    pub user_id: crate::UserId,
    pub tenant_id: crate::TenantId,
    pub data: HashMap<String, serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

/// Audit logger trait
#[async_trait]
pub trait AuditLogger: Send + Sync {
    /// Log audit event
    async fn log_event(&self, event: &AuditEvent) -> Result<(), crate::StepflowError>;

    /// Get audit events
    async fn get_events(&self, filter: Option<AuditFilter>) -> Result<Vec<AuditEvent>, crate::StepflowError>;

    /// Export audit log
    async fn export_log(&self, filter: Option<AuditFilter>, format: ExportFormat) -> Result<Vec<u8>, crate::StepflowError>;
}

/// Audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub event_id: String,
    pub event_type: String,
    pub user_id: Option<crate::UserId>,
    pub tenant_id: Option<crate::TenantId>,
    pub resource_type: String,
    pub resource_id: String,
    pub action: String,
    pub details: HashMap<String, serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Audit filter
#[derive(Debug, Clone)]
pub struct AuditFilter {
    pub event_type: Option<String>,
    pub user_id: Option<crate::UserId>,
    pub tenant_id: Option<crate::TenantId>,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub action: Option<String>,
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub success: Option<bool>,
}

/// Export format
#[derive(Debug, Clone)]
pub enum ExportFormat {
    Json,
    Csv,
    Xml,
    Pdf,
}

/// Security policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    pub policy_id: String,
    pub name: String,
    pub description: String,
    pub rules: Vec<SecurityRule>,
    pub enabled: bool,
    pub priority: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Security rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRule {
    pub rule_id: String,
    pub name: String,
    pub description: String,
    pub condition: SecurityCondition,
    pub action: SecurityAction,
    pub enabled: bool,
    pub priority: u32,
}

/// Security condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityCondition {
    pub field: String,
    pub operator: SecurityOperator,
    pub value: serde_json::Value,
    pub logical_operator: Option<LogicalOperator>,
}

/// Security operator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityOperator {
    Equals,
    NotEquals,
    Contains,
    NotContains,
    StartsWith,
    EndsWith,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    In,
    NotIn,
    IsNull,
    IsNotNull,
}

/// Logical operator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogicalOperator {
    And,
    Or,
    Not,
}

/// Security action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAction {
    pub action_type: SecurityActionType,
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Security action type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityActionType {
    Allow,
    Deny,
    Log,
    Alert,
    Block,
    RateLimit,
    Sanitize,
    Encrypt,
    Decrypt,
}

/// Security policy manager trait
#[async_trait]
pub trait SecurityPolicyManager: Send + Sync {
    /// Create policy
    async fn create_policy(&self, policy: &SecurityPolicy) -> Result<String, crate::StepflowError>;

    /// Get policy
    async fn get_policy(&self, policy_id: &str) -> Result<SecurityPolicy, crate::StepflowError>;

    /// Update policy
    async fn update_policy(&self, policy_id: &str, policy: &SecurityPolicy) -> Result<(), crate::StepflowError>;

    /// Delete policy
    async fn delete_policy(&self, policy_id: &str) -> Result<(), crate::StepflowError>;

    /// List policies
    async fn list_policies(&self, filter: Option<PolicyFilter>) -> Result<Vec<SecurityPolicy>, crate::StepflowError>;

    /// Evaluate policies
    async fn evaluate_policies(&self, context: &SecurityContext, resource: &str, action: &str) -> Result<PolicyResult, crate::StepflowError>;
}

/// Policy filter
#[derive(Debug, Clone)]
pub struct PolicyFilter {
    pub enabled: Option<bool>,
    pub priority: Option<u32>,
    pub created_after: Option<chrono::DateTime<chrono::Utc>>,
    pub created_before: Option<chrono::DateTime<chrono::Utc>>,
}

/// Policy result
#[derive(Debug, Clone)]
pub struct PolicyResult {
    pub allowed: bool,
    pub applied_policies: Vec<String>,
    pub actions: Vec<SecurityAction>,
    pub reason: Option<String>,
} 