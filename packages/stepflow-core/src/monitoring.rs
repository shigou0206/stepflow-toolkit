//! Monitoring types and traits for Stepflow Tool System

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Metric type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Summary,
}

/// Metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub value: f64,
    pub metric_type: MetricType,
    pub labels: HashMap<String, String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub description: Option<String>,
    pub unit: Option<String>,
}

/// Metric collector trait
#[async_trait]
pub trait MetricCollector: Send + Sync {
    /// Record counter
    async fn record_counter(&self, name: &str, value: u64, labels: &HashMap<String, String>) -> Result<(), crate::StepflowError>;

    /// Record gauge
    async fn record_gauge(&self, name: &str, value: f64, labels: &HashMap<String, String>) -> Result<(), crate::StepflowError>;

    /// Record histogram
    async fn record_histogram(&self, name: &str, value: f64, labels: &HashMap<String, String>) -> Result<(), crate::StepflowError>;

    /// Record summary
    async fn record_summary(&self, name: &str, value: f64, labels: &HashMap<String, String>) -> Result<(), crate::StepflowError>;

    /// Get metric
    async fn get_metric(&self, name: &str, labels: &HashMap<String, String>) -> Result<Option<Metric>, crate::StepflowError>;

    /// List metrics
    async fn list_metrics(&self, filter: Option<MetricFilter>) -> Result<Vec<Metric>, crate::StepflowError>;

    /// Export metrics
    async fn export_metrics(&self, format: MetricFormat) -> Result<Vec<u8>, crate::StepflowError>;
}

/// Metric filter
#[derive(Debug, Clone)]
pub struct MetricFilter {
    pub name: Option<String>,
    pub metric_type: Option<MetricType>,
    pub labels: HashMap<String, String>,
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
}

/// Metric format
#[derive(Debug, Clone)]
pub enum MetricFormat {
    Prometheus,
    Json,
    Csv,
    Graphite,
}

/// Log level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "trace"),
            LogLevel::Debug => write!(f, "debug"),
            LogLevel::Info => write!(f, "info"),
            LogLevel::Warn => write!(f, "warn"),
            LogLevel::Error => write!(f, "error"),
            LogLevel::Fatal => write!(f, "fatal"),
        }
    }
}

/// Log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: String,
    pub level: LogLevel,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub source: String,
    pub target: String,
    pub module_path: Option<String>,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub thread_id: Option<u64>,
    pub thread_name: Option<String>,
    pub user_id: Option<crate::UserId>,
    pub tenant_id: Option<crate::TenantId>,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub fields: HashMap<String, serde_json::Value>,
}

/// Logger trait
#[async_trait]
pub trait Logger: Send + Sync {
    /// Log message
    async fn log(&self, level: LogLevel, message: &str, fields: &HashMap<String, serde_json::Value>) -> Result<(), crate::StepflowError>;

    /// Log with context
    async fn log_with_context(
        &self,
        level: LogLevel,
        message: &str,
        fields: &HashMap<String, serde_json::Value>,
        context: &LogContext,
    ) -> Result<(), crate::StepflowError>;

    /// Get logs
    async fn get_logs(&self, filter: Option<LogFilter>) -> Result<Vec<LogEntry>, crate::StepflowError>;

    /// Export logs
    async fn export_logs(&self, filter: Option<LogFilter>, format: LogFormat) -> Result<Vec<u8>, crate::StepflowError>;

    /// Clear logs
    async fn clear_logs(&self, older_than: Option<chrono::DateTime<chrono::Utc>>) -> Result<u64, crate::StepflowError>;
}

/// Log context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogContext {
    pub user_id: Option<crate::UserId>,
    pub tenant_id: Option<crate::TenantId>,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub request_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

/// Log filter
#[derive(Debug, Clone)]
pub struct LogFilter {
    pub level: Option<LogLevel>,
    pub source: Option<String>,
    pub target: Option<String>,
    pub user_id: Option<crate::UserId>,
    pub tenant_id: Option<crate::TenantId>,
    pub trace_id: Option<String>,
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub message_contains: Option<String>,
}

/// Log format
#[derive(Debug, Clone)]
pub enum LogFormat {
    Json,
    Csv,
    Text,
    Structured,
}

/// Trace span
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceSpan {
    pub span_id: String,
    pub trace_id: String,
    pub parent_span_id: Option<String>,
    pub name: String,
    pub kind: SpanKind,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub duration: Option<Duration>,
    pub attributes: HashMap<String, serde_json::Value>,
    pub events: Vec<SpanEvent>,
    pub links: Vec<SpanLink>,
    pub status: SpanStatus,
}

/// Span kind
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpanKind {
    Internal,
    Server,
    Client,
    Producer,
    Consumer,
}

/// Span event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanEvent {
    pub name: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub attributes: HashMap<String, serde_json::Value>,
}

/// Span link
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanLink {
    pub trace_id: String,
    pub span_id: String,
    pub attributes: HashMap<String, serde_json::Value>,
}

/// Span status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanStatus {
    pub code: SpanStatusCode,
    pub message: Option<String>,
}

/// Span status code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpanStatusCode {
    Unset,
    Ok,
    Error,
}

/// Tracer trait
#[async_trait]
pub trait Tracer: Send + Sync {
    /// Start span
    async fn start_span(&self, name: &str, attributes: &HashMap<String, serde_json::Value>) -> Result<TraceSpan, crate::StepflowError>;

    /// End span
    async fn end_span(&self, span: &TraceSpan) -> Result<(), crate::StepflowError>;

    /// Add event to span
    async fn add_event(&self, span: &TraceSpan, event: &SpanEvent) -> Result<(), crate::StepflowError>;

    /// Set span attributes
    async fn set_attributes(&self, span: &TraceSpan, attributes: &HashMap<String, serde_json::Value>) -> Result<(), crate::StepflowError>;

    /// Get trace
    async fn get_trace(&self, trace_id: &str) -> Result<Vec<TraceSpan>, crate::StepflowError>;

    /// Get traces
    async fn get_traces(&self, filter: Option<TraceFilter>) -> Result<Vec<Vec<TraceSpan>>, crate::StepflowError>;

    /// Export traces
    async fn export_traces(&self, filter: Option<TraceFilter>, format: TraceFormat) -> Result<Vec<u8>, crate::StepflowError>;
}

/// Trace filter
#[derive(Debug, Clone)]
pub struct TraceFilter {
    pub trace_id: Option<String>,
    pub span_name: Option<String>,
    pub service_name: Option<String>,
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub duration_min: Option<Duration>,
    pub duration_max: Option<Duration>,
    pub status: Option<SpanStatusCode>,
}

/// Trace format
#[derive(Debug, Clone)]
pub enum TraceFormat {
    Jaeger,
    Zipkin,
    Otlp,
    Json,
}

/// Alert severity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Critical,
    Warning,
    Info,
    Debug,
}

impl std::fmt::Display for AlertSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertSeverity::Critical => write!(f, "critical"),
            AlertSeverity::Warning => write!(f, "warning"),
            AlertSeverity::Info => write!(f, "info"),
            AlertSeverity::Debug => write!(f, "debug"),
        }
    }
}

/// Alert status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertStatus {
    Firing,
    Resolved,
    Acknowledged,
}

impl std::fmt::Display for AlertStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertStatus::Firing => write!(f, "firing"),
            AlertStatus::Resolved => write!(f, "resolved"),
            AlertStatus::Acknowledged => write!(f, "acknowledged"),
        }
    }
}

/// Alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub alert_id: String,
    pub name: String,
    pub description: String,
    pub severity: AlertSeverity,
    pub status: AlertStatus,
    pub labels: HashMap<String, String>,
    pub annotations: HashMap<String, String>,
    pub value: Option<f64>,
    pub fired_at: chrono::DateTime<chrono::Utc>,
    pub resolved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub acknowledged_at: Option<chrono::DateTime<chrono::Utc>>,
    pub acknowledged_by: Option<crate::UserId>,
    pub resolved_by: Option<crate::UserId>,
}

/// Alert manager trait
#[async_trait]
pub trait AlertManager: Send + Sync {
    /// Create alert
    async fn create_alert(&self, alert: &Alert) -> Result<String, crate::StepflowError>;

    /// Get alert
    async fn get_alert(&self, alert_id: &str) -> Result<Alert, crate::StepflowError>;

    /// Update alert
    async fn update_alert(&self, alert_id: &str, alert: &Alert) -> Result<(), crate::StepflowError>;

    /// Delete alert
    async fn delete_alert(&self, alert_id: &str) -> Result<(), crate::StepflowError>;

    /// List alerts
    async fn list_alerts(&self, filter: Option<AlertFilter>) -> Result<Vec<Alert>, crate::StepflowError>;

    /// Acknowledge alert
    async fn acknowledge_alert(&self, alert_id: &str, user_id: &crate::UserId) -> Result<(), crate::StepflowError>;

    /// Resolve alert
    async fn resolve_alert(&self, alert_id: &str, user_id: &crate::UserId) -> Result<(), crate::StepflowError>;
}

/// Alert filter
#[derive(Debug, Clone)]
pub struct AlertFilter {
    pub severity: Option<AlertSeverity>,
    pub status: Option<AlertStatus>,
    pub labels: HashMap<String, String>,
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
}

/// Health check status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "healthy"),
            HealthStatus::Degraded => write!(f, "degraded"),
            HealthStatus::Unhealthy => write!(f, "unhealthy"),
            HealthStatus::Unknown => write!(f, "unknown"),
        }
    }
}

/// Health check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub name: String,
    pub status: HealthStatus,
    pub message: Option<String>,
    pub details: HashMap<String, serde_json::Value>,
    pub last_check: chrono::DateTime<chrono::Utc>,
    pub response_time: Duration,
}

/// Health checker trait
#[async_trait]
pub trait HealthChecker: Send + Sync {
    /// Check health
    async fn check_health(&self, component: &str) -> Result<HealthCheck, crate::StepflowError>;

    /// Check all health
    async fn check_all_health(&self) -> Result<Vec<HealthCheck>, crate::StepflowError>;

    /// Register health check
    async fn register_health_check(&self, name: &str, check: Box<dyn HealthCheckProvider>) -> Result<(), crate::StepflowError>;

    /// Unregister health check
    async fn unregister_health_check(&self, name: &str) -> Result<(), crate::StepflowError>;

    /// Get health status
    async fn get_health_status(&self) -> Result<HealthStatus, crate::StepflowError>;
}

/// Health check provider trait
#[async_trait]
pub trait HealthCheckProvider: Send + Sync {
    /// Check health
    async fn check(&self) -> Result<HealthCheck, crate::StepflowError>;
}

/// Profiler trait
#[async_trait]
pub trait Profiler: Send + Sync {
    /// Start profiling
    async fn start_profiling(&self, name: &str) -> Result<String, crate::StepflowError>;

    /// Stop profiling
    async fn stop_profiling(&self, profile_id: &str) -> Result<ProfileResult, crate::StepflowError>;

    /// Get profile
    async fn get_profile(&self, profile_id: &str) -> Result<ProfileResult, crate::StepflowError>;

    /// List profiles
    async fn list_profiles(&self, filter: Option<ProfileFilter>) -> Result<Vec<ProfileResult>, crate::StepflowError>;

    /// Export profile
    async fn export_profile(&self, profile_id: &str, format: ProfileFormat) -> Result<Vec<u8>, crate::StepflowError>;
}

/// Profile result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileResult {
    pub profile_id: String,
    pub name: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: chrono::DateTime<chrono::Utc>,
    pub duration: Duration,
    pub data: HashMap<String, serde_json::Value>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Profile filter
#[derive(Debug, Clone)]
pub struct ProfileFilter {
    pub name: Option<String>,
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub duration_min: Option<Duration>,
    pub duration_max: Option<Duration>,
}

/// Profile format
#[derive(Debug, Clone)]
pub enum ProfileFormat {
    Pprof,
    Json,
    Csv,
    Flamegraph,
} 