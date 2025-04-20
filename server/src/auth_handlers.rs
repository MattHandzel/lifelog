// Authentication endpoint handlers (login, profile)
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use crate::auth::{self, LoginRequest, LoginResponse, UserInfo, UserStore};
use crate::error::ApiError;
use serde_json::json;

pub struct AuthState {
    pub user_store: Arc<Mutex<UserStore>>,
}

pub async fn login(
    data: web::Data<AuthState>,
    login_req: web::Json<LoginRequest>,
) -> Result<HttpResponse, ApiError> {
    let user_store = data.user_store.lock().unwrap();
    
    if !user_store.validate_password(&login_req.username, &login_req.password) {
        return Err(ApiError::Unauthorized("Invalid username or password".to_string()));
    }
    
    let user = user_store
        .find_by_username(&login_req.username)
        .ok_or_else(|| ApiError::Unauthorized("User not found".to_string()))?;
    
    let token = auth::create_token(&user.id, &user.username, &user.role)
        .map_err(|e| ApiError::InternalError(format!("Failed to create token: {}", e)))?;
    
    let response = LoginResponse {
        token,
        user: UserInfo {
            id: user.id,
            username: user.username,
            role: user.role,
        },
    };
    
    Ok(HttpResponse::Ok().json(response))
}

#[derive(Serialize, Deserialize)]
pub struct ProfileResponse {
    pub user: UserInfo,
}

pub async fn get_profile(
    data: web::Data<AuthState>,
    claims: web::ReqData<auth::Claims>,
) -> Result<HttpResponse, ApiError> {
    let user_store = data.user_store.lock().unwrap();
    
    let user = user_store
        .find_by_username(&claims.sub)
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;
    
    let response = ProfileResponse {
        user: UserInfo {
            id: user.id,
            username: user.username,
            role: user.role,
        },
    };
    
    Ok(HttpResponse::Ok().json(response))
} 