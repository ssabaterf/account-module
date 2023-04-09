use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Deposit{
    pub symbol: String,
    pub amount: f64,
    pub account: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DepositCreation<T>
where T: Clone + PartialEq + Serialize {
    pub account: T,
    pub tx_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DepositConfirmation {
    pub external_id: String,
}

pub type Withdrawal = Deposit;
pub type WithdrawalCreation<T> = DepositCreation<T>;
pub type WithdrawalConfirmation = DepositConfirmation;