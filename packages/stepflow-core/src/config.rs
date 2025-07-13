//! Configuration management for Stepflow Tool System

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub security: SecurityConfig,
    pub monitoring: MonitoringConfig,
    pub tools: ToolsConfig,
    pub execution: ExecutionConfig,
    pub sandbox: SandboxConfig,
    pub api: ApiConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            database: DatabaseConfig::default(),
            security: SecurityConfig::default(),
            monitoring: MonitoringConfig::default(),
            tools: ToolsConfig::default(),
            execution: ExecutionConfig::default(),
            sandbox: SandboxConfig::default(),
            api: ApiConfig::default(),
        }
    }
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
    pub max_connections: usize,
    pub connection_timeout: Duration,
    pub keep_alive_timeout: Duration,
    pub request_timeout: Duration,
    pub enable_compression: bool,
    pub enable_logging: bool,
    pub enable_metrics: bool,
    pub enable_health_checks: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            workers: num_cpus::get(),
            max_connections: 1000,
            connection_timeout: Duration::from_secs(30),
            keep_alive_timeout: Duration::from_secs(60),
            request_timeout: Duration::from_secs(300),
            enable_compression: true,
            enable_logging: true,
            enable_metrics: true,
            enable_health_checks: true,
        }
    }
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connection_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
    pub enable_migrations: bool,
    pub enable_logging: bool,
    pub enable_metrics: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "sqlite://stepflow.db".to_string(),
            max_connections: 10,
            min_connections: 2,
            connection_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(300),
            max_lifetime: Duration::from_secs(3600),
            enable_migrations: true,
            enable_logging: true,
            enable_metrics: true,
        }
    }
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub secret_key: String,
    pub jwt_secret: String,
    pub jwt_expiration: Duration,
    pub bcrypt_cost: u32,
    pub enable_rate_limiting: bool,
    pub rate_limit_requests: u32,
    pub rate_limit_window: Duration,
    pub enable_cors: bool,
    pub cors_origins: Vec<String>,
    pub enable_csrf_protection: bool,
    pub enable_xss_protection: bool,
    pub enable_content_security_policy: bool,
    pub enable_hsts: bool,
    pub enable_secure_cookies: bool,
    pub session_timeout: Duration,
    pub max_login_attempts: u32,
    pub lockout_duration: Duration,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            secret_key: "your-secret-key-here".to_string(),
            jwt_secret: "your-jwt-secret-here".to_string(),
            jwt_expiration: Duration::from_secs(3600),
            bcrypt_cost: 12,
            enable_rate_limiting: true,
            rate_limit_requests: 100,
            rate_limit_window: Duration::from_secs(60),
            enable_cors: true,
            cors_origins: vec!["http://localhost:3000".to_string()],
            enable_csrf_protection: true,
            enable_xss_protection: true,
            enable_content_security_policy: true,
            enable_hsts: true,
            enable_secure_cookies: true,
            session_timeout: Duration::from_secs(3600),
            max_login_attempts: 5,
            lockout_duration: Duration::from_secs(900),
        }
    }
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub enable_metrics: bool,
    pub metrics_port: u16,
    pub metrics_path: String,
    pub enable_logging: bool,
    pub log_level: String,
    pub log_format: String,
    pub log_file: Option<String>,
    pub enable_tracing: bool,
    pub tracing_sampling_rate: f64,
    pub enable_health_checks: bool,
    pub health_check_interval: Duration,
    pub enable_alerting: bool,
    pub alert_webhook_url: Option<String>,
    pub enable_profiling: bool,
    pub profiling_port: u16,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enable_metrics: true,
            metrics_port: 9090,
            metrics_path: "/metrics".to_string(),
            enable_logging: true,
            log_level: "info".to_string(),
            log_format: "json".to_string(),
            log_file: None,
            enable_tracing: true,
            tracing_sampling_rate: 0.1,
            enable_health_checks: true,
            health_check_interval: Duration::from_secs(30),
            enable_alerting: false,
            alert_webhook_url: None,
            enable_profiling: false,
            profiling_port: 6060,
        }
    }
}

/// Tools configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsConfig {
    pub enable_openapi: bool,
    pub enable_asyncapi: bool,
    pub enable_python: bool,
    pub enable_shell: bool,
    pub enable_ai: bool,
    pub enable_system: bool,
    pub tool_registry_url: Option<String>,
    pub tool_cache_size: usize,
    pub tool_cache_ttl: Duration,
    pub enable_tool_validation: bool,
    pub enable_tool_testing: bool,
    pub max_tool_size: usize,
    pub allowed_tool_types: Vec<String>,
    pub blocked_tool_types: Vec<String>,
}

impl Default for ToolsConfig {
    fn default() -> Self {
        Self {
            enable_openapi: true,
            enable_asyncapi: true,
            enable_python: true,
            enable_shell: true,
            enable_ai: true,
            enable_system: true,
            tool_registry_url: None,
            tool_cache_size: 1000,
            tool_cache_ttl: Duration::from_secs(3600),
            enable_tool_validation: true,
            enable_tool_testing: true,
            max_tool_size: 10 * 1024 * 1024, // 10MB
            allowed_tool_types: vec![],
            blocked_tool_types: vec![],
        }
    }
}

/// Execution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    pub max_concurrent_executions: usize,
    pub execution_timeout: Duration,
    pub max_execution_memory: usize,
    pub max_execution_cpu: f64,
    pub enable_execution_logging: bool,
    pub enable_execution_metrics: bool,
    pub enable_execution_tracing: bool,
    pub execution_retry_count: u32,
    pub execution_retry_delay: Duration,
    pub enable_execution_caching: bool,
    pub execution_cache_size: usize,
    pub execution_cache_ttl: Duration,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            max_concurrent_executions: 100,
            execution_timeout: Duration::from_secs(300),
            max_execution_memory: 512 * 1024 * 1024, // 512MB
            max_execution_cpu: 1.0,
            enable_execution_logging: true,
            enable_execution_metrics: true,
            enable_execution_tracing: true,
            execution_retry_count: 3,
            execution_retry_delay: Duration::from_secs(1),
            enable_execution_caching: true,
            execution_cache_size: 1000,
            execution_cache_ttl: Duration::from_secs(3600),
        }
    }
}

/// Sandbox configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub enable_sandboxing: bool,
    pub sandbox_type: String,
    pub sandbox_timeout: Duration,
    pub sandbox_memory_limit: usize,
    pub sandbox_cpu_limit: f64,
    pub sandbox_disk_limit: usize,
    pub sandbox_network_enabled: bool,
    pub sandbox_allowed_ports: Vec<u16>,
    pub sandbox_allowed_hosts: Vec<String>,
    pub sandbox_allowed_paths: Vec<String>,
    pub sandbox_blocked_paths: Vec<String>,
    pub sandbox_allowed_commands: Vec<String>,
    pub sandbox_blocked_commands: Vec<String>,
    pub sandbox_enable_seccomp: bool,
    pub sandbox_enable_capabilities: bool,
    pub sandbox_enable_namespaces: bool,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            enable_sandboxing: true,
            sandbox_type: "docker".to_string(),
            sandbox_timeout: Duration::from_secs(300),
            sandbox_memory_limit: 256 * 1024 * 1024, // 256MB
            sandbox_cpu_limit: 0.5,
            sandbox_disk_limit: 100 * 1024 * 1024, // 100MB
            sandbox_network_enabled: false,
            sandbox_allowed_ports: vec![80, 443],
            sandbox_allowed_hosts: vec!["api.openai.com".to_string()],
            sandbox_allowed_paths: vec!["/tmp".to_string()],
            sandbox_blocked_paths: vec!["/etc".to_string(), "/root".to_string()],
            sandbox_allowed_commands: vec!["ls".to_string(), "cat".to_string()],
            sandbox_blocked_commands: vec!["rm".to_string(), "dd".to_string()],
            sandbox_enable_seccomp: true,
            sandbox_enable_capabilities: true,
            sandbox_enable_namespaces: true,
        }
    }
}

/// API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub enable_swagger: bool,
    pub swagger_path: String,
    pub enable_openapi: bool,
    pub openapi_path: String,
    pub enable_graphql: bool,
    pub graphql_path: String,
    pub enable_websocket: bool,
    pub websocket_path: String,
    pub enable_grpc: bool,
    pub grpc_port: u16,
    pub api_version: String,
    pub api_prefix: String,
    pub enable_api_documentation: bool,
    pub enable_api_metrics: bool,
    pub enable_api_logging: bool,
    pub enable_api_rate_limiting: bool,
    pub api_rate_limit: u32,
    pub api_rate_limit_window: Duration,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            enable_swagger: true,
            swagger_path: "/swagger".to_string(),
            enable_openapi: true,
            openapi_path: "/openapi".to_string(),
            enable_graphql: false,
            graphql_path: "/graphql".to_string(),
            enable_websocket: false,
            websocket_path: "/ws".to_string(),
            enable_grpc: false,
            grpc_port: 9090,
            api_version: "v1".to_string(),
            api_prefix: "/api".to_string(),
            enable_api_documentation: true,
            enable_api_metrics: true,
            enable_api_logging: true,
            enable_api_rate_limiting: true,
            api_rate_limit: 1000,
            api_rate_limit_window: Duration::from_secs(60),
        }
    }
}

/// Configuration loader trait
#[async_trait::async_trait]
pub trait ConfigLoader: Send + Sync {
    /// Load configuration from file
    async fn load_from_file(&self, path: &str) -> Result<Config, crate::StepflowError>;

    /// Load configuration from environment
    async fn load_from_env(&self) -> Result<Config, crate::StepflowError>;

    /// Load configuration from string
    async fn load_from_string(&self, content: &str) -> Result<Config, crate::StepflowError>;

    /// Validate configuration
    async fn validate(&self, config: &Config) -> Result<(), crate::StepflowError>;

    /// Save configuration to file
    async fn save_to_file(&self, config: &Config, path: &str) -> Result<(), crate::StepflowError>;
}

/// Configuration manager
pub struct ConfigManager {
    config: Config,
    loader: Box<dyn ConfigLoader>,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new(loader: Box<dyn ConfigLoader>) -> Self {
        Self {
            config: Config::default(),
            loader,
        }
    }

    /// Load configuration
    pub async fn load(&mut self, source: ConfigSource) -> Result<(), crate::StepflowError> {
        let config = match source {
            ConfigSource::File(path) => self.loader.load_from_file(&path).await?,
            ConfigSource::Environment => self.loader.load_from_env().await?,
            ConfigSource::String(content) => self.loader.load_from_string(&content).await?,
        };

        self.loader.validate(&config).await?;
        self.config = config;
        Ok(())
    }

    /// Get configuration
    pub fn get(&self) -> &Config {
        &self.config
    }

    /// Get mutable configuration
    pub fn get_mut(&mut self) -> &mut Config {
        &mut self.config
    }

    /// Save configuration
    pub async fn save(&self, path: &str) -> Result<(), crate::StepflowError> {
        self.loader.save_to_file(&self.config, path).await
    }

    /// Update configuration
    pub async fn update(&mut self, config: Config) -> Result<(), crate::StepflowError> {
        self.loader.validate(&config).await?;
        self.config = config;
        Ok(())
    }
}

/// Configuration source
#[derive(Debug, Clone)]
pub enum ConfigSource {
    File(String),
    Environment,
    String(String),
}

/// Default configuration loader implementation
pub struct DefaultConfigLoader;

#[async_trait::async_trait]
impl ConfigLoader for DefaultConfigLoader {
    async fn load_from_file(&self, path: &str) -> Result<Config, crate::StepflowError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| crate::StepflowError::ConfigurationError(format!("Failed to read config file: {}", e)))?;

        self.load_from_string(&content).await
    }

    async fn load_from_env(&self) -> Result<Config, crate::StepflowError> {
        let mut config = Config::default();

        // Load from environment variables
        if let Ok(server_host) = std::env::var("STEPFLOW_SERVER_HOST") {
            config.server.host = server_host;
        }

        if let Ok(server_port) = std::env::var("STEPFLOW_SERVER_PORT") {
            config.server.port = server_port.parse()
                .map_err(|e| crate::StepflowError::ConfigurationError(format!("Invalid server port: {}", e)))?;
        }

        if let Ok(db_url) = std::env::var("STEPFLOW_DATABASE_URL") {
            config.database.url = db_url;
        }

        if let Ok(secret_key) = std::env::var("STEPFLOW_SECRET_KEY") {
            config.security.secret_key = secret_key;
        }

        if let Ok(jwt_secret) = std::env::var("STEPFLOW_JWT_SECRET") {
            config.security.jwt_secret = jwt_secret;
        }

        Ok(config)
    }

    async fn load_from_string(&self, content: &str) -> Result<Config, crate::StepflowError> {
        let config: Config = serde_json::from_str(content)
            .map_err(|e| crate::StepflowError::ConfigurationError(format!("Failed to parse config: {}", e)))?;

        Ok(config)
    }

    async fn validate(&self, config: &Config) -> Result<(), crate::StepflowError> {
        // Validate server configuration
        if config.server.port == 0 {
            return Err(crate::StepflowError::ConfigurationError("Invalid server port".to_string()));
        }

        if config.server.workers == 0 {
            return Err(crate::StepflowError::ConfigurationError("Invalid number of workers".to_string()));
        }

        // Validate database configuration
        if config.database.url.is_empty() {
            return Err(crate::StepflowError::ConfigurationError("Database URL is required".to_string()));
        }

        // Validate security configuration
        if config.security.secret_key.is_empty() {
            return Err(crate::StepflowError::ConfigurationError("Secret key is required".to_string()));
        }

        if config.security.jwt_secret.is_empty() {
            return Err(crate::StepflowError::ConfigurationError("JWT secret is required".to_string()));
        }

        Ok(())
    }

    async fn save_to_file(&self, config: &Config, path: &str) -> Result<(), crate::StepflowError> {
        let content = serde_json::to_string_pretty(config)
            .map_err(|e| crate::StepflowError::ConfigurationError(format!("Failed to serialize config: {}", e)))?;

        std::fs::write(path, content)
            .map_err(|e| crate::StepflowError::ConfigurationError(format!("Failed to write config file: {}", e)))?;

        Ok(())
    }
} 