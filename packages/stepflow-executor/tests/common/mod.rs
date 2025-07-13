//! Common test utilities and fixtures

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use chrono::Utc;
use stepflow_core::*;
use stepflow_database::SqliteDatabase;
use stepflow_registry::{Registry, RegistryImpl};
use stepflow_executor::*;

/// Test database setup
pub async fn setup_test_database() -> Arc<SqliteDatabase> {
    let db = Arc::new(SqliteDatabase::new(":memory:").await.unwrap());
    
    // Create tables for testing - match the actual database schema
    db.execute(
        r#"
        CREATE TABLE IF NOT EXISTS tools (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            version_major INTEGER NOT NULL,
            version_minor INTEGER NOT NULL,
            version_patch INTEGER NOT NULL,
            version_pre_release TEXT,
            version_build TEXT,
            tool_type TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'active',
            author TEXT NOT NULL,
            repository TEXT,
            documentation TEXT,
            tags TEXT,
            capabilities TEXT,
            configuration_schema TEXT,
            examples TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )
        "#,
        &[],
    ).await.unwrap();
    
    db.execute(
        r#"
        CREATE TABLE IF NOT EXISTS tasks (
            id TEXT PRIMARY KEY,
            tool_id TEXT NOT NULL,
            execution_request TEXT NOT NULL,
            priority INTEGER NOT NULL,
            status TEXT NOT NULL DEFAULT 'queued',
            task_data TEXT,
            created_at TEXT NOT NULL,
            scheduled_at TEXT,
            started_at TEXT,
            completed_at TEXT
        )
        "#,
        &[],
    ).await.unwrap();
    
    db.execute(
        r#"
        CREATE TABLE IF NOT EXISTS works (
            id TEXT PRIMARY KEY,
            task_id TEXT NOT NULL,
            assigned_worker TEXT,
            status TEXT NOT NULL DEFAULT 'pending',
            started_at TEXT,
            completed_at TEXT,
            result TEXT,
            FOREIGN KEY (task_id) REFERENCES tasks (id)
        )
        "#,
        &[],
    ).await.unwrap();
    
    db.execute(
        r#"
        CREATE TABLE IF NOT EXISTS workers (
            id TEXT PRIMARY KEY,
            status TEXT NOT NULL DEFAULT 'idle',
            current_work_id TEXT,
            last_activity TEXT NOT NULL,
            created_at TEXT NOT NULL
        )
        "#,
        &[],
    ).await.unwrap();
    
    db.execute(
        r#"
        CREATE TABLE IF NOT EXISTS execution_results (
            execution_id TEXT PRIMARY KEY,
            success BOOLEAN NOT NULL,
            output_data TEXT,
            error TEXT,
            logs TEXT,
            metrics TEXT,
            metadata TEXT,
            created_at TEXT NOT NULL
        )
        "#,
        &[],
    ).await.unwrap();
    
    db.execute(
        r#"
        CREATE TABLE IF NOT EXISTS metrics (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            execution_id TEXT NOT NULL,
            name TEXT NOT NULL,
            value REAL NOT NULL,
            timestamp TEXT NOT NULL,
            labels TEXT NOT NULL
        )
        "#,
        &[],
    ).await.unwrap();
    
    db.execute(
        r#"
        CREATE TABLE IF NOT EXISTS logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            execution_id TEXT NOT NULL,
            level TEXT NOT NULL,
            message TEXT NOT NULL,
            timestamp TEXT NOT NULL,
            source TEXT NOT NULL,
            metadata TEXT NOT NULL
        )
        "#,
        &[],
    ).await.unwrap();
    
    db.execute(
        r#"
        CREATE TABLE IF NOT EXISTS executions (
            execution_id TEXT PRIMARY KEY,
            tool_id TEXT NOT NULL,
            status TEXT NOT NULL,
            created_at TEXT NOT NULL,
            started_at TEXT,
            completed_at TEXT,
            user_id TEXT NOT NULL,
            tenant_id TEXT NOT NULL
        )
        "#,
        &[],
    ).await.unwrap();
    
    db
}

/// Create test registry with sample tools
pub async fn setup_test_registry(db: Arc<SqliteDatabase>) -> Arc<RegistryImpl> {
    let registry = Arc::new(RegistryImpl::new(db.clone()).await.unwrap());
    
    // Register sample tools
    let sample_tools = create_sample_tools();
    for tool in sample_tools {
        registry.register_tool(tool).await.unwrap();
    }
    
    registry
}

/// Create sample tools for testing
pub fn create_sample_tools() -> Vec<ToolInfo> {
    vec![
        ToolInfo {
            id: ToolId::from_string("test-tool-1".to_string()),
            name: "Test Tool 1".to_string(),
            description: "A simple test tool".to_string(),
            version: ToolVersion::new(1, 0, 0),
            tool_type: ToolType::Python,
            status: ToolStatus::Active,
            author: "test-author".to_string(),
            repository: Some("https://github.com/test/tool1".to_string()),
            documentation: Some("https://docs.test.com/tool1".to_string()),
            tags: vec!["test".to_string(), "utility".to_string()],
            capabilities: vec!["process".to_string(), "transform".to_string()],
            configuration_schema: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "input": {"type": "string"},
                    "output_format": {"type": "string", "enum": ["json", "text"]}
                }
            })),
            examples: vec![
                ToolExample {
                    name: "Basic usage".to_string(),
                    description: "Basic tool usage example".to_string(),
                    input: serde_json::json!({"input": "test data"}),
                    output: serde_json::json!({"result": "processed data"}),
                }
            ],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
        ToolInfo {
            id: ToolId::from_string("test-tool-2".to_string()),
            name: "Test Tool 2".to_string(),
            description: "Another test tool for complex operations".to_string(),
            version: ToolVersion::new(2, 1, 0),
            tool_type: ToolType::Shell,
            status: ToolStatus::Active,
            author: "test-author".to_string(),
            repository: None,
            documentation: None,
            tags: vec!["test".to_string(), "complex".to_string()],
            capabilities: vec!["analyze".to_string(), "report".to_string()],
            configuration_schema: None,
            examples: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
        ToolInfo {
            id: ToolId::from_string("slow-tool".to_string()),
            name: "Slow Tool".to_string(),
            description: "A tool that takes time to execute".to_string(),
            version: ToolVersion::new(1, 0, 0),
            tool_type: ToolType::System,
            status: ToolStatus::Active,
            author: "test-author".to_string(),
            repository: None,
            documentation: None,
            tags: vec!["test".to_string(), "slow".to_string()],
            capabilities: vec!["wait".to_string()],
            configuration_schema: None,
            examples: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
    ]
}

/// Create test execution request
pub fn create_test_execution_request(tool_id: &str) -> ExecutionRequest {
    ExecutionRequest {
        tool_id: ToolId::from_string(tool_id.to_string()),
        version: None,
        parameters: HashMap::from([
            ("input".to_string(), serde_json::Value::String("test input".to_string())),
            ("format".to_string(), serde_json::Value::String("json".to_string())),
        ]),
        context: create_test_execution_context(),
        options: create_test_execution_options(),
    }
}

/// Create test execution context
pub fn create_test_execution_context() -> ExecutionContext {
    ExecutionContext {
        user_id: "test-user-123".to_string(),
        tenant_id: "test-tenant-456".to_string(),
        session_id: "test-session-789".to_string(),
        request_id: "test-request-abc".to_string(),
        parent_execution_id: None,
        environment: HashMap::from([
            ("ENV_VAR_1".to_string(), "value1".to_string()),
            ("ENV_VAR_2".to_string(), "value2".to_string()),
        ]),
    }
}

/// Create test execution options
pub fn create_test_execution_options() -> ExecutionOptions {
    ExecutionOptions {
        timeout: Some(Duration::from_secs(30)),
        retry_count: 2,
        retry_delay: Duration::from_millis(100),
        priority: Priority::Normal,
        resource_limits: ResourceLimits {
            memory_limit: Some(512 * 1024 * 1024), // 512MB
            cpu_limit: Some(1.0),
            execution_time_limit: Some(Duration::from_secs(60)),
            network_limit: Some(10 * 1024 * 1024), // 10MB
        },
        logging_level: LogLevel::Info,
    }
}

/// Create test executor with all components
pub async fn create_test_executor() -> ExecutorResult<ExecutorImpl> {
    let db = setup_test_database().await;
    let registry = setup_test_registry(db.clone()).await;
    
    create_default_executor(db, registry)
}

/// Create test executor with custom configuration
pub async fn create_test_executor_with_config(
    scheduler_config: Option<SchedulerConfig>,
    worker_pool_config: Option<WorkerPoolConfig>,
) -> ExecutorResult<ExecutorImpl> {
    let db = setup_test_database().await;
    let registry = setup_test_registry(db.clone()).await;
    
    create_executor(db, registry, scheduler_config, worker_pool_config)
}

/// Wait for async operation with timeout
pub async fn wait_for_condition<F, Fut>(
    condition: F,
    timeout: Duration,
    check_interval: Duration,
) -> bool
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let start = std::time::Instant::now();
    
    while start.elapsed() < timeout {
        if condition().await {
            return true;
        }
        tokio::time::sleep(check_interval).await;
    }
    
    false
}

/// Assert execution result is successful
pub fn assert_execution_success(result: &ExecutionResult) {
    assert!(result.success, "Execution should be successful");
    assert!(result.error.is_none(), "Error should be None for successful execution");
    assert!(result.output.is_some(), "Output should be present for successful execution");
}

/// Assert execution result has failed
pub fn assert_execution_failure(result: &ExecutionResult) {
    assert!(!result.success, "Execution should have failed");
    assert!(result.error.is_some(), "Error should be present for failed execution");
}

/// Create test metrics
pub fn create_test_metrics() -> Vec<Metric> {
    vec![
        Metric {
            name: "execution_duration".to_string(),
            value: 1.5,
            labels: HashMap::from([
                ("tool_id".to_string(), "test-tool-1".to_string()),
                ("status".to_string(), "success".to_string()),
            ]),
            timestamp: Utc::now(),
        },
        Metric {
            name: "memory_usage".to_string(),
            value: 1024.0,
            labels: HashMap::from([
                ("tool_id".to_string(), "test-tool-1".to_string()),
                ("unit".to_string(), "bytes".to_string()),
            ]),
            timestamp: Utc::now(),
        },
    ]
}

/// Create test log entries
pub fn create_test_logs() -> Vec<LogEntry> {
    vec![
        LogEntry {
            level: LogLevel::Info,
            message: "Execution started".to_string(),
            timestamp: Utc::now(),
            source: "executor".to_string(),
            metadata: HashMap::from([
                ("tool_id".to_string(), serde_json::Value::String("test-tool-1".to_string())),
            ]),
        },
        LogEntry {
            level: LogLevel::Debug,
            message: "Processing input data".to_string(),
            timestamp: Utc::now(),
            source: "tool".to_string(),
            metadata: HashMap::new(),
        },
        LogEntry {
            level: LogLevel::Info,
            message: "Execution completed successfully".to_string(),
            timestamp: Utc::now(),
            source: "executor".to_string(),
            metadata: HashMap::new(),
        },
    ]
}

/// Performance test configuration
pub struct PerformanceConfig {
    pub concurrent_executions: usize,
    pub total_executions: usize,
    pub max_duration: Duration,
    pub expected_throughput: f64, // executions per second
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            concurrent_executions: 10,
            total_executions: 100,
            max_duration: Duration::from_secs(30),
            expected_throughput: 10.0,
        }
    }
}

/// Benchmark execution performance (simplified version without lifetime issues)
pub async fn benchmark_execution(
    config: PerformanceConfig,
) -> Result<PerformanceBenchmark, ExecutorError> {
    let executor = create_test_executor().await?;
    let start_time = std::time::Instant::now();
    let mut handles = Vec::new();
    
    // Create semaphore to limit concurrent executions
    let semaphore = Arc::new(tokio::sync::Semaphore::new(config.concurrent_executions));
    
    for i in 0..config.total_executions {
        let semaphore = semaphore.clone();
        let tool_id = if i % 3 == 0 { "test-tool-1" } else if i % 3 == 1 { "test-tool-2" } else { "slow-tool" };
        
        let handle = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            let executor = create_test_executor().await.unwrap();
            let request = create_test_execution_request(tool_id);
            let start = std::time::Instant::now();
            
            match executor.execute_tool(request).await {
                Ok(_) => Some(start.elapsed()),
                Err(_) => None,
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all executions to complete
    let mut successful_executions = 0;
    let mut total_duration = Duration::from_secs(0);
    let mut min_duration = Duration::from_secs(u64::MAX);
    let mut max_duration = Duration::from_secs(0);
    
    for handle in handles {
        if let Ok(Some(duration)) = handle.await {
            successful_executions += 1;
            total_duration += duration;
            min_duration = min_duration.min(duration);
            max_duration = max_duration.max(duration);
        }
    }
    
    let total_elapsed = start_time.elapsed();
    let throughput = successful_executions as f64 / total_elapsed.as_secs_f64();
    let average_duration = if successful_executions > 0 {
        total_duration / successful_executions as u32
    } else {
        Duration::from_secs(0)
    };
    
    Ok(PerformanceBenchmark {
        total_executions: config.total_executions,
        successful_executions,
        failed_executions: config.total_executions - successful_executions,
        total_duration: total_elapsed,
        average_duration,
        min_duration: if min_duration == Duration::from_secs(u64::MAX) { Duration::from_secs(0) } else { min_duration },
        max_duration,
        throughput,
        concurrent_executions: config.concurrent_executions,
    })
}

/// Performance benchmark results
#[derive(Debug, Clone)]
pub struct PerformanceBenchmark {
    pub total_executions: usize,
    pub successful_executions: usize,
    pub failed_executions: usize,
    pub total_duration: Duration,
    pub average_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub throughput: f64,
    pub concurrent_executions: usize,
}

impl PerformanceBenchmark {
    pub fn print_summary(&self) {
        println!("\n=== Performance Benchmark Results ===");
        println!("Total Executions: {}", self.total_executions);
        println!("Successful: {}", self.successful_executions);
        println!("Failed: {}", self.failed_executions);
        println!("Success Rate: {:.2}%", 
            (self.successful_executions as f64 / self.total_executions as f64) * 100.0);
        println!("Total Duration: {:?}", self.total_duration);
        println!("Average Duration: {:?}", self.average_duration);
        println!("Min Duration: {:?}", self.min_duration);
        println!("Max Duration: {:?}", self.max_duration);
        println!("Throughput: {:.2} executions/second", self.throughput);
        println!("Concurrent Executions: {}", self.concurrent_executions);
        println!("=====================================\n");
    }
} 