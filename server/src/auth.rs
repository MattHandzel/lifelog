// Authentication and JWT token handling
use actix_web::{error::ErrorUnauthorized, Error, HttpMessage};
use actix_web::dev::{self, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::http::header;
use futures::future::{ok, Future, Ready};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};
use std::env;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use derive_more::{Display, Error as DeriveError};
use std::error::Error as StdError;
use std::fmt;
use uuid::Uuid;

// JWT claim structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,         
    pub name: String,        
    pub role: String,        
    pub exp: usize,         
    pub iat: usize,     
    pub jti: String,         
}

// App error types
#[derive(Debug, Display)]
pub enum AuthError {
    #[display(fmt = "unauthorized")]
    Unauthorized,
    
    #[display(fmt = "forbidden")]
    Forbidden,
    
    #[display(fmt = "internal server error: {}", _0)]
    Internal(String),
}

impl StdError for AuthError {}

impl actix_web::error::ResponseError for AuthError {}

// JWT configuration
pub struct JwtConfig {
    pub secret: String,
    pub expires_in: Duration,
}

impl JwtConfig {
    pub fn from_env() -> Self {
        Self {
            secret: env::var("JWT_SECRET").unwrap_or_else(|_| "development_secret_key_change_in_production".to_string()),
            expires_in: Duration::from_secs(env::var("JWT_EXPIRES_IN")
                .unwrap_or_else(|_| "86400".to_string()) // Default: 24 hours
                .parse::<u64>()
                .unwrap_or(86400)),
        }
    }
}

// Create JWT token
pub fn create_token(user_id: &str, username: &str, role: &str) -> Result<String, AuthError> {
    let config = JwtConfig::from_env();
    
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| AuthError::Internal(e.to_string()))?
        .as_secs() as usize;
    
    let claims = Claims {
        sub: user_id.to_string(),
        name: username.to_string(),
        role: role.to_string(),
        exp: now + config.expires_in.as_secs() as usize,
        iat: now,
        jti: Uuid::new_v4().to_string(),
    };
    
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.secret.as_bytes()),
    )
    .map_err(|e| AuthError::Internal(format!("Failed to create token: {}", e)))
}

// Validate JWT token
pub fn validate_token(token: &str) -> Result<TokenData<Claims>, AuthError> {
    let config = JwtConfig::from_env();
    
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| AuthError::Unauthorized)
}

// Extract token from authorization header
pub fn extract_token(req: &ServiceRequest) -> Result<String, AuthError> {
    req.headers()
        .get(header::AUTHORIZATION)
        .ok_or(AuthError::Unauthorized)
        .and_then(|header_value| {
            header_value
                .to_str()
                .map_err(|_| AuthError::Unauthorized)
                .and_then(|auth_header| {
                    if auth_header.starts_with("Bearer ") {
                        Ok(auth_header[7..].to_string())
                    } else {
                        Err(AuthError::Unauthorized)
                    }
                })
        })
}

pub struct JwtAuth;

impl<S, B> Transform<S, ServiceRequest> for JwtAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = JwtAuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(JwtAuthMiddleware {
            service: Rc::new(service),
        })
    }
}

pub struct JwtAuthMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for JwtAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let path = req.path().to_string();
        if path == "/api/auth/login" || path == "/api/health" || path.starts_with("/api/public") {
            let fut = self.service.call(req);
            return Box::pin(async move {
                let res = fut.await?;
                Ok(res)
            });
        }

        // Attempt to extract and validate token
        let result = extract_token(&req).and_then(|token| {
            let token_data = validate_token(&token)?;
            Ok(token_data.claims)
        });

        match result {
            Ok(claims) => {
                // Attach claims to request extensions for use in handlers
                req.extensions_mut().insert(claims);
                let fut = self.service.call(req);
                Box::pin(async move {
                    let res = fut.await?;
                    Ok(res)
                })
            }
            Err(_) => {
                Box::pin(async { Err(ErrorUnauthorized("Invalid token")) })
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: String,
    pub username: String,
    pub password_hash: String,
    pub role: String,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub role: String,
}

pub struct UserStore {
    users: Vec<User>,
}

impl Default for UserStore {
    fn default() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_secs();

        let default_pwd = env::var("DEFAULT_ADMIN_PASSWORD")
            .unwrap_or_else(|_| "admin".to_string());

        let pwd_hash = bcrypt::hash(default_pwd, bcrypt::DEFAULT_COST)
            .unwrap_or_else(|_| "failed_to_hash".to_string());

        Self {
            users: vec![
                User {
                    id: "1".to_string(),
                    username: "admin".to_string(),
                    password_hash: pwd_hash,
                    role: "admin".to_string(),
                    created_at: now,
                    updated_at: now,
                }
            ],
        }
    }
}

impl UserStore {
    pub fn find_by_username(&self, username: &str) -> Option<User> {
        self.users.iter()
            .find(|user| user.username == username)
            .cloned()
    }
    
    pub fn validate_password(&self, username: &str, password: &str) -> bool {
        if let Some(user) = self.find_by_username(username) {
            bcrypt::verify(password, &user.password_hash).unwrap_or(false)
        } else {
            false
        }
    }
} 