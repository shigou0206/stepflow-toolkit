//! Stepflow Core - Core interfaces and types for Stepflow Tool System
//!
//! This crate provides the foundational interfaces, types, and traits that define
//! the Stepflow Tool System architecture. It serves as the common foundation
//! for all other packages in the system.

pub mod types;
pub mod traits;
pub mod errors;
pub mod config;
pub mod security;
pub mod monitoring;
pub mod models;

// Re-export specific types to avoid conflicts
pub use types::{
    ToolId, ToolVersion, ToolType, ToolStatus, ToolInfo, ToolExample, ToolConfig,
    ToolRequest, ToolResponse, ToolStats, ExecutionId, ExecutionStatus, ExecutionResult,
    LogEntry, LogLevel, TenantId, TenantInfo, UserId, UserRole, UserInfo,
    Pagination, Filter, FilterOperator, Sort, SortDirection, Query, Metric, Execution
};
pub use traits::{
    ToolRegistry, ToolExecutor, ExecutionManager, TenantManager, UserManager,
    Authenticator, Authorizer, Monitor, Cache, Database, ToolFilter,
    ExecutionFilter, TenantFilter, UserFilter, Credentials, AuthResult, Permission,
    Event, MetricFilter, EventFilter, CacheStats, QueryResult, Migration, DatabaseStats
};
pub use errors::{
    StepflowError, StepflowResult, DatabaseError, ValidationError, SecurityError,
    NetworkError, ConfigurationError, MonitoringError
};
pub use models::{
    ToolSpec, ExecutionConfig, SandboxLevel, RetryConfig, BackoffStrategy
};
pub use config::*;
pub use security::*;
pub use monitoring::*;

/// Re-export commonly used types for convenience
pub mod prelude {
    pub use super::{
        ToolId, ToolInfo, ToolStatus, ToolType, ToolVersion,
        ExecutionId, ExecutionStatus, ExecutionResult,
        TenantId, TenantInfo,
        UserId, UserInfo, UserRole,
        StepflowError, StepflowResult,
    };
    pub use crate::types::Execution;
    pub use crate::config::{Config, SecurityConfig, MonitoringConfig};
} 