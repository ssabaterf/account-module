use revolt_rocket_okapi::{JsonSchema, OpenApiFromRequest};
use rocket::{request::{FromRequest, Outcome}, Request, http::Status};
use serde::{Serialize, Deserialize};

use crate::{security::jwt::{DecodeJwtHelper, get_jwt_secret, decode_jwt, check_data_from_auth_header}, domain::user::Role};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, OpenApiFromRequest)]
pub struct AuthorizedUser {
    pub user_id: String,
    pub role: Role,
    pub resource: Vec<String>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthorizedUser {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let auth_header = request.headers().get_one("Authorization");
        let secret = &get_jwt_secret().await;
        match check_data_from_auth_header(auth_header) {
            Ok(vec_header) => match decode_jwt(vec_header[1].to_string(), secret) {
                DecodeJwtHelper::Ok(token_data) => Outcome::Success(AuthorizedUser {
                    user_id: token_data.claims.user_id,
                    role: Role::from_str(&token_data.claims.role),
                    resource: token_data.claims.resource.split(',').map(|s| s.to_string()).collect(),
                }),
                DecodeJwtHelper::Err => Outcome::Failure((Status::Unauthorized, ())),
            },
            Err(_) => Outcome::Failure((Status::Unauthorized, ())),
        }
    }
}
