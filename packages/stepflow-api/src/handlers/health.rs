use axum::{extract::State, Json};
use crate::models::responses::*;
use crate::errors::*;

/// 健康检查处理器
pub async fn health_check() -> Result<Json<HealthResponse>, ApiError> {
    Ok(Json(HealthResponse {
        status: "healthy".to_string(),
        message: "API is running".to_string(),
        timestamp: chrono::Utc::now(),
    }))
}

/// 详细健康检查
pub async fn detailed_health(
    State(state): State<AppState>,
) -> Result<Json<DetailedHealthResponse>, ApiError> {
    let mut services = std::collections::HashMap::new();

    // 检查数据库连接
    let db_health = match state.database.health_check().await {
        Ok(_) => ServiceHealth {
            status: "healthy".to_string(),
            message: "Database connection is healthy".to_string(),
            duration: 0,
            timestamp: chrono::Utc::now(),
        },
        Err(e) => ServiceHealth {
            status: "unhealthy".to_string(),
            message: format!("Database connection failed: {}", e),
            duration: 0,
            timestamp: chrono::Utc::now(),
        },
    };
    services.insert("database".to_string(), db_health);

    // 检查注册表服务
    let registry_health = match state.registry.health_check().await {
        Ok(_) => ServiceHealth {
            status: "healthy".to_string(),
            message: "Registry service is healthy".to_string(),
            duration: 0,
            timestamp: chrono::Utc::now(),
        },
        Err(e) => ServiceHealth {
            status: "unhealthy".to_string(),
            message: format!("Registry service failed: {}", e),
            duration: 0,
            timestamp: chrono::Utc::now(),
        },
    };
    services.insert("registry".to_string(), registry_health);

    // 检查执行器服务
    let executor_health = match state.executor.health_check().await {
        Ok(_) => ServiceHealth {
            status: "healthy".to_string(),
            message: "Executor service is healthy".to_string(),
            duration: 0,
            timestamp: chrono::Utc::now(),
        },
        Err(e) => ServiceHealth {
            status: "unhealthy".to_string(),
            message: format!("Executor service failed: {}", e),
            duration: 0,
            timestamp: chrono::Utc::now(),
        },
    };
    services.insert("executor".to_string(), executor_health);

    // 检查沙箱服务
    let sandbox_health = match state.sandbox.health_check().await {
        Ok(_) => ServiceHealth {
            status: "healthy".to_string(),
            message: "Sandbox service is healthy".to_string(),
            duration: 0,
            timestamp: chrono::Utc::now(),
        },
        Err(e) => ServiceHealth {
            status: "unhealthy".to_string(),
            message: format!("Sandbox service failed: {}", e),
            duration: 0,
            timestamp: chrono::Utc::now(),
        },
    };
    services.insert("sandbox".to_string(), sandbox_health);

    // 确定整体状态
    let overall_status = if services.values().all(|s| s.status == "healthy") {
        "healthy".to_string()
    } else {
        "unhealthy".to_string()
    };

    Ok(Json(DetailedHealthResponse {
        status: overall_status,
        services,
        timestamp: chrono::Utc::now(),
    }))
}

/// 就绪检查
pub async fn readiness_check(
    State(_state): State<AppState>,
) -> Result<Json<HealthResponse>, ApiError> {
    // 检查服务是否准备好接受请求
    Ok(Json(HealthResponse {
        status: "ready".to_string(),
        message: "Service is ready to accept requests".to_string(),
        timestamp: chrono::Utc::now(),
    }))
}

/// 存活检查
pub async fn liveness_check() -> Result<Json<HealthResponse>, ApiError> {
    Ok(Json(HealthResponse {
        status: "alive".to_string(),
        message: "Service is alive".to_string(),
        timestamp: chrono::Utc::now(),
    }))
} 