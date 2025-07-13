//! Database migrations

use stepflow_core::{Migration, StepflowResult, Database};
use sqlx::SqlitePool;
use crate::SqliteDatabase;
use tracing::{debug, info, error};

/// Migration status
#[derive(Debug, Clone, PartialEq)]
pub enum MigrationStatus {
    Pending,
    Running,
    Completed,
    Failed(String),
}

/// Migration record
#[derive(Debug, Clone)]
pub struct MigrationRecord {
    pub version: u32,
    pub name: String,
    pub applied_at: String,
}

/// Migration manager for SQLite database
pub struct MigrationManager {
    pool: SqlitePool,
}

impl MigrationManager {
    /// Create a new migration manager
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Run migrations
    pub async fn run_migrations(database: &SqliteDatabase) -> StepflowResult<()> {
        info!("Starting database migrations");
        
        // First, ensure migrations table exists
        Self::ensure_migrations_table(database).await?;
        
        let migrations = Self::get_migrations();
        let applied_migrations = Self::get_applied_migrations(database).await?;
        
        let mut applied_count = 0;
        for migration in migrations {
            if !applied_migrations.contains(&migration.version) {
                info!("Applying migration: {} (version {})", migration.name, migration.version);
                
                // Record migration start
                Self::record_migration_start(database, &migration).await?;
                
                // Apply migration
                match database.migrate(&[migration.clone()]).await {
                    Ok(_) => {
                        // Record successful migration
                        Self::record_migration_success(database, &migration).await?;
                        applied_count += 1;
                        info!("Successfully applied migration: {}", migration.name);
                    }
                    Err(e) => {
                        // Record failed migration
                        Self::record_migration_failure(database, &migration, &e.to_string()).await?;
                        error!("Failed to apply migration {}: {}", migration.name, e);
                        return Err(e);
                    }
                }
            } else {
                debug!("Migration {} (version {}) already applied", migration.name, migration.version);
            }
        }
        
        if applied_count > 0 {
            info!("Applied {} new migrations", applied_count);
        } else {
            info!("No new migrations to apply");
        }
        
        Ok(())
    }

    /// Check if migrations are needed
    pub async fn check_migrations_needed(database: &SqliteDatabase) -> StepflowResult<bool> {
        let migrations = Self::get_migrations();
        let applied_migrations = Self::get_applied_migrations(database).await?;
        
        let pending_count = migrations.iter()
            .filter(|m| !applied_migrations.contains(&m.version))
            .count();
        
        Ok(pending_count > 0)
    }

    /// Get migration status
    pub async fn get_migration_status(database: &SqliteDatabase) -> StepflowResult<MigrationStatus> {
        let migrations = Self::get_migrations();
        let applied_migrations = Self::get_applied_migrations(database).await?;
        
        let pending_count = migrations.iter()
            .filter(|m| !applied_migrations.contains(&m.version))
            .count();
        
        if pending_count == 0 {
            Ok(MigrationStatus::Completed)
        } else {
            Ok(MigrationStatus::Pending)
        }
    }

    /// Get migration history
    pub async fn get_migration_history(database: &SqliteDatabase) -> StepflowResult<Vec<MigrationRecord>> {
        let sql = "SELECT version, name, applied_at FROM migrations ORDER BY version";
        let result = database.execute(sql, &[]).await?;
        
        let mut records = Vec::new();
        for row in result.rows {
            let version = row.get("version")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as u32;
            
            let name = row.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            
            let applied_at = row.get("applied_at")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            
            records.push(MigrationRecord {
                version,
                name,
                applied_at,
            });
        }
        
        Ok(records)
    }

    /// Ensure migrations table exists
    async fn ensure_migrations_table(database: &SqliteDatabase) -> StepflowResult<()> {
        let sql = r#"
            CREATE TABLE IF NOT EXISTS migrations (
                version INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                applied_at TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'completed',
                error_message TEXT
            )
        "#;
        
        database.execute(sql, &[]).await?;
        Ok(())
    }

    /// Get applied migrations
    async fn get_applied_migrations(database: &SqliteDatabase) -> StepflowResult<Vec<u32>> {
        let sql = "SELECT version FROM migrations WHERE status = 'completed' ORDER BY version";
        let result = database.execute(sql, &[]).await?;
        
        let mut versions = Vec::new();
        for row in result.rows {
            if let Some(version_value) = row.get("version") {
                if let Some(version) = version_value.as_i64() {
                    versions.push(version as u32);
                }
            }
        }
        
        Ok(versions)
    }

    /// Record migration start
    async fn record_migration_start(database: &SqliteDatabase, migration: &Migration) -> StepflowResult<()> {
        let sql = r#"
            INSERT OR REPLACE INTO migrations (version, name, applied_at, status)
            VALUES (?, ?, datetime('now'), 'running')
        "#;
        
        let params = vec![
            serde_json::json!(migration.version),
            serde_json::json!(migration.name),
        ];
        
        database.execute(sql, &params).await?;
        Ok(())
    }

    /// Record successful migration
    async fn record_migration_success(database: &SqliteDatabase, migration: &Migration) -> StepflowResult<()> {
        let sql = r#"
            UPDATE migrations 
            SET status = 'completed', applied_at = datetime('now')
            WHERE version = ?
        "#;
        
        let params = vec![serde_json::json!(migration.version)];
        database.execute(sql, &params).await?;
        Ok(())
    }

    /// Record failed migration
    async fn record_migration_failure(database: &SqliteDatabase, migration: &Migration, error: &str) -> StepflowResult<()> {
        let sql = r#"
            UPDATE migrations 
            SET status = 'failed', error_message = ?
            WHERE version = ?
        "#;
        
        let params = vec![
            serde_json::json!(error),
            serde_json::json!(migration.version),
        ];
        
        database.execute(sql, &params).await?;
        Ok(())
    }

    /// Get all migrations
    fn get_migrations() -> Vec<Migration> {
        vec![
            Migration {
                version: 1,
                name: "create_tools_table".to_string(),
                sql: r#"
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
                        status TEXT NOT NULL,
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
                "#.to_string(),
            },
            Migration {
                version: 2,
                name: "create_tenants_table".to_string(),
                sql: r#"
                    CREATE TABLE IF NOT EXISTS tenants (
                        id TEXT PRIMARY KEY,
                        name TEXT NOT NULL,
                        description TEXT,
                        domain TEXT,
                        settings TEXT,
                        created_at TEXT NOT NULL,
                        updated_at TEXT NOT NULL
                    )
                "#.to_string(),
            },
            Migration {
                version: 3,
                name: "create_users_table".to_string(),
                sql: r#"
                    CREATE TABLE IF NOT EXISTS users (
                        id TEXT PRIMARY KEY,
                        username TEXT NOT NULL UNIQUE,
                        email TEXT NOT NULL UNIQUE,
                        password_hash TEXT NOT NULL,
                        role TEXT NOT NULL,
                        tenant_id TEXT NOT NULL,
                        settings TEXT,
                        created_at TEXT NOT NULL,
                        updated_at TEXT NOT NULL,
                        FOREIGN KEY (tenant_id) REFERENCES tenants (id)
                    )
                "#.to_string(),
            },
            Migration {
                version: 4,
                name: "create_executions_table".to_string(),
                sql: r#"
                    CREATE TABLE IF NOT EXISTS executions (
                        id TEXT PRIMARY KEY,
                        tool_id TEXT NOT NULL,
                        tenant_id TEXT NOT NULL,
                        user_id TEXT NOT NULL,
                        status TEXT NOT NULL,
                        request TEXT NOT NULL,
                        result TEXT,
                        started_at TEXT NOT NULL,
                        completed_at TEXT,
                        created_at TEXT NOT NULL,
                        updated_at TEXT NOT NULL,
                        FOREIGN KEY (tool_id) REFERENCES tools (id),
                        FOREIGN KEY (tenant_id) REFERENCES tenants (id),
                        FOREIGN KEY (user_id) REFERENCES users (id)
                    )
                "#.to_string(),
            },
            Migration {
                version: 5,
                name: "create_migrations_table".to_string(),
                sql: r#"
                    CREATE TABLE IF NOT EXISTS migrations (
                        version INTEGER PRIMARY KEY,
                        name TEXT NOT NULL,
                        applied_at TEXT NOT NULL,
                        status TEXT NOT NULL DEFAULT 'completed',
                        error_message TEXT
                    )
                "#.to_string(),
            },
            Migration {
                version: 6,
                name: "create_metrics_table".to_string(),
                sql: r#"
                    CREATE TABLE IF NOT EXISTS metrics (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        execution_id TEXT NOT NULL,
                        name TEXT NOT NULL,
                        value REAL NOT NULL,
                        timestamp TEXT NOT NULL,
                        labels TEXT NOT NULL
                    );
                    CREATE INDEX IF NOT EXISTS idx_metrics_execution_id ON metrics(execution_id);
                    CREATE INDEX IF NOT EXISTS idx_metrics_name ON metrics(name);
                    CREATE INDEX IF NOT EXISTS idx_metrics_timestamp ON metrics(timestamp);
                "#.to_string(),
            },
            Migration {
                version: 7,
                name: "create_logs_table".to_string(),
                sql: r#"
                    CREATE TABLE IF NOT EXISTS logs (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        execution_id TEXT NOT NULL,
                        level TEXT NOT NULL,
                        message TEXT NOT NULL,
                        timestamp TEXT NOT NULL,
                        source TEXT NOT NULL,
                        metadata TEXT NOT NULL
                    );
                    CREATE INDEX IF NOT EXISTS idx_logs_execution_id ON logs(execution_id);
                    CREATE INDEX IF NOT EXISTS idx_logs_level ON logs(level);
                    CREATE INDEX IF NOT EXISTS idx_logs_timestamp ON logs(timestamp);
                "#.to_string(),
            },
            Migration {
                version: 8,
                name: "create_executor_tables".to_string(),
                sql: r#"
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
                    );
                    
                    CREATE TABLE IF NOT EXISTS works (
                        id TEXT PRIMARY KEY,
                        task_id TEXT NOT NULL,
                        assigned_worker TEXT,
                        status TEXT NOT NULL DEFAULT 'pending',
                        started_at TEXT,
                        completed_at TEXT,
                        result TEXT,
                        FOREIGN KEY (task_id) REFERENCES tasks (id)
                    );
                    
                    CREATE TABLE IF NOT EXISTS workers (
                        id TEXT PRIMARY KEY,
                        status TEXT NOT NULL DEFAULT 'idle',
                        current_work_id TEXT,
                        last_activity TEXT NOT NULL,
                        created_at TEXT NOT NULL
                    );
                    
                    CREATE TABLE IF NOT EXISTS execution_results (
                        execution_id TEXT PRIMARY KEY,
                        success BOOLEAN NOT NULL,
                        output_data TEXT,
                        error TEXT,
                        logs TEXT,
                        metrics TEXT,
                        metadata TEXT,
                        created_at TEXT NOT NULL
                    );
                    
                    CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
                    CREATE INDEX IF NOT EXISTS idx_tasks_priority ON tasks(priority);
                    CREATE INDEX IF NOT EXISTS idx_works_task_id ON works(task_id);
                    CREATE INDEX IF NOT EXISTS idx_works_status ON works(status);
                    CREATE INDEX IF NOT EXISTS idx_workers_status ON workers(status);
                "#.to_string(),
            },
            Migration {
                version: 9,
                name: "add_indexes".to_string(),
                sql: r#"
                    CREATE INDEX IF NOT EXISTS idx_tools_status ON tools(status);
                    CREATE INDEX IF NOT EXISTS idx_tools_type ON tools(tool_type);
                    CREATE INDEX IF NOT EXISTS idx_executions_tool_id ON executions(tool_id);
                    CREATE INDEX IF NOT EXISTS idx_executions_tenant_id ON executions(tenant_id);
                    CREATE INDEX IF NOT EXISTS idx_executions_user_id ON executions(user_id);
                    CREATE INDEX IF NOT EXISTS idx_executions_status ON executions(status);
                    CREATE INDEX IF NOT EXISTS idx_executions_started_at ON executions(started_at);
                "#.to_string(),
            },
            Migration {
                version: 10,
                name: "fix_tasks_table_schema".to_string(),
                sql: r#"
                    -- Drop and recreate tasks table with correct schema
                    DROP TABLE IF EXISTS tasks;
                    CREATE TABLE tasks (
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
                    );
                    
                    -- Also fix works table to ensure it has proper foreign key
                    DROP TABLE IF EXISTS works;
                    CREATE TABLE works (
                        id TEXT PRIMARY KEY,
                        task_id TEXT NOT NULL,
                        assigned_worker TEXT,
                        status TEXT NOT NULL DEFAULT 'pending',
                        started_at TEXT,
                        completed_at TEXT,
                        result TEXT,
                        FOREIGN KEY (task_id) REFERENCES tasks (id)
                    );
                "#.to_string(),
            },
            Migration {
                version: 11,
                name: "create_sandbox_tables".to_string(),
                sql: r#"
                    CREATE TABLE IF NOT EXISTS sandboxes (
                        id TEXT PRIMARY KEY,
                        name TEXT NOT NULL,
                        isolation_type TEXT NOT NULL,
                        status TEXT NOT NULL,
                        config TEXT NOT NULL, -- JSON
                        created_at TEXT NOT NULL,
                        destroyed_at TEXT,
                        created_by TEXT NOT NULL,
                        tenant_id TEXT NOT NULL
                    );
                    
                    CREATE TABLE IF NOT EXISTS sandbox_executions (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        sandbox_id TEXT NOT NULL,
                        execution_id TEXT NOT NULL,
                        command TEXT NOT NULL,
                        status TEXT NOT NULL,
                        start_time TEXT NOT NULL,
                        end_time TEXT,
                        exit_code INTEGER,
                        output TEXT,
                        error_message TEXT,
                        resource_usage TEXT, -- JSON
                        FOREIGN KEY (sandbox_id) REFERENCES sandboxes(id) ON DELETE CASCADE
                    );
                    
                    CREATE TABLE IF NOT EXISTS sandbox_metrics (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        sandbox_id TEXT NOT NULL,
                        metric_name TEXT NOT NULL,
                        metric_value REAL NOT NULL,
                        metric_unit TEXT,
                        timestamp TEXT NOT NULL,
                        FOREIGN KEY (sandbox_id) REFERENCES sandboxes(id) ON DELETE CASCADE
                    );
                    
                    CREATE TABLE IF NOT EXISTS sandbox_violations (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        sandbox_id TEXT NOT NULL,
                        violation_type TEXT NOT NULL,
                        description TEXT NOT NULL,
                        severity TEXT NOT NULL,
                        timestamp TEXT NOT NULL,
                        details TEXT, -- JSON
                        FOREIGN KEY (sandbox_id) REFERENCES sandboxes(id) ON DELETE CASCADE
                    );
                    
                    CREATE TABLE IF NOT EXISTS sandbox_containers (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        sandbox_id TEXT NOT NULL,
                        container_id TEXT NOT NULL,
                        image TEXT NOT NULL,
                        status TEXT NOT NULL,
                        created_at TEXT NOT NULL,
                        started_at TEXT,
                        finished_at TEXT,
                        FOREIGN KEY (sandbox_id) REFERENCES sandboxes(id) ON DELETE CASCADE
                    );
                    
                    -- Create indexes for efficient queries
                    CREATE INDEX IF NOT EXISTS idx_sandboxes_status ON sandboxes(status);
                    CREATE INDEX IF NOT EXISTS idx_sandboxes_tenant_id ON sandboxes(tenant_id);
                    CREATE INDEX IF NOT EXISTS idx_sandboxes_created_by ON sandboxes(created_by);
                    CREATE INDEX IF NOT EXISTS idx_sandboxes_isolation_type ON sandboxes(isolation_type);
                    CREATE INDEX IF NOT EXISTS idx_sandbox_executions_sandbox_id ON sandbox_executions(sandbox_id);
                    CREATE INDEX IF NOT EXISTS idx_sandbox_executions_status ON sandbox_executions(status);
                    CREATE INDEX IF NOT EXISTS idx_sandbox_metrics_sandbox_id ON sandbox_metrics(sandbox_id);
                    CREATE INDEX IF NOT EXISTS idx_sandbox_metrics_timestamp ON sandbox_metrics(timestamp);
                    CREATE INDEX IF NOT EXISTS idx_sandbox_violations_sandbox_id ON sandbox_violations(sandbox_id);
                    CREATE INDEX IF NOT EXISTS idx_sandbox_violations_severity ON sandbox_violations(severity);
                    CREATE INDEX IF NOT EXISTS idx_sandbox_containers_sandbox_id ON sandbox_containers(sandbox_id);
                    CREATE INDEX IF NOT EXISTS idx_sandbox_containers_container_id ON sandbox_containers(container_id);
                "#.to_string(),
            },
        ]
    }
} 