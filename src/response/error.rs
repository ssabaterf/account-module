use chrono::{Utc};
use serde::Serialize;

#[derive(Debug, Serialize)]
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