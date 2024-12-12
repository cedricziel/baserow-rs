use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token: String,
    pub user: User,
}

#[derive(Deserialize, Clone)]
pub struct User {
    pub first_name: String,
    pub username: String,
    pub language: String,
}

#[derive(Deserialize)]
pub struct TokenAuthErrorResponse {
    pub error: String,
}
