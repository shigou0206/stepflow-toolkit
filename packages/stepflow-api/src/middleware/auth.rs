use crate::errors::{ApiError, ApiResult, AuthError, AuthResult};
use crate::server::{AuthService, Middleware};
use crate::types::{HttpRequest, HttpResponse, JwtClaims, UserContext};
use async_trait::async_trait;
use axum::{
    extract::{Request, State},
    http::{header, HeaderMap},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use stepflow_core::UserId;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// 认证中间件
pub struct AuthMiddleware {
    auth_service: Arc<dyn AuthService>,
    jwt_secret: String,
    skip_auth_paths: Vec<String>,
}

impl AuthMiddleware {
    pub fn new(auth_service: Arc<dyn AuthService>, jwt_secret: String) -> Self {
        Self {
            auth_service,
            jwt_secret,
            skip_auth_paths: vec![
                "/health".to_string(),
                "/api/v1/auth/login".to_string(),
                "/api/v1/auth/register".to_string(),
                "/api/v1/auth/refresh".to_string(),
                "/api/v1/auth/forgot-password".to_string(),
            ],
        }
    }
    
    pub fn with_skip_auth_paths(mut self, paths: Vec<String>) -> Self {
        self.skip_auth_paths = paths;
        self
    }
    
    /// 验证 JWT 令牌
    async fn validate_jwt_token(&self, token: &str) -> AuthResult<UserContext> {
        let decoding_key = DecodingKey::from_secret(self.jwt_secret.as_ref());
        let validation = Validation::default();
        
        match decode::<JwtClaims>(token, &decoding_key, &validation) {
            Ok(token_data) => {
                let claims = token_data.claims;
                
                // 检查令牌是否过期
                let now = chrono::Utc::now().timestamp();
                if claims.exp < now {
                    return Err(AuthError::ExpiredToken);
                }
                
                // 构建用户上下文
                let user_context = UserContext {
                    user_id: UserId::from_string(claims.user_id),
                    tenant_id: claims.tenant_id,
                    roles: claims.roles,
                    permissions: claims.permissions,
                    session_id: claims.sub,
                    expires_at: chrono::DateTime::from_timestamp(claims.exp, 0)
                        .unwrap_or_else(|| chrono::Utc::now()),
                };
                
                Ok(user_context)
            }
            Err(e) => {
                error!("JWT validation failed: {}", e);
                Err(AuthError::InvalidToken)
            }
        }
    }
    
    /// 验证 API 密钥
    async fn validate_api_key(&self, key: &str) -> AuthResult<UserContext> {
        self.auth_service.validate_api_key(key).await
            .map_err(|e| {
                error!("API key validation failed: {}", e);
                AuthError::InvalidCredentials
            })
    }
    
    /// 从请求头中提取令牌
    fn extract_token_from_headers(&self, headers: &HeaderMap) -> Option<String> {
        headers.get("authorization")
            .and_then(|header| header.to_str().ok())
            .and_then(|header| {
                if header.starts_with("Bearer ") {
                    Some(header[7..].to_string())
                } else {
                    None
                }
            })
    }
    
    /// 从请求头中提取 API 密钥
    fn extract_api_key_from_headers(&self, headers: &HeaderMap) -> Option<String> {
        headers.get("x-api-key")
            .and_then(|header| header.to_str().ok())
            .map(|key| key.to_string())
    }
    
    /// 检查路径是否需要跳过认证
    fn should_skip_auth(&self, path: &str) -> bool {
        self.skip_auth_paths.iter().any(|skip_path| {
            path.starts_with(skip_path) || path == *skip_path
        })
    }
}

#[async_trait]
impl Middleware for AuthMiddleware {
    async fn process_request(&self, request: &mut HttpRequest) -> ApiResult<()> {
        // 检查是否需要跳过认证
        if self.should_skip_auth(&request.path) {
            debug!("Skipping auth for path: {}", request.path);
            return Ok(());
        }
        
        // 尝试从 Authorization 头中提取 JWT 令牌
        if let Some(token) = request.get_header("authorization")
            .and_then(|header| {
                if header.starts_with("Bearer ") {
                    Some(header[7..].to_string())
                } else {
                    None
                }
            }) {
            
            match self.validate_jwt_token(&token).await {
                Ok(user_context) => {
                    request.user_id = Some(user_context.user_id.clone());
                    request.tenant_id = user_context.tenant_id.clone();
                    info!("User authenticated via JWT: {}", user_context.user_id);
                    return Ok(());
                }
                Err(e) => {
                    warn!("JWT validation failed: {}", e);
                    return Err(ApiError::from(e));
                }
            }
        }
        
        // 尝试从 X-API-Key 头中提取 API 密钥
        if let Some(api_key) = request.get_header("x-api-key") {
            match self.validate_api_key(api_key).await {
                Ok(user_context) => {
                    request.user_id = Some(user_context.user_id.clone());
                    request.tenant_id = user_context.tenant_id.clone();
                    info!("User authenticated via API key: {}", user_context.user_id);
                    return Ok(());
                }
                Err(e) => {
                    warn!("API key validation failed: {}", e);
                    return Err(ApiError::from(e));
                }
            }
        }
        
        // 没有有效的认证信息
        error!("No valid authentication found for path: {}", request.path);
        Err(ApiError::Unauthorized("Authentication required".to_string()))
    }
    
    async fn process_response(&self, response: &mut HttpResponse) -> ApiResult<()> {
        // 添加安全头
        response.headers.insert(
            "X-Content-Type-Options".to_string(),
            "nosniff".to_string(),
        );
        response.headers.insert(
            "X-Frame-Options".to_string(),
            "DENY".to_string(),
        );
        response.headers.insert(
            "X-XSS-Protection".to_string(),
            "1; mode=block".to_string(),
        );
        response.headers.insert(
            "Strict-Transport-Security".to_string(),
            "max-age=31536000; includeSubDomains".to_string(),
        );
        
        Ok(())
    }
    
    fn name(&self) -> &str {
        "auth"
    }
    
    fn priority(&self) -> u32 {
        100
    }
}

/// Axum 中间件函数
pub async fn auth_middleware(
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    // 这里可以添加认证逻辑
    // 目前只是简单地传递请求
    Ok(next.run(request).await)
}

/// JWT 认证中间件
pub async fn jwt_auth(
    State(secret): State<String>,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .ok_or_else(|| ApiError::Unauthorized("Missing authorization header".to_string()))?;

    if !auth_header.starts_with("Bearer ") {
        return Err(ApiError::Unauthorized("Invalid authorization header format".to_string()));
    }

    let token = &auth_header[7..]; // Remove "Bearer " prefix

    let token_data = decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|e| ApiError::Unauthorized(format!("Invalid token: {}", e)))?;

    let claims = token_data.claims;

    // 验证令牌是否过期
    let now = chrono::Utc::now().timestamp();
    if claims.exp < now {
        return Err(ApiError::Unauthorized("Token has expired".to_string()));
    }

    // 创建用户上下文
    let user_context = UserContext {
        user_id: UserId::from_string(claims.user_id),
        tenant_id: claims.tenant_id,
        roles: claims.roles,
        permissions: claims.permissions,
        session_id: "".to_string(), // TODO: 从令牌中获取会话ID
        expires_at: chrono::DateTime::from_timestamp(claims.exp, 0).unwrap_or_else(chrono::Utc::now),
    };

    // 将用户上下文添加到请求扩展中
    request.extensions_mut().insert(user_context);

    Ok(next.run(request).await)
}

/// 权限检查中间件
pub struct PermissionMiddleware {
    auth_service: Arc<dyn AuthService>,
    required_permissions: Vec<String>,
}

impl PermissionMiddleware {
    pub fn new(auth_service: Arc<dyn AuthService>, required_permissions: Vec<String>) -> Self {
        Self {
            auth_service,
            required_permissions,
        }
    }
}

#[async_trait]
impl Middleware for PermissionMiddleware {
    async fn process_request(&self, request: &mut HttpRequest) -> ApiResult<()> {
        // 检查用户是否已认证
        let user_id = request.user_id.as_ref()
            .ok_or_else(|| ApiError::Unauthorized("User not authenticated".to_string()))?;
        
        // 获取用户权限
        let user_permissions = self.auth_service.get_user_permissions(user_id.as_str()).await?;
        
        // 检查是否具有所需权限
        for required_permission in &self.required_permissions {
            if !user_permissions.contains(required_permission) {
                return Err(ApiError::Forbidden(
                    format!("Missing required permission: {}", required_permission)
                ));
            }
        }
        
        Ok(())
    }
    
    async fn process_response(&self, _response: &mut HttpResponse) -> ApiResult<()> {
        Ok(())
    }
    
    fn name(&self) -> &str {
        "permission"
    }
    
    fn priority(&self) -> u32 {
        300
    }
}

/// 角色检查中间件
pub struct RoleMiddleware {
    auth_service: Arc<dyn AuthService>,
    required_roles: Vec<String>,
}

impl RoleMiddleware {
    pub fn new(auth_service: Arc<dyn AuthService>, required_roles: Vec<String>) -> Self {
        Self {
            auth_service,
            required_roles,
        }
    }
}

#[async_trait]
impl Middleware for RoleMiddleware {
    async fn process_request(&self, request: &mut HttpRequest) -> ApiResult<()> {
        // 检查用户是否已认证
        let user_id = request.user_id.as_ref()
            .ok_or_else(|| ApiError::Unauthorized("User not authenticated".to_string()))?;
        
        // 获取用户角色
        let user_roles = self.auth_service.get_user_roles(user_id.as_str()).await?;
        
        // 检查是否具有所需角色
        for required_role in &self.required_roles {
            if !user_roles.contains(required_role) {
                return Err(ApiError::Forbidden(
                    format!("Missing required role: {}", required_role)
                ));
            }
        }
        
        Ok(())
    }
    
    async fn process_response(&self, _response: &mut HttpResponse) -> ApiResult<()> {
        Ok(())
    }
    
    fn name(&self) -> &str {
        "role"
    }
    
    fn priority(&self) -> u32 {
        85
    }
}

/// 租户隔离中间件
pub struct TenantMiddleware {
    enforce_tenant_isolation: bool,
}

impl TenantMiddleware {
    pub fn new(enforce_tenant_isolation: bool) -> Self {
        Self {
            enforce_tenant_isolation,
        }
    }
}

#[async_trait]
impl Middleware for TenantMiddleware {
    async fn process_request(&self, request: &mut HttpRequest) -> ApiResult<()> {
        if !self.enforce_tenant_isolation {
            return Ok(());
        }
        
        // 检查用户是否已认证
        let user_id = request.user_id.as_ref()
            .ok_or_else(|| ApiError::Unauthorized("User not authenticated".to_string()))?;
        
        // 检查租户 ID 是否存在
        if request.tenant_id.is_none() {
            return Err(ApiError::BadRequest(
                "Tenant ID is required for multi-tenant operations".to_string()
            ));
        }
        
        debug!("Tenant isolation enforced for user: {} in tenant: {:?}", 
               user_id, request.tenant_id);
        
        Ok(())
    }
    
    async fn process_response(&self, _response: &mut HttpResponse) -> ApiResult<()> {
        Ok(())
    }
    
    fn name(&self) -> &str {
        "tenant"
    }
    
    fn priority(&self) -> u32 {
        80
    }
} 