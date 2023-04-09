use chrono::{Utc};
use revolt_rocket_okapi::JsonSchema;
use serde::Serialize;

#[derive(Debug, Serialize, JsonSchema)]
pub struct ErrorResponse {
    pub(crate) cause: String,
    pub(crate) message: String,
    pub(crate) date: String,
}
impl ErrorResponse {
    pub fn new(cause: String, message: String) -> Self {
        Self {
            cause,
            message,
            date: Utc::now().to_rfc3339(),
        }
    }
}