use std::fmt::{Display, self};

use mongodb::bson::oid::ObjectId;
use revolt_rocket_okapi::JsonSchema;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema)]
pub enum Role{
    Admin,
    User,
}
impl Role {
    pub fn from_str(s: &str) -> Role {
        match s.to_lowercase().as_str() {
            "admin" => Role::Admin,
            "user" => Role::User,
            _ => Role::User,
        }
    }
}
impl Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Role::Admin => write!(f, "admin"),
            Role::User => write!(f, "user"),
        }
    }
}
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub _id: Option<ObjectId>,
    pub id: String,
    pub email: String,
    pub password: String,
    pub name: String,
    pub location: String,
    pub role: Role,
    pub title: String,
}
impl User {
    pub fn to_response(&self) -> UserPublic {
        UserPublic {
            id: self.id.to_owned(),
            email: self.email.to_owned(),
            name: self.name.to_owned(),
            location: self.location.to_owned(),
            title: self.title.to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct UserPublic {
    pub id: String,
    pub email: String,
    pub name: String,
    pub location: String,
    pub title: String,
}