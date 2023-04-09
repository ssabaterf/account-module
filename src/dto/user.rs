use revolt_rocket_okapi::JsonSchema;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct UserRegisterRequest {
    pub id: Option<String>,
    pub email: String,
    pub password: String,
    pub name: String,
    pub location: String,
    pub title: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct RefreshToken {
    pub(crate) refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct Token {
    pub token: String,
    pub refresh_token: String,
}
