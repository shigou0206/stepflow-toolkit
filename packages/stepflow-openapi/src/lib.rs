//! stepflow-openapi
//! 
//! 这个包提供了 OpenAPI 规范解析和 JSON RPC 代理服务功能。

pub mod cst;
pub mod ast;
pub mod openapi_3_0;
pub mod proxy;
pub mod srn;
pub mod document;
pub mod ref_resolver;
pub mod tool;
pub mod generator;
pub mod registry;

// 重新导出主要的公共 API
pub use proxy::*;
pub use cst::{CstParser, SourceType, TreeCursorSyntaxNode};
pub use ast::fold::*;
pub use srn::{Srn, SrnBuilder, SrnComponents, SrnError};
pub use document::{DocumentManager, DocumentUploadRequest, DocumentUploadResult, OpenApiDocument, OperationInfo, SchemaInfo};
pub use ref_resolver::{RefResolver, RefResolverConfig, RefResolverError, resolve_refs};
pub use tool::{OpenApiTool, OpenApiToolConfig, OpenApiToolError, AuthConfig};
pub use generator::{ToolGenerator, ToolGenerationRequest, ToolGenerationResult, GeneratorConfig, ToolRegistry, InMemoryToolRegistry};
pub use registry::{OpenApiToolRegistry, RegistryConfig, ToolSearchCriteria, ToolExecutionStats, GlobalRegistryStats};