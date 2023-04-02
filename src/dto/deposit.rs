use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Deposit{
    pub symbol: String,
    pub amount: f64,
    pub account: String,
}