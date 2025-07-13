//! Advanced usage example for stepflow-executor
//! 
//! This example demonstrates advanced features:
//! - Batch execution with different tools
//! - Resource management and monitoring
//! - Custom execution options and environments
//! - Performance testing and metrics collection

use std::sync::Arc;
use std::collections::HashMap;
use std::time::Duration;
use stepflow_core::*;
use stepflow_database::SqliteDatabase;
use stepflow_registry::{Registry, RegistryImpl};
use stepflow_executor::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Stepflow Executor Advanced Usage Example");
    
    // Setup infrastructure
    let (database, registry) = setup_infrastructure().await?;
    let executor = create_executor(database.clone(), registry.clone()).await?;
    
    // Advanced examples
    println!("\n🔄 Example 1: Batch Execution");
    batch_execution_example(&executor).await?;
    
    println!("\n⚙️ Example 2: Custom Resource Management");
    resource_management_example(&executor).await?;
    
    println!("\n📊 Example 3: Performance Monitoring");
    performance_monitoring_example(&executor).await?;
    
    println!("\n🌍 Example 4: Environment Variables");
    environment_variables_example(&executor).await?;
    
    println!("\n✅ All advanced examples completed successfully!");
    Ok(())
}

/// Setup database and registry infrastructure
async fn setup_infrastructure() -> Result<(Arc<SqliteDatabase>, Arc<RegistryImpl>), Box<dyn std::error::Error>> {
    let database = Arc::new(SqliteDatabase::new(":memory:").await?);
    stepflow_database::MigrationManager::run_migrations(&database).await?;
    let registry = Arc::new(stepflow_registry::create_registry(database.clone()).await?);
    
    // Register multiple tools for testing
    register_test_tools(&registry).await?;
    
    Ok((database, registry))
}

/// Register test tools
async fn register_test_tools(registry: &Arc<RegistryImpl>) -> Result<(), Box<dyn std::error::Error>> {
    use chrono::Utc;
    
    let tools = vec![
        ToolInfo {
            id: ToolId::from_string("data-processor".to_string()),
            name: "Data Processor".to_string(),
            description: "Process large datasets".to_string(),
            version: ToolVersion::new(1, 0, 0),
            tool_type: ToolType::Python,
            status: ToolStatus::Active,
            author: "data-team@stepflow.dev".to_string(),
            repository: Some("https://github.com/stepflow/data-processor".to_string()),
            documentation: Some("https://docs.stepflow.dev/tools/data-processor".to_string()),
            tags: vec!["data".to_string(), "processing".to_string()],
            capabilities: vec!["transform".to_string(), "aggregate".to_string()],
            configuration_schema: None,
            examples: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
        ToolInfo {
            id: ToolId::from_string("file-converter".to_string()),
            name: "File Converter".to_string(),
            description: "Convert between different file formats".to_string(),
            version: ToolVersion::new(2, 0, 0),
            tool_type: ToolType::System,
            status: ToolStatus::Active,
            author: "file-team@stepflow.dev".to_string(),
            repository: Some("https://github.com/stepflow/file-converter".to_string()),
            documentation: Some("https://docs.stepflow.dev/tools/file-converter".to_string()),
            tags: vec!["files".to_string(), "conversion".to_string()],
            capabilities: vec!["convert".to_string(), "validate".to_string()],
            configuration_schema: None,
            examples: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
        ToolInfo {
            id: ToolId::from_string("image-processor".to_string()),
            name: "Image Processor".to_string(),
            description: "Process and manipulate images".to_string(),
            version: ToolVersion::new(1, 2, 0),
            tool_type: ToolType::Python,
            status: ToolStatus::Active,
            author: "image-team@stepflow.dev".to_string(),
            repository: Some("https://github.com/stepflow/image-processor".to_string()),
            documentation: Some("https://docs.stepflow.dev/tools/image-processor".to_string()),
            tags: vec!["image".to_string(), "processing".to_string()],
            capabilities: vec!["resize".to_string(), "filter".to_string()],
            configuration_schema: None,
            examples: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
    ];
    
    for tool in tools {
        registry.register_tool(tool).await?;
    }
    
    println!("✅ Test tools registered successfully");
    Ok(())
}

/// Create executor
async fn create_executor(
    database: Arc<SqliteDatabase>,
    registry: Arc<RegistryImpl>,
) -> Result<Arc<dyn Executor>, Box<dyn std::error::Error>> {
    let executor = stepflow_executor::create_default_executor(database, registry)?;
    Ok(Arc::new(executor))
}

/// Example 1: Batch execution with different tools
async fn batch_execution_example(executor: &Arc<dyn Executor>) -> Result<(), Box<dyn std::error::Error>> {
    let tools = vec![
        ("data-processor", "transform", serde_json::json!({"input": "data.csv", "output": "processed.json"})),
        ("file-converter", "convert", serde_json::json!({"from": "csv", "to": "json"})),
        ("image-processor", "resize", serde_json::json!({"width": 800, "height": 600})),
    ];
    
    println!("Executing {} tools in batch...", tools.len());
    
    let mut execution_ids = Vec::new();
    
    // Start all executions asynchronously
    for (i, (tool_id, operation, params)) in tools.iter().enumerate() {
        let request = ExecutionRequest {
            tool_id: ToolId::from_string(tool_id.to_string()),
            version: None,
            parameters: HashMap::from([
                ("operation".to_string(), serde_json::json!(operation)),
                ("params".to_string(), params.clone()),
            ]),
            context: ExecutionContext {
                user_id: "batch-user".to_string(),
                tenant_id: "default".to_string(),
                session_id: format!("batch-session-{}", i),
                request_id: format!("batch-req-{}", i),
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
        
        match executor.execute_tool_async(request).await {
            Ok(execution_id) => {
                println!("   Started execution for {}: {}", tool_id, execution_id);
                execution_ids.push((tool_id.to_string(), execution_id));
            }
            Err(e) => {
                println!("   Failed to start execution for {}: {}", tool_id, e);
            }
        }
    }
    
    // Monitor all executions
    println!("Monitoring {} executions...", execution_ids.len());
    let mut completed = 0;
    let mut attempts = 0;
    
    while completed < execution_ids.len() && attempts < 50 {
        for (tool_id, execution_id) in &execution_ids {
            match executor.get_execution_status(execution_id).await {
                Ok(status) => {
                    match status {
                        ExecutionStatus::Completed => {
                            if let Ok(result) = executor.get_execution_result(execution_id).await {
                                println!("   ✅ {} completed: Success={}", tool_id, result.success);
                                completed += 1;
                            }
                        }
                        ExecutionStatus::Failed => {
                            println!("   ❌ {} failed", tool_id);
                            completed += 1;
                        }
                        _ => {
                            // Still running
                        }
                    }
                }
                Err(e) => {
                    println!("   ⚠️ Error checking status for {}: {}", tool_id, e);
                }
            }
        }
        
        attempts += 1;
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    
    println!("Batch execution completed: {}/{} tools finished", completed, execution_ids.len());
    Ok(())
}

/// Example 2: Resource management
async fn resource_management_example(executor: &Arc<dyn Executor>) -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing resource management...");
    
    // High-resource execution
    let high_resource_request = ExecutionRequest {
        tool_id: ToolId::from_string("data-processor".to_string()),
        version: None,
        parameters: HashMap::from([
            ("operation".to_string(), serde_json::json!("heavy_processing")),
            ("dataset_size".to_string(), serde_json::json!("large")),
        ]),
        context: ExecutionContext {
            user_id: "resource-user".to_string(),
            tenant_id: "default".to_string(),
            session_id: "resource-session".to_string(),
            request_id: "resource-req-1".to_string(),
            parent_execution_id: None,
            environment: HashMap::new(),
        },
        options: ExecutionOptions {
            timeout: Some(Duration::from_secs(60)),
            retry_count: 1,
            retry_delay: Duration::from_millis(2000),
            priority: Priority::High,
            resource_limits: ResourceLimits {
                memory_limit: Some(2 * 1024 * 1024 * 1024), // 2GB
                cpu_limit: Some(80.0), // 80% CPU
                execution_time_limit: Some(Duration::from_secs(45)),
                network_limit: Some(100 * 1024 * 1024), // 100MB
            },
            logging_level: LogLevel::Debug,
        },
    };
    
    match executor.execute_tool(high_resource_request).await {
        Ok(result) => {
            println!("   ✅ High-resource execution completed:");
            println!("      Success: {}", result.success);
            println!("      Metrics: {:?}", result.metrics);
        }
        Err(e) => {
            println!("   ❌ High-resource execution failed: {}", e);
        }
    }
    
    // Low-resource execution
    let low_resource_request = ExecutionRequest {
        tool_id: ToolId::from_string("file-converter".to_string()),
        version: None,
        parameters: HashMap::from([
            ("operation".to_string(), serde_json::json!("light_conversion")),
            ("file_size".to_string(), serde_json::json!("small")),
        ]),
        context: ExecutionContext {
            user_id: "resource-user".to_string(),
            tenant_id: "default".to_string(),
            session_id: "resource-session".to_string(),
            request_id: "resource-req-2".to_string(),
            parent_execution_id: None,
            environment: HashMap::new(),
        },
        options: ExecutionOptions {
            timeout: Some(Duration::from_secs(10)),
            retry_count: 3,
            retry_delay: Duration::from_millis(500),
            priority: Priority::Low,
            resource_limits: ResourceLimits {
                memory_limit: Some(256 * 1024 * 1024), // 256MB
                cpu_limit: Some(25.0), // 25% CPU
                execution_time_limit: Some(Duration::from_secs(5)),
                network_limit: Some(10 * 1024 * 1024), // 10MB
            },
            logging_level: LogLevel::Info,
        },
    };
    
    match executor.execute_tool(low_resource_request).await {
        Ok(result) => {
            println!("   ✅ Low-resource execution completed:");
            println!("      Success: {}", result.success);
            println!("      Metrics: {:?}", result.metrics);
        }
        Err(e) => {
            println!("   ❌ Low-resource execution failed: {}", e);
        }
    }
    
    Ok(())
}

/// Example 3: Performance monitoring
async fn performance_monitoring_example(executor: &Arc<dyn Executor>) -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing performance monitoring...");
    
    let start_time = std::time::Instant::now();
    let mut execution_times = Vec::new();
    
    // Run multiple executions to collect performance data
    for i in 0..5 {
        let execution_start = std::time::Instant::now();
        
        let request = ExecutionRequest {
            tool_id: ToolId::from_string("image-processor".to_string()),
            version: None,
            parameters: HashMap::from([
                ("operation".to_string(), serde_json::json!("filter")),
                ("filter_type".to_string(), serde_json::json!("blur")),
                ("intensity".to_string(), serde_json::json!(i + 1)),
            ]),
            context: ExecutionContext {
                user_id: "perf-user".to_string(),
                tenant_id: "default".to_string(),
                session_id: format!("perf-session-{}", i),
                request_id: format!("perf-req-{}", i),
                parent_execution_id: None,
                environment: HashMap::new(),
            },
            options: ExecutionOptions {
                timeout: Some(Duration::from_secs(20)),
                retry_count: 1,
                retry_delay: Duration::from_millis(500),
                priority: Priority::Normal,
                resource_limits: ResourceLimits::default(),
                logging_level: LogLevel::Info,
            },
        };
        
        match executor.execute_tool(request).await {
            Ok(result) => {
                let execution_time = execution_start.elapsed();
                execution_times.push(execution_time);
                
                println!("   Execution {}: Success={}, Duration={:?}", 
                    i + 1, result.success, execution_time);
                
                // Extract metrics if available
                if !result.metrics.is_empty() {
                    println!("      Metrics: {:?}", result.metrics);
                }
            }
            Err(e) => {
                println!("   Execution {} failed: {}", i + 1, e);
            }
        }
    }
    
    // Calculate performance statistics
    let total_time = start_time.elapsed();
    let avg_time = if !execution_times.is_empty() {
        execution_times.iter().sum::<Duration>() / execution_times.len() as u32
    } else {
        Duration::from_secs(0)
    };
    
    let min_time = execution_times.iter().min().copied().unwrap_or(Duration::from_secs(0));
    let max_time = execution_times.iter().max().copied().unwrap_or(Duration::from_secs(0));
    
    println!("   Performance Summary:");
    println!("      Total time: {:?}", total_time);
    println!("      Average execution time: {:?}", avg_time);
    println!("      Min execution time: {:?}", min_time);
    println!("      Max execution time: {:?}", max_time);
    println!("      Throughput: {:.2} executions/second", 
        execution_times.len() as f64 / total_time.as_secs_f64());
    
    Ok(())
}

/// Example 4: Environment variables
async fn environment_variables_example(executor: &Arc<dyn Executor>) -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing environment variables...");
    
    // Create custom environment
    let mut custom_env = HashMap::new();
    custom_env.insert("DEBUG".to_string(), "true".to_string());
    custom_env.insert("LOG_LEVEL".to_string(), "verbose".to_string());
    custom_env.insert("CACHE_ENABLED".to_string(), "false".to_string());
    custom_env.insert("MAX_WORKERS".to_string(), "4".to_string());
    custom_env.insert("API_KEY".to_string(), "test-key-12345".to_string());
    
    let request = ExecutionRequest {
        tool_id: ToolId::from_string("data-processor".to_string()),
        version: None,
        parameters: HashMap::from([
            ("operation".to_string(), serde_json::json!("analyze")),
            ("use_cache".to_string(), serde_json::json!(false)),
        ]),
        context: ExecutionContext {
            user_id: "env-user".to_string(),
            tenant_id: "default".to_string(),
            session_id: "env-session".to_string(),
            request_id: "env-req-1".to_string(),
            parent_execution_id: None,
            environment: custom_env,
        },
        options: ExecutionOptions {
            timeout: Some(Duration::from_secs(30)),
            retry_count: 2,
            retry_delay: Duration::from_millis(1000),
            priority: Priority::Normal,
            resource_limits: ResourceLimits::default(),
            logging_level: LogLevel::Debug,
        },
    };
    
    match executor.execute_tool(request).await {
        Ok(result) => {
            println!("   ✅ Environment variables execution completed:");
            println!("      Success: {}", result.success);
            if let Some(output) = &result.output {
                println!("      Output: {}", output);
            }
            
            // Check if environment variables affected the execution
            if let Some(metadata) = result.metadata.get("environment_used") {
                println!("      Environment variables used: {}", metadata);
            }
        }
        Err(e) => {
            println!("   ❌ Environment variables execution failed: {}", e);
        }
    }
    
    Ok(())
} 