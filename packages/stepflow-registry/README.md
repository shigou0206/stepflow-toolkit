# Stepflow Registry

工具注册表是 Stepflow Tool System 的核心组件，负责工具的注册、发现、版本管理和元数据存储。

## 特性

- 工具注册和管理
- 工具搜索和发现
- 按类型和状态过滤工具
- 完整的 CRUD 操作
- 基于 SQLite 的持久化存储
- 异步操作支持

## 快速开始

### 安装

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
stepflow-registry = { path = "../stepflow-registry" }
stepflow-database = { path = "../stepflow-database" }
stepflow-core = { path = "../stepflow-core" }
```

### 基本使用

```rust
use stepflow_registry::{create_registry, RegistryResult};
use stepflow_database::SqliteDatabase;
use stepflow_core::*;
use std::sync::Arc;
use chrono::Utc;

#[tokio::main]
async fn main() -> RegistryResult<()> {
    // 创建数据库连接
    let db = Arc::new(SqliteDatabase::new("registry.db").await?);
    
    // 运行数据库迁移
    stepflow_database::MigrationManager::run_migrations(&db).await?;
    
    // 创建注册表
    let registry = create_registry(db).await?;
    
    // 创建工具信息
    let tool = ToolInfo {
        id: ToolId::new(),
        name: "my-python-tool".to_string(),
        description: "A Python tool for data processing".to_string(),
        version: ToolVersion::new(1, 0, 0),
        tool_type: ToolType::Python,
        status: ToolStatus::Active,
        author: "developer@example.com".to_string(),
        repository: Some("https://github.com/example/my-tool".to_string()),
        documentation: Some("https://docs.example.com/my-tool".to_string()),
        tags: vec!["python".to_string(), "data".to_string(), "processing".to_string()],
        capabilities: vec!["transform".to_string(), "validate".to_string()],
        configuration_schema: None,
        examples: vec![],
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    // 注册工具
    let tool_id = registry.register_tool(tool).await?;
    println!("Tool registered with ID: {}", tool_id);
    
    // 获取工具
    let retrieved_tool = registry.get_tool(&tool_id).await?;
    println!("Retrieved tool: {}", retrieved_tool.name);
    
    // 搜索工具
    let search_results = registry.search_tools("python").await?;
    println!("Found {} tools matching 'python'", search_results.len());
    
    // 列出所有工具
    let all_tools = registry.list_tools().await?;
    println!("Total tools: {}", all_tools.len());
    
    // 按类型获取工具
    let python_tools = registry.get_tools_by_type(&ToolType::Python).await?;
    println!("Python tools: {}", python_tools.len());
    
    // 按状态获取工具
    let active_tools = registry.get_tools_by_status(&ToolStatus::Active).await?;
    println!("Active tools: {}", active_tools.len());
    
    Ok(())
}
```

## API 参考

### SimpleRegistry

主要的注册表实现，提供以下方法：

#### `register_tool(tool: ToolInfo) -> RegistryResult<ToolId>`

注册一个新工具。

#### `get_tool(tool_id: &ToolId) -> RegistryResult<ToolInfo>`

根据 ID 获取工具信息。

#### `list_tools() -> RegistryResult<Vec<ToolInfo>>`

列出所有已注册的工具。

#### `search_tools(query: &str) -> RegistryResult<Vec<ToolInfo>>`

根据查询字符串搜索工具。

#### `update_tool(tool_id: &ToolId, tool: &ToolInfo) -> RegistryResult<()>`

更新工具信息。

#### `delete_tool(tool_id: &ToolId) -> RegistryResult<()>`

删除工具。

#### `get_tools_by_type(tool_type: &ToolType) -> RegistryResult<Vec<ToolInfo>>`

根据工具类型获取工具列表。

#### `get_tools_by_status(status: &ToolStatus) -> RegistryResult<Vec<ToolInfo>>`

根据状态获取工具列表。

## 错误处理

注册表使用 `RegistryResult<T>` 类型来处理错误，主要的错误类型包括：

- `RegistryError::ToolNotFound` - 工具未找到
- `RegistryError::DatabaseError` - 数据库操作错误
- `RegistryError::InternalError` - 内部错误

## 测试

运行测试：

```bash
cargo test
```

## 许可证

MIT License 