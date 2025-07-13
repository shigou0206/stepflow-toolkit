//! Execution context and data structures

use std::collections::HashMap;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use stepflow_core::*;

// 添加缺失的ID类型定义
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(String);

impl TaskId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
    
    pub fn from_string(s: String) -> Self {
        Self(s)
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkId(String);

impl WorkId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
    
    pub fn from_string(s: String) -> Self {
        Self(s)
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for WorkId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkerId(String);

impl WorkerId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
    
    pub fn from_string(s: String) -> Self {
        Self(s)
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for WorkerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Execution request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRequest {
    pub tool_id: ToolId,
    pub version: Option<ToolVersion>,
    pub parameters: HashMap<String, serde_json::Value>,
    pub context: ExecutionContext,
    pub options: ExecutionOptions,
}

/// Execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub user_id: String,
    pub tenant_id: String,
    pub session_id: String,
    pub request_id: String,
    pub parent_execution_id: Option<ExecutionId>,
    pub environment: HashMap<String, String>,
}

/// Execution options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionOptions {
    pub timeout: Option<Duration>,
    pub retry_count: u32,
    pub retry_delay: Duration,
    pub priority: Priority,
    pub resource_limits: ResourceLimits,
    pub logging_level: LogLevel,
}

/// Execution output (不与stepflow_core冲突的自定义类型)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionOutput {
    pub data: serde_json::Value,
    pub logs: Vec<LogEntry>,
    pub metrics: Vec<MetricEntry>,
}

/// Execution metadata (不与stepflow_core冲突的自定义类型)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetadata {
    pub tool_info: ToolInfo,
    pub execution_environment: HashMap<String, String>,
    pub resource_usage: ResourceUsage,
}

/// Execution timing (不与stepflow_core冲突的自定义类型)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTiming {
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration: Option<Duration>,
    pub queue_time: Option<Duration>,
    pub execution_time: Option<Duration>,
}

/// Priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

/// Resource limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub memory_limit: Option<u64>,
    pub cpu_limit: Option<f64>,
    pub execution_time_limit: Option<Duration>,
    pub network_limit: Option<u64>,
}

/// Resource usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub memory_used: u64,
    pub cpu_used: f64,
    pub execution_time: Duration,
    pub network_used: u64,
}

/// Metric entry (不与stepflow_core::Metric冲突的自定义类型)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricEntry {
    pub name: String,
    pub value: f64,
    pub unit: String,
    pub timestamp: DateTime<Utc>,
    pub labels: HashMap<String, String>,
}

/// Task definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub execution_request: ExecutionRequest,
    pub priority: Priority,
    pub created_at: DateTime<Utc>,
    pub scheduled_at: Option<DateTime<Utc>>,
}

/// Task status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Work unit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Work {
    pub id: WorkId,
    pub task: Task,
    pub assigned_worker: Option<WorkerId>,
    pub started_at: Option<DateTime<Utc>>,
}

/// Work status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkStatus {
    Pending,
    Assigned,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Queue status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStatus {
    pub pending_tasks: usize,
    pub running_tasks: usize,
    pub completed_tasks: usize,
    pub failed_tasks: usize,
    pub total_capacity: usize,
}

/// Pool status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStatus {
    pub active_workers: usize,
    pub idle_workers: usize,
    pub total_workers: usize,
    pub pending_work: usize,
    pub completed_work: usize,
}

/// Execution information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionInfo {
    pub execution_id: ExecutionId,
    pub tool_id: ToolId,
    pub status: ExecutionStatus,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub user_id: String,
    pub tenant_id: String,
}

// 为了兼容性，重新导出stepflow_core的类型
pub use stepflow_core::{LogEntry, LogLevel, Metric, MetricFilter, ExecutionFilter};

// 实现默认值
impl Default for Priority {
    fn default() -> Self {
        Priority::Normal
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            memory_limit: None,
            cpu_limit: None,
            execution_time_limit: None,
            network_limit: None,
        }
    }
}

impl Default for ExecutionOptions {
    fn default() -> Self {
        Self {
            timeout: Some(Duration::from_secs(300)),
            retry_count: 3,
            retry_delay: Duration::from_secs(1),
            priority: Priority::default(),
            resource_limits: ResourceLimits::default(),
            logging_level: LogLevel::Info,
        }
    }
} 