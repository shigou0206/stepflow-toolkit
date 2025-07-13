//! Simple demo for stepflow-executor
//! 
//! This is a minimal working example that demonstrates:
//! - Basic executor setup
//! - Simple tool execution
//! - Error handling

use std::sync::Arc;
use std::collections::HashMap;
use std::time::Duration;
use stepflow_core::*;
use stepflow_database::SqliteDatabase;
use stepflow_registry::Registry;
use stepflow_executor::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Stepflow Executor Simple Demo");
    
    // Setup database
    let database = Arc::new(SqliteDatabase::new(":memory:").await?);
    stepflow_database::MigrationManager::run_migrations(&database).await?;
    
    // Create registry
    let registry = Arc::new(stepflow_registry::create_registry(database.clone()).await?);
    
    // Register a simple tool
    let tool = ToolInfo {
        id: ToolId::new(),
        name: "simple-tool".to_string(),
        description: "A simple demonstration tool".to_string(),
        version: ToolVersion::new(1, 0, 0),
        tool_type: ToolType::Python,
        status: ToolStatus::Active,
        author: "demo@stepflow.dev".to_string(),
        repository: None,
        documentation: None,
        tags: vec!["demo".to_string()],
        capabilities: vec!["process".to_string()],
        configuration_schema: None,
        examples: vec![],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    let tool_id = registry.register_tool(tool).await?;
    println!("✅ Registered tool: {}", tool_id);
    
    // Create executor
    let executor = stepflow_executor::create_default_executor(database, registry)?;
    println!("✅ Created executor");
    
    // Create execution request
    let request = ExecutionRequest {
        tool_id: tool_id.clone(),
        version: None,
        parameters: HashMap::from([
            ("input".to_string(), serde_json::json!("Hello, World!")),
            ("operation".to_string(), serde_json::json!("process")),
        ]),
        context: ExecutionContext {
            user_id: "demo-user".to_string(),
            tenant_id: "demo-tenant".to_string(),
            session_id: "demo-session".to_string(),
            request_id: "demo-request".to_string(),
            parent_execution_id: None,
            environment: HashMap::new(),
        },
        options: ExecutionOptions {
            timeout: Some(Duration::from_secs(30)),
            retry_count: 2,
            retry_delay: Duration::from_millis(1000),
            priority: Priority::Normal,
            resource_limits: ResourceLimits::default(),
            logging_level: LogLevel::Info,
        },
    };
    
    // Execute tool
    println!("🔄 Executing tool...");
    match executor.execute_tool(request).await {
        Ok(result) => {
            println!("✅ Execution completed successfully:");
            println!("   Success: {}", result.success);
            if let Some(output) = &result.output {
                println!("   Output: {}", output);
            }
            if !result.logs.is_empty() {
                println!("   Logs: {} entries", result.logs.len());
            }
            if !result.metrics.is_empty() {
                println!("   Metrics: {} entries", result.metrics.len());
            }
        }
        Err(e) => {
            println!("❌ Execution failed: {}", e);
            return Err(e.into());
        }
    }
    
    // Test async execution
    println!("\n🔄 Testing async execution...");
    let async_request = ExecutionRequest {
        tool_id: tool_id.clone(),
        version: None,
        parameters: HashMap::from([
            ("input".to_string(), serde_json::json!("Async test")),
            ("operation".to_string(), serde_json::json!("async_process")),
        ]),
        context: ExecutionContext {
            user_id: "demo-user".to_string(),
            tenant_id: "demo-tenant".to_string(),
            session_id: "demo-session-async".to_string(),
            request_id: "demo-request-async".to_string(),
            parent_execution_id: None,
            environment: HashMap::new(),
        },
        options: ExecutionOptions {
            timeout: Some(Duration::from_secs(30)),
            retry_count: 1,
            retry_delay: Duration::from_millis(500),
            priority: Priority::Normal,
            resource_limits: ResourceLimits::default(),
            logging_level: LogLevel::Info,
        },
    };
    
    match executor.execute_tool_async(async_request).await {
        Ok(execution_id) => {
            println!("✅ Async execution started: {}", execution_id);
            
            // Check status
            tokio::time::sleep(Duration::from_millis(100)).await;
            match executor.get_execution_status(&execution_id).await {
                Ok(status) => {
                    println!("   Status: {:?}", status);
                }
                Err(e) => {
                    println!("   Status check failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Async execution failed: {}", e);
        }
    }
    
    // Test error handling
    println!("\n🔄 Testing error handling...");
    let error_request = ExecutionRequest {
        tool_id: ToolId::from_string("non-existent-tool".to_string()),
        version: None,
        parameters: HashMap::new(),
        context: ExecutionContext {
            user_id: "demo-user".to_string(),
            tenant_id: "demo-tenant".to_string(),
            session_id: "demo-session-error".to_string(),
            request_id: "demo-request-error".to_string(),
            parent_execution_id: None,
            environment: HashMap::new(),
        },
        options: ExecutionOptions {
            timeout: Some(Duration::from_secs(5)),
            retry_count: 1,
            retry_delay: Duration::from_millis(100),
            priority: Priority::Normal,
            resource_limits: ResourceLimits::default(),
            logging_level: LogLevel::Info,
        },
    };
    
    match executor.execute_tool(error_request).await {
        Ok(result) => {
            println!("⚠️ Expected error but got result: {:?}", result);
        }
        Err(e) => {
            println!("✅ Error handled correctly: {}", e);
        }
    }
    
    println!("\n🎉 Demo completed successfully!");
    Ok(())
} 