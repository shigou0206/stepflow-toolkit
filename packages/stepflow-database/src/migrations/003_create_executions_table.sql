-- Create executions table
-- This migration creates the executions table for storing execution information

CREATE TABLE IF NOT EXISTS executions (
    id TEXT PRIMARY KEY,
    tool_id TEXT NOT NULL,
    tenant_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    request_input TEXT, -- JSON object
    request_configuration TEXT, -- JSON object
    request_metadata TEXT, -- JSON object
    result_success BOOLEAN,
    result_output TEXT, -- JSON object
    result_error TEXT,
    result_logs TEXT, -- JSON array
    result_metrics TEXT, -- JSON object
    result_metadata TEXT, -- JSON object
    started_at TIMESTAMP,
    completed_at TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (tool_id) REFERENCES tools(id),
    FOREIGN KEY (tenant_id) REFERENCES tenants(id),
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Create indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_executions_tool_id ON executions(tool_id);
CREATE INDEX IF NOT EXISTS idx_executions_tenant_id ON executions(tenant_id);
CREATE INDEX IF NOT EXISTS idx_executions_user_id ON executions(user_id);
CREATE INDEX IF NOT EXISTS idx_executions_status ON executions(status);
CREATE INDEX IF NOT EXISTS idx_executions_created_at ON executions(created_at);
CREATE INDEX IF NOT EXISTS idx_executions_started_at ON executions(started_at); 