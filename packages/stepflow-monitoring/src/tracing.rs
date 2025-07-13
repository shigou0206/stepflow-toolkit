//! Tracing for Stepflow Tool System

use stepflow_core::{LogEntry, LogLevel};
use chrono::Utc;

/// Tracing collector
pub struct TracingCollector {
    entries: Vec<LogEntry>,
}

impl TracingCollector {
    /// Create a new tracing collector
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Add a log entry
    pub fn add_entry(&mut self, level: LogLevel, message: String, source: String) {
        let entry = LogEntry {
            level,
            message,
            timestamp: Utc::now(),
            source,
            metadata: std::collections::HashMap::new(),
        };
        self.entries.push(entry);
    }

    /// Get log entries
    pub fn get_entries(&self) -> &[LogEntry] {
        &self.entries
    }
} 