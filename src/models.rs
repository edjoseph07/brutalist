use mongodb::Database;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct AppState {
    pub database: Database,
    pub jwt_secret: String,
    pub cookie_secure: bool,
}

#[derive(Debug, Serialize)]
pub struct PublicUser {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct UsersQuery {
    pub page: Option<u64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct UsersResponse {
    pub users: Vec<PublicUser>,
    pub has_more: bool,
}

#[derive(Debug, Deserialize)]
pub struct SignupRequest {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub message: String,
}
