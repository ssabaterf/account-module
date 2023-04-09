use revolt_rocket_okapi::JsonSchema;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TransactionRequest{
    pub symbol: String,
    pub amount: f64,
    pub from: String,
    pub to: String,
}
