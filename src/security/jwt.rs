use bcrypt::{hash, verify};
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use lazy_static::lazy_static;
use rocket::{http::Status, serde::json::Json};
use serde::{Deserialize, Serialize};
use std::env;

use crate::{
    domain::{
        user::{Role, User},
    },
    dto::user::{RefreshToken, Token},
    response::error::ErrorResponse,
};

//encode prepare data
pub async fn encode_token(
    user: &User,
    resources: Vec<String>,
    password: String,
) -> Result<Json<Token>, (Status, Json<ErrorResponse>)> {
    match verify(password, &user.password) {
        Ok(true) => {
            match encode_token_and_refresh(
                user.id.clone(),
                user.role.clone(),
                resources.join(","),
                get_jwt_secret().await,
                get_jwt_refresh().await,
                get_jwt_refresh_expiration().await,
                get_jwt_expiration().await,
            ) {
                Ok(tokens) => Ok(Json(tokens)),
                Err(_) => Err((
                    Status::InternalServerError,
                    Json(ErrorResponse::new(
                        "Error encoding token".to_string(),
                        "Error encoding token".to_string(),
                    )),
                )),
            }
        }
        Ok(false) => Err((
            Status::BadRequest,
            Json(ErrorResponse::new(
                "Invalid password".to_string(),
                "Invalid password".to_string(),
            )),
        )),
        Err(_) => Err((
            Status::InternalServerError,
            Json(ErrorResponse::new(
                "Error verifying password".to_string(),
                "Error verifying password".to_string(),
            )),
        )),
    }
}
pub fn check_data_from_auth_header(auth_header: Option<&str>) -> Result<Vec<&str>, ()> {
    return if let Some(auth_string) = auth_header {
        let vec_header = auth_string.split_whitespace().collect::<Vec<_>>();
        if vec_header.len() != 2
            && vec_header[0] == "Bearer"
            && !vec_header[0].is_empty()
            && !vec_header[1].is_empty()
        {
            Err(())
        } else {
            Ok(vec_header)
        }
    } else {
        Err(())
    };
}
//encode prepare data
pub async fn encode_token_by_refresh(
    user: &User,
    resources: Vec<String>,
) -> Result<Json<Token>, (Status, Json<ErrorResponse>)> {
    match encode_token_and_refresh(
        user.id.clone(),
        user.role.clone(),
        resources.join(","),
        get_jwt_secret().await,
        get_jwt_refresh().await,
        get_jwt_refresh_expiration().await,
        get_jwt_expiration().await,
    ) {
        Ok(tokens) => Ok(Json(tokens)),
        Err(_) => Err((
            Status::InternalServerError,
            Json(ErrorResponse::new(
                "Error encoding token".to_string(),
                "Error encoding token".to_string(),
            )),
        )),
    }
}

//decode jwt from body and return id
pub async fn decode_jwt_return_id(refresh_token: Json<RefreshToken>) -> Result<String, ()> {
    match decode_jwt(
        refresh_token.refresh_token.to_string(),
        get_jwt_refresh().await,
    ) {
        DecodeJwtHelper::Ok(token_data) => {
            let id_str = token_data.claims.user_id;
            Ok(id_str)
        }
        DecodeJwtHelper::Err => Err(()),
    }
}

pub fn encode_token_and_refresh(
    id: String,
    role: Role,
    resource: String,
    jwt_secret: &str,
    refresh_token_secret: &str,
    expiration_refresh_token: &i64,
    expiration_token: &i64,
) -> Result<Token, ()> {
    match encode_jwt(
        id.clone(),
        role.clone(),
        resource.clone(),
        jwt_secret,
        expiration_token,
    ) {
        EncodeJwtHelper::Ok(token) => {
            match encode_jwt(
                id,
                role,
                resource,
                refresh_token_secret,
                expiration_refresh_token,
            ) {
                EncodeJwtHelper::Ok(refresh_token) => Ok(Token {
                    token,
                    refresh_token,
                }),
                EncodeJwtHelper::Err => Err(()),
            }
        }
        EncodeJwtHelper::Err => Err(()),
    }
}
pub fn decode_jwt(token: String, secret: &str) -> DecodeJwtHelper {
    let token = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    );
    match token {
        Ok(token_string) => DecodeJwtHelper::Ok(Box::new(token_string)),
        Err(_) => DecodeJwtHelper::Err,
    }
}
pub fn hash_text(text: String, cost: u32) -> Result<String, Status> {
    match hash(text, cost) {
        Ok(hash_text) => Ok(hash_text),
        Err(_) => Err(Status::BadRequest),
    }
}
pub enum EncodeJwtHelper {
    Ok(String),
    Err,
}

pub enum DecodeJwtHelper {
    Ok(Box<TokenData<Claims>>),
    Err,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: String,
    pub role: String,
    pub resource: String,
    pub exp: usize,
}

pub fn encode_jwt(
    id: String,
    role: Role,
    resource: String,
    secret: &str,
    expiration: &i64,
) -> EncodeJwtHelper {
    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::seconds(*expiration))
        .expect("valid timestamp")
        .timestamp();

    let my_claims = Claims {
        user_id: id,
        role: role.to_string(),
        resource,
        exp: expiration as usize,
    };
    match encode(
        &Header::default(),
        &my_claims,
        &EncodingKey::from_secret(secret.as_ref()),
    ) {
        Ok(token) => EncodeJwtHelper::Ok(token),
        Err(_) => EncodeJwtHelper::Err,
    }
}
lazy_static! {
    static ref JWT_SECRET: String =
        env::var("JWT_SECRET").expect("Error loading env variable: JWT_SECRET");
    static ref JWT_REFRESH: String =
        env::var("JWT_REFRESH").expect("Error loading env variable: JWT_REFRESH");
    static ref JWT_EXPIRES_IN: i64 =
        env::var("JWT_EXPIRES_IN")
            .expect("Error loading env variable: JWT_REFRESH")
            .parse()
            .expect("Error parsing env variable: JWT_REFRESH");
    static ref JWT_REFRESH_EXPIRES_IN: i64 =
        env::var("JWT_REFRESH_EXPIRES_IN")
            .expect("Error loading env variable: JWT_REFRESH_EXPIRES_IN")
            .parse()
            .expect("Error parsing env variable: JWT_REFRESH_EXPIRES_IN");
}

pub async fn get_jwt_secret() -> &'static str {
    &JWT_SECRET
}
pub async fn get_jwt_refresh() -> &'static str {
    &JWT_REFRESH
}
pub async fn get_jwt_expiration() -> &'static i64 {
    &JWT_EXPIRES_IN
}

pub async fn get_jwt_refresh_expiration() -> &'static i64 {
    &JWT_REFRESH_EXPIRES_IN
}
