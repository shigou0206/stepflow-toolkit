//! Stepflow API - HTTP REST API and GraphQL Server
//! 
//! This module provides the HTTP REST API and GraphQL server for the Stepflow Tool System,
//! including authentication, rate limiting, monitoring, and comprehensive API endpoints.

pub mod errors;
pub mod types;
pub mod server;
pub mod models;
pub mod middleware;
pub mod routes;
pub mod handlers;
pub mod graphql;

// Re-export commonly used types and traits
pub use errors::*;
pub use types::*;
pub use server::*;
pub use models::*;

// Re-export middleware
pub use middleware::*;

// Re-export routes and handlers
pub use routes::*;
pub use handlers::*;

// Re-export GraphQL
pub use graphql::*;

/// API 包版本
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// API 包名称
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// API 包描述
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

/// 默认 API 版本
pub const DEFAULT_API_VERSION: ApiVersion = ApiVersion::V1; 