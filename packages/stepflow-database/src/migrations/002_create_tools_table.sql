-- Create tools table
-- This migration creates the tools table for storing tool information

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
    tags TEXT, -- JSON array
    capabilities TEXT, -- JSON array
    configuration_schema TEXT, -- JSON object
    examples TEXT, -- JSON array
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create index on tool type and status for efficient queries
CREATE INDEX IF NOT EXISTS idx_tools_type_status ON tools(tool_type, status);
CREATE INDEX IF NOT EXISTS idx_tools_author ON tools(author);
CREATE INDEX IF NOT EXISTS idx_tools_created_at ON tools(created_at); 