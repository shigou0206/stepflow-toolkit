use crate::models::responses::{HealthResponse, ServiceHealth, DetailedHealthResponse, AppState};
use crate::errors::ApiError;
use axum::{extract::State, Json};
use std::collections::HashMap;

/// 健康检查路由
pub async fn health_check(
    State(_state): State<AppState>,
) -> Result<Json<HealthResponse>, ApiError> {
    let response = HealthResponse {
        status: "healthy".to_string(),
        message: "API is running".to_string(),
        timestamp: chrono::Utc::now(),
    };
    Ok(Json(response))
}

/// 详细健康检查路由
pub async fn detailed_health_check(
    State(state): State<AppState>,
) -> Result<Json<DetailedHealthResponse>, ApiError> {
    let mut services = HashMap::new();
    
    // 检查数据库
    let db_health = match state.database.health_check().await {
        Ok(_) => ServiceHealth {
            status: "healthy".to_string(),
            message: "Database is healthy".to_string(),
            duration: 0,
            timestamp: chrono::Utc::now(),
        },
        Err(e) => ServiceHealth {
            status: "unhealthy".to_string(),
            message: format!("Database error: {}", e),
            duration: 0,
            timestamp: chrono::Utc::now(),
        },
    };
    services.insert("database".to_string(), db_health);
    
    // 检查注册表
    let registry_health = match state.registry.health_check().await {
        Ok(_) => ServiceHealth {
            status: "healthy".to_string(),
            message: "Registry is healthy".to_string(),
            duration: 0,
            timestamp: chrono::Utc::now(),
        },
        Err(e) => ServiceHealth {
            status: "unhealthy".to_string(),
            message: format!("Registry error: {}", e),
            duration: 0,
            timestamp: chrono::Utc::now(),
        },
    };
    services.insert("registry".to_string(), registry_health);
    
    // 检查执行器
    let executor_health = match state.executor.health_check().await {
        Ok(_) => ServiceHealth {
            status: "healthy".to_string(),
            message: "Executor is healthy".to_string(),
            duration: 0,
            timestamp: chrono::Utc::now(),
        },
        Err(e) => ServiceHealth {
            status: "unhealthy".to_string(),
            message: format!("Executor error: {}", e),
            duration: 0,
            timestamp: chrono::Utc::now(),
        },
    };
    services.insert("executor".to_string(), executor_health);
    
    // 检查沙箱
    let sandbox_health = match state.sandbox.health_check().await {
        Ok(_) => ServiceHealth {
            status: "healthy".to_string(),
            message: "Sandbox is healthy".to_string(),
            duration: 0,
            timestamp: chrono::Utc::now(),
        },
        Err(e) => ServiceHealth {
            status: "unhealthy".to_string(),
            message: format!("Sandbox error: {}", e),
            duration: 0,
            timestamp: chrono::Utc::now(),
        },
    };
    services.insert("sandbox".to_string(), sandbox_health);
    
    let overall_status = if services.values().all(|s| s.status == "healthy") {
        "healthy".to_string()
    } else {
        "unhealthy".to_string()
    };

    let response = DetailedHealthResponse {
        status: overall_status,
        services,
        timestamp: chrono::Utc::now(),
    };
    Ok(Json(response))
}

/// 就绪检查路由
pub async fn readiness_check(
    State(_state): State<AppState>,
) -> Result<Json<HealthResponse>, ApiError> {
    let response = HealthResponse {
        status: "ready".to_string(),
        message: "Service is ready".to_string(),
        timestamp: chrono::Utc::now(),
    };
    Ok(Json(response))
} 