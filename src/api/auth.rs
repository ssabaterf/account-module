use revolt_rocket_okapi::openapi;
use rocket::{http::Status, post, serde::json::Json, State};
use uuid::Uuid;

use crate::{
    domain::{
        account::Account,
        user::{Role, User, UserPublic},
    },
    dto::user::{LoginRequest, RefreshToken, Token, UserRegisterRequest},
    mongo::{Crud, Repository},
    response::error::ErrorResponse,
    security::jwt::{decode_jwt_return_id, encode_token, encode_token_by_refresh, hash_text},
};

#[openapi(tag = "Auths")]
#[post("/auths/register", format = "json", data = "<new_user>")]
pub async fn register(
    db: &State<Repository<User>>,
    new_user: Json<UserRegisterRequest>,
) -> Result<Json<UserPublic>, (Status, Json<ErrorResponse>)> {
    let mut data = User {
        _id: None,
        id: Uuid::new_v4().to_string(),
        email: new_user.email.to_owned(),
        password: new_user.password.to_owned(),
        name: new_user.name.to_owned(),
        role: Role::User,
        location: new_user.location.to_owned(),
        title: new_user.title.to_owned(),
    };
    let user = db
        .get_by_fields(vec!["email".to_string()], vec![data.email.clone()])
        .await;
    if user.is_ok() && !user.unwrap().is_empty() {
        return Err((
            Status::BadRequest,
            Json(ErrorResponse::new(
                "Email already exists".to_string(),
                "Email already exists".to_string(),
            )),
        ));
    }

    let hash_password = match hash_text(data.password.clone(), 4) {
        Ok(hash) => hash,
        Err(_) => {
            return Err((
                Status::InternalServerError,
                Json(ErrorResponse::new(
                    "Error hashing password".to_string(),
                    "Error hashing password".to_string(),
                )),
            ))
        }
    };
    data.password = hash_password;
    let user_detail = db.create(data.clone()).await;
    match user_detail {
        Ok(_) => Ok(Json(data.to_response())),
        Err(err) => Err((
            Status::InternalServerError,
            Json(ErrorResponse::new("Error creating user".to_string(), err)),
        )),
    }
}

#[openapi(tag = "Auths")]
#[post("/auths/login", format = "json", data = "<option_login_request>")]
pub async fn login(
    db: &State<Repository<User>>,
    resource_db: &State<Repository<Account>>,
    option_login_request: Option<Json<LoginRequest>>,
) -> Result<Json<Token>, (Status, Json<ErrorResponse>)> {
    let login_request = match option_login_request {
        Some(login_request) => login_request,
        None => {
            return Err((
                Status::BadRequest,
                Json(ErrorResponse::new(
                    "Invalid request".to_string(),
                    "Invalid request".to_string(),
                )),
            ))
        }
    };
    let user = db
        .get_by_fields(vec!["email".to_string()], vec![login_request.email.clone()])
        .await
        .unwrap_or(vec![]);
    if user.len() != 1 {
        return Err((
            Status::InternalServerError,
            Json(ErrorResponse::new(
                "Error getting user".to_string(),
                "Error getting user".to_string(),
            )),
        ));
    }
    let resources: Vec<String> = resource_db
        .get_by_fields(vec!["user_owner_id".to_string()], vec![user[0].id.clone()])
        .await
        .unwrap_or(vec![])
        .iter()
        .map(|acc| acc.account_number.clone())
        .collect();
    encode_token(&user[0], resources, login_request.password.clone()).await
}

#[openapi(tag = "Auths")]
#[post(
    "/auths/refresh-token",
    format = "json",
    data = "<option_refresh_token>"
)]
pub async fn refresh_tokens(
    database: &State<Repository<User>>,
    resource_db: &State<Repository<Account>>,
    option_refresh_token: Option<Json<RefreshToken>>,
) -> Result<Json<Token>, (Status, Json<ErrorResponse>)> {
    match option_refresh_token {
        Some(refresh_token) => match decode_jwt_return_id(refresh_token).await {
            Ok(id) => {
                let user = match database.get_by_id(&id).await {
                    Ok(user) => user,
                    Err(_) => {
                        return Err((
                            Status::Unauthorized,
                            Json(ErrorResponse::new(
                                "Error getting user".to_string(),
                                "Error getting user".to_string(),
                            )),
                        ))
                    }
                };
                let resources: Vec<String> = resource_db
                    .get_by_fields(vec!["user_owner_id".to_string()], vec![user.id.clone()])
                    .await
                    .unwrap_or(vec![])
                    .iter()
                    .map(|acc| acc.account_number.clone())
                    .collect();
                match encode_token_by_refresh(&user, resources).await {
                    Ok(token) => Ok(token),
                    Err(_) => Err((
                        Status::Unauthorized,
                        Json(ErrorResponse::new(
                            "Error encoding token".to_string(),
                            "Error encoding token".to_string(),
                        )),
                    )),
                }
            }
            Err(_) => Err((
                Status::Unauthorized,
                Json(ErrorResponse::new(
                    "Error decoding token".to_string(),
                    "Error decoding token".to_string(),
                )),
            )),
        },
        None => Err((
            Status::BadRequest,
            Json(ErrorResponse::new(
                "Invalid request".to_string(),
                "Invalid request".to_string(),
            )),
        )),
    }
}
