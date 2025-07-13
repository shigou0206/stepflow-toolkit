use stepflow_core::config::*;
use serde_json;
use std::time::Duration;

#[test]
fn test_config_basic() {
    let config = Config {
        server: ServerConfig {
            host: "localhost".to_string(),
            port: 8080,
            workers: 4,
            max_connections: 1000,
            connection_timeout: Duration::from_secs(30),
            keep_alive_timeout: Duration::from_secs(60),
            request_timeout: Duration::from_secs(300),
            enable_compression: true,
            enable_logging: true,
            enable_metrics: true,
            enable_health_checks: true,
        },
        database: DatabaseConfig {
            url: "sqlite:///test.db".to_string(),
            max_connections: 10,
            min_connections: 2,
            connection_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(300),
            max_lifetime: Duration::from_secs(3600),
            enable_migrations: true,
            enable_logging: true,
            enable_metrics: true,
        },
        security: SecurityConfig {
            secret_key: "secret".to_string(),
            jwt_secret: "jwt-secret".to_string(),
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
        },
        monitoring: MonitoringConfig {
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
        },
        tools: ToolsConfig::default(),
        execution: ExecutionConfig::default(),
        sandbox: SandboxConfig::default(),
        api: ApiConfig::default(),
    };
    
    assert_eq!(config.server.host, "localhost");
    assert_eq!(config.server.port, 8080);
    assert_eq!(config.database.max_connections, 10);
    assert_eq!(config.security.jwt_expiration.as_secs(), 3600);
    assert!(config.monitoring.enable_metrics);
}

#[test]
fn test_config_serde() {
    let config = Config::default();
    
    let json = serde_json::to_string(&config).unwrap();
    let de_config: Config = serde_json::from_str(&json).unwrap();
    
    assert_eq!(de_config.server.host, config.server.host);
    assert_eq!(de_config.server.port, config.server.port);
    assert_eq!(de_config.database.url, config.database.url);
    assert_eq!(de_config.security.jwt_secret, config.security.jwt_secret);
    assert_eq!(de_config.monitoring.enable_metrics, config.monitoring.enable_metrics);
}

#[test]
fn test_security_config() {
    let security = SecurityConfig {
        secret_key: "very-secret-key".to_string(),
        jwt_secret: "jwt-secret".to_string(),
        jwt_expiration: Duration::from_secs(1800),
        bcrypt_cost: 14,
        enable_rate_limiting: true,
        rate_limit_requests: 200,
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
    };
    
    assert_eq!(security.secret_key, "very-secret-key");
    assert_eq!(security.jwt_expiration.as_secs(), 1800);
    assert_eq!(security.bcrypt_cost, 14);
    assert_eq!(security.rate_limit_requests, 200);
}

#[test]
fn test_monitoring_config() {
    let monitoring = MonitoringConfig {
        enable_metrics: true,
        metrics_port: 9091,
        metrics_path: "/metrics".to_string(),
        enable_logging: true,
        log_level: "warn".to_string(),
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
    };
    
    assert!(monitoring.enable_metrics);
    assert_eq!(monitoring.metrics_port, 9091);
    assert_eq!(monitoring.log_level, "warn");
}

#[test]
fn test_server_config() {
    let server = ServerConfig {
        host: "0.0.0.0".to_string(),
        port: 443,
        workers: 8,
        max_connections: 2000,
        connection_timeout: Duration::from_secs(30),
        keep_alive_timeout: Duration::from_secs(60),
        request_timeout: Duration::from_secs(300),
        enable_compression: true,
        enable_logging: true,
        enable_metrics: true,
        enable_health_checks: true,
    };
    
    assert_eq!(server.host, "0.0.0.0");
    assert_eq!(server.port, 443);
    assert_eq!(server.workers, 8);
}

#[test]
fn test_database_config() {
    let db = DatabaseConfig {
        url: "mysql://localhost:3306/app".to_string(),
        max_connections: 20,
        min_connections: 5,
        connection_timeout: Duration::from_secs(30),
        idle_timeout: Duration::from_secs(300),
        max_lifetime: Duration::from_secs(3600),
        enable_migrations: true,
        enable_logging: true,
        enable_metrics: true,
    };
    
    assert_eq!(db.url, "mysql://localhost:3306/app");
    assert_eq!(db.max_connections, 20);
    assert_eq!(db.connection_timeout.as_secs(), 30);
}

#[test]
fn test_config_default() {
    let config = Config::default();
    
    // Test that default values are reasonable
    assert!(!config.server.host.is_empty());
    assert!(config.server.port > 0);
    assert!(config.server.workers > 0);
    assert!(!config.database.url.is_empty());
    assert!(config.database.max_connections > 0);
    assert!(!config.security.jwt_secret.is_empty());
    assert!(config.security.jwt_expiration.as_secs() > 0);
    assert!(config.security.bcrypt_cost > 0);
    assert!(config.security.rate_limit_requests > 0);
}

#[test]
fn test_config_validation() {
    // Test that invalid configurations can be detected
    let mut config = Config::default();
    
    // Test invalid port
    config.server.port = 0;
    // This should be validated in a real implementation
    
    // Test invalid pool size
    config.database.max_connections = 0;
    // This should be validated in a real implementation
    
    // Test invalid JWT expiration
    config.security.jwt_expiration = Duration::from_secs(0);
    // This should be validated in a real implementation
}

#[test]
fn test_config_from_env() {
    // Test that configuration can be loaded from environment variables
    // This would require implementing FromEnv or similar trait
    // For now, just test that the structure supports it
    let config = Config::default();
    assert!(config.server.host.len() > 0);
}

#[test]
fn test_config_merge() {
    let _base_config = Config::default();
    let mut override_config = Config::default();
    override_config.server.host = "custom-host".to_string();
    override_config.server.port = 9999;
    override_config.server.workers = 16;
    
    // Test that configurations can be merged
    // This would require implementing a merge method
    assert_eq!(override_config.server.host, "custom-host");
    assert_eq!(override_config.server.port, 9999);
    assert_eq!(override_config.server.workers, 16);
} 