use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};

#[derive(Serialize, Debug)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, Debug)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token: String,
    pub user: User,
}

#[derive(Deserialize, Clone, Debug)]
pub struct User {
    pub first_name: String,
    pub username: String,
    pub language: String,
}

#[derive(Deserialize, Debug)]
pub struct TokenAuthErrorResponse {
    pub error: String,
}
