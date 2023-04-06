use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransactionRequest{
    pub symbol: String,
    pub amount: f64,
    pub from: String,
    pub to: String,
}
