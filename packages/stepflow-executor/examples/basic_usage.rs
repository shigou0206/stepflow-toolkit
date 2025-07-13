//! Basic usage example for stepflow-executor
//! 
//! This example demonstrates how to:
//! - Set up the executor with database and registry
//! - Execute tools synchronously and asynchronously
//! - Handle execution results and errors
//! - Monitor execution status

use std::sync::Arc;
use std::collections::HashMap;
use std::time::Duration;
use stepflow_core::*;
use stepflow_database::SqliteDatabase;
use stepflow_registry::{Registry, RegistryImpl};
use stepflow_executor::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Stepflow Executor Basic Usage Example");
    
    // 1. Setup database and registry
    let (database, registry) = setup_infrastructure().await?;
    
    // 2. Create executor
    let executor = create_executor(database.clone(), registry.clone()).await?;
    
    // 3. Basic synchronous execution
    println!("\n📋 Example 1: Basic Synchronous Execution");
    basic_sync_execution(&executor).await?;
    
    // 4. Asynchronous execution with status monitoring
    println!("\n⚡ Example 2: Asynchronous Execution with Monitoring");
    async_execution_with_monitoring(&executor).await?;
    
    // 5. Error handling
    println!("\n❌ Example 3: Error Handling");
    error_handling_example(&executor).await?;
    
    println!("\n✅ All examples completed successfully!");
    Ok(())
}

/// Setup database and registry infrastructure
async fn setup_infrastructure() -> Result<(Arc<SqliteDatabase>, Arc<RegistryImpl>), Box<dyn std::error::Error>> {
    println!("Setting up database and registry...");
    
    // Create database
    let database = Arc::new(SqliteDatabase::new(":memory:").await?);
    
    // Run migrations
    stepflow_database::MigrationManager::run_migrations(&database).await?;
    
    // Create registry
    let registry = Arc::new(stepflow_registry::create_registry(database.clone()).await?);
    
    // Register sample tools
    register_sample_tools(&registry).await?;
    
    Ok((database, registry))
}

/// Register sample tools for testing
async fn register_sample_tools(registry: &Arc<RegistryImpl>) -> Result<(), Box<dyn std::error::Error>> {
    use chrono::Utc;
    
    // Register a basic Python tool
    let python_tool = ToolInfo {
        id: ToolId::from_string("python-calculator".to_string()),
        name: "Python Calculator".to_string(),
        description: "A simple calculator tool implemented in Python".to_string(),
        version: ToolVersion::new(1, 0, 0),
        tool_type: ToolType::Python,
        status: ToolStatus::Active,
        author: "example@stepflow.dev".to_string(),
        repository: Some("https://github.com/stepflow/python-calculator".to_string()),
        documentation: Some("https://docs.stepflow.dev/tools/python-calculator".to_string()),
        tags: vec!["calculator".to_string(), "math".to_string(), "python".to_string()],
        capabilities: vec!["add".to_string(), "subtract".to_string(), "multiply".to_string(), "divide".to_string()],
        configuration_schema: None,
        examples: vec![],
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    registry.register_tool(python_tool).await?;
    
    // Register a Openapi tool (since Rust variant doesn't exist)
    let js_tool = ToolInfo {
        id: ToolId::from_string("js-text-processor".to_string()),
        name: "JavaScript Text Processor".to_string(),
        description: "A text processing tool implemented in OpenAPI".to_string(),
        version: ToolVersion::new(2, 1, 0),
        tool_type: ToolType::OpenAPI, // Fixed: Use an existing ToolType variant, e.g., NodeJS
        status: ToolStatus::Active,
        author: "example@stepflow.dev".to_string(),
        repository: Some("https://github.com/stepflow/js-text-processor".to_string()),
        documentation: Some("https://docs.stepflow.dev/tools/js-text-processor".to_string()),
        tags: vec!["text".to_string(), "processing".to_string(), "openapi".to_string()],
        capabilities: vec!["uppercase".to_string(), "lowercase".to_string(), "reverse".to_string()],
        configuration_schema: None,
        examples: vec![],
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    registry.register_tool(js_tool).await?;
    
    println!("✅ Sample tools registered successfully");
    Ok(())
}

/// Create executor with proper configuration
async fn create_executor(
    database: Arc<SqliteDatabase>,
    registry: Arc<RegistryImpl>,
) -> Result<Arc<dyn Executor>, Box<dyn std::error::Error>> {
    let executor = stepflow_executor::create_default_executor(database, registry)?;
    println!("✅ Executor created successfully");
    Ok(Arc::new(executor))
}

/// Example 1: Basic synchronous execution
async fn basic_sync_execution(executor: &Arc<dyn Executor>) -> Result<(), Box<dyn std::error::Error>> {
    let request = ExecutionRequest {
        tool_id: ToolId::from_string("python-calculator".to_string()),
        version: None,
        parameters: HashMap::from([
            ("operation".to_string(), serde_json::json!("add")),
            ("operands".to_string(), serde_json::json!([10, 20])),
        ]),
        context: ExecutionContext {
            user_id: "user-1".to_string(),
            tenant_id: "default".to_string(),
            session_id: "session-1".to_string(),
            request_id: "req-1".to_string(),
            parent_execution_id: None,
            environment: HashMap::new(),
        },
        options: ExecutionOptions {
            timeout: Some(Duration::from_secs(10)),
            retry_count: 2,
            retry_delay: Duration::from_millis(500),
            priority: Priority::Normal,
            resource_limits: ResourceLimits::default(),
            logging_level: LogLevel::Info,
        },
    };
    
    match executor.execute_tool(request).await {
        Ok(result) => {
            println!("✅ Synchronous execution completed:");
            println!("   Success: {}", result.success);
            if let Some(output) = &result.output {
                println!("   Output: {}", output);
            }
            if let Some(error) = &result.error {
                println!("   Error: {}", error);
            }
        }
        Err(e) => {
            println!("❌ Synchronous execution failed: {}", e);
        }
    }
    
    Ok(())
}

/// Example 2: Asynchronous execution with status monitoring
async fn async_execution_with_monitoring(executor: &Arc<dyn Executor>) -> Result<(), Box<dyn std::error::Error>> {
    let request = ExecutionRequest {
        tool_id: ToolId::from_string("js-text-processor".to_string()),
        version: None,
        parameters: HashMap::from([
            ("text".to_string(), serde_json::json!("Hello, World!")),
            ("operation".to_string(), serde_json::json!("uppercase")),
        ]),
        context: ExecutionContext {
            user_id: "user-1".to_string(),
            tenant_id: "default".to_string(),
            session_id: "session-2".to_string(),
            request_id: "req-2".to_string(),
            parent_execution_id: None,
            environment: HashMap::new(),
        },
        options: ExecutionOptions {
            timeout: Some(Duration::from_secs(15)),
            retry_count: 3,
            retry_delay: Duration::from_millis(1000),
            priority: Priority::High,
            resource_limits: ResourceLimits::default(),
            logging_level: LogLevel::Info,
        },
    };
    
    // Start asynchronous execution
    match executor.execute_tool_async(request).await {
        Ok(execution_id) => {
            println!("✅ Asynchronous execution started: {}", execution_id);
            
            // Monitor execution status
            let mut attempts = 0;
            loop {
                if attempts > 10 {
                    println!("⏰ Timeout waiting for execution to complete");
                    break;
                }
                
                match executor.get_execution_status(&execution_id).await {
                    Ok(status) => {
                        println!("   Status: {:?}", status);
                        
                        if matches!(status, ExecutionStatus::Completed | ExecutionStatus::Failed) {
                            // Get final result
                            if let Ok(result) = executor.get_execution_result(&execution_id).await {
                                println!("   Final result: Success={}, Output={:?}", 
                                    result.success, result.output);
                            }
                            break;
                        }
                    }
                    Err(e) => {
                        println!("❌ Failed to get execution status: {}", e);
                        break;
                    }
                }
                
                attempts += 1;
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
        Err(e) => {
            println!("❌ Failed to start asynchronous execution: {}", e);
        }
    }
    
    Ok(())
}

/// Example 3: Error handling
async fn error_handling_example(executor: &Arc<dyn Executor>) -> Result<(), Box<dyn std::error::Error>> {
    // Try to execute a non-existent tool
    let request = ExecutionRequest {
        tool_id: ToolId::from_string("non-existent-tool".to_string()),
        version: None,
        parameters: HashMap::new(),
        context: ExecutionContext {
            user_id: "user-1".to_string(),
            tenant_id: "default".to_string(),
            session_id: "session-3".to_string(),
            request_id: "req-error-1".to_string(),
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
    
    match executor.execute_tool(request).await {
        Ok(result) => {
            println!("⚠️ Expected error but got result: {:?}", result);
        }
        Err(e) => {
            println!("✅ Properly handled error: {}", e);
            
            // Demonstrate error type handling
            match e {
                ExecutorError::ToolNotFound(tool_id) => {
                    println!("   Error type: Tool not found - {}", tool_id);
                }
                ExecutorError::TimeoutExceeded => {
                    println!("   Error type: Execution timeout");
                }
                ExecutorError::InvalidParameters(msg) => {
                    println!("   Error type: Invalid parameters - {}", msg);
                }
                ExecutorError::DatabaseError(msg) => {
                    println!("   Error type: Database error - {}", msg);
                }
                _ => {
                    println!("   Error type: Other - {}", e);
                }
            }
        }
    }
    
    Ok(())
} 