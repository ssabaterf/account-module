use std::{time::SystemTime, fmt::Display};

use chrono::{DateTime, Utc};
use rand::{distributions::Alphanumeric, Rng};
use revolt_rocket_okapi::JsonSchema;
use serde::{Serialize, Deserialize};
use sha2::Digest;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum TransactionType {
    Deposit,
    Withdraw,
    Transfer,
    Trading,
}
impl Display for TransactionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionType::Deposit => write!(f, "Deposit"),
            TransactionType::Withdraw => write!(f, "Withdraw"),
            TransactionType::Transfer => write!(f, "Transfer"),
            TransactionType::Trading => write!(f, "Trading"),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Completed,
    Cancelled,
    Failed,
}
impl Display for TransactionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionStatus::Pending => write!(f, "Pending"),
            TransactionStatus::Confirmed => write!(f, "Confirmed"),
            TransactionStatus::Completed => write!(f, "Completed"),
            TransactionStatus::Cancelled => write!(f, "Cancelled"),
            TransactionStatus::Failed => write!(f, "Failed"),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Confirmed {
    pub id_confirmer: String,
    pub timestamp: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct HashEvents {
    pub hash: String,
    pub timestamp: String,
    pub field_changed: String,
    pub value: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct FeeReason {
    pub reason: String,
    pub amount: f64,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Transaction {
    pub tx_id: String,
    pub external_id: Option<String>,
    pub transaction_type: TransactionType,
    pub transaction_status: TransactionStatus,
    pub asset: String,
    pub amount: f64,
    pub total_amount: f64,
    pub from_wallet: Option<String>,
    pub to_wallet: Option<String>,
    pub timestamp: String,
    pub fee: Vec<FeeReason>,
    pub memo: String,
    pub hash_chain: Option<String>,
    pub block: Option<String>,
    pub confirmations: Vec<Confirmed>,
    pub confirmations_required: u32,
    pub hash: Vec<HashEvents>,
}
impl Transaction {
    pub fn new_transfer(
        asset: String,
        amount: f64,
        from_wallet: String,
        to_wallet: String,
        memo: String,
        confirmations_required: u32,
    ) -> Transaction {
        Transaction {
            tx_id: transaction_id_generator(),
            external_id: None,
            transaction_type: TransactionType::Transfer,
            transaction_status: TransactionStatus::Pending,
            asset,
            amount,
            total_amount:amount,
            from_wallet:Some(from_wallet),
            to_wallet:Some(to_wallet),
            timestamp: timestamp_generator(),
            fee: Vec::new(),
            memo,
            hash_chain: None,
            block: None,
            confirmations: Vec::new(),
            confirmations_required,
            hash: Vec::new(),
        }
    }
    pub fn new_deposit(
        asset: String,
        amount: f64,
        to_wallet: String,
        confirmations_required: u32,
    ) -> Transaction {
        Transaction {
            tx_id: transaction_id_generator(),
            external_id: None,
            transaction_type: TransactionType::Deposit,
            transaction_status: TransactionStatus::Pending,
            asset,
            amount,
            total_amount:amount,
            from_wallet:None,
            to_wallet:Some(to_wallet),
            timestamp: timestamp_generator(),
            fee: Vec::new(),
            memo:"Deposit".to_string(),
            hash_chain: None,
            block: None,
            confirmations: Vec::new(),
            confirmations_required,
            hash: Vec::new(),
        }
    }
    pub fn new_withdraw(
        asset: String,
        amount: f64,
        from_wallet: String,
        confirmations_required: u32,
    ) -> Transaction {
        Transaction {
            tx_id: transaction_id_generator(),
            external_id: None,
            transaction_type: TransactionType::Withdraw,
            transaction_status: TransactionStatus::Pending,
            asset,
            amount,
            total_amount:amount,
            from_wallet:Some(from_wallet),
            to_wallet:None,
            timestamp: timestamp_generator(),
            fee: Vec::new(),
            memo:"Withdraw".to_string(),
            hash_chain: None,
            block: None,
            confirmations: Vec::new(),
            confirmations_required,
            hash: Vec::new(),
        }
    }
    pub fn add_fee(&mut self, reason: String, amount: f64) {
        self.fee.push(FeeReason { reason, amount });
        self.total_amount+=amount;
        self.create_hash_event("fee".to_string(), amount.to_string());
    }
    pub fn confirm_transaction(&mut self, id_confirmer: String)->Result<(),String> {
        if self.transaction_status == TransactionStatus::Pending {
            self.confirmations.push(Confirmed {
                id_confirmer:id_confirmer.clone(),
                timestamp: timestamp_generator(),
            });
            self.create_hash_event("id_confirmer".to_string(), id_confirmer);
            if self.confirmations.len() as u32 >= self.confirmations_required {
                self.transaction_status = TransactionStatus::Confirmed;
            }
            self.create_hash_event(
                "transaction_status".to_string(),
                self.transaction_status.to_string(),
            );
            Ok(())
        }
        else{
            Err("Transaction is not pending".to_string())
        }
    }
    pub fn complete_transaction(&mut self, external_id: String)->Result<(),String> {
        if self.transaction_status == TransactionStatus::Confirmed {
            self.external_id = Some(external_id.clone());
            self.create_hash_event("external_id".to_string(), external_id);
            self.transaction_status = TransactionStatus::Completed;
            self.create_hash_event(
                "transaction_status".to_string(),
                self.transaction_status.to_string(),
            );
            Ok(())
        }
        else{
            Err("Transaction is not confirmed".to_string())
        }        
    }
    pub fn fail_transaction(&mut self)->Result<(),String> {
        if self.transaction_status == TransactionStatus::Confirmed {
            self.transaction_status = TransactionStatus::Failed;
            self.create_hash_event(
                "transaction_status".to_string(),
                self.transaction_status.to_string(),
            );
            Ok(())
        }
        else{
            Err("Transaction is not confirmed".to_string())
        }
    }
    pub fn cancel_transaction(&mut self)->Result<(),String> {
        if self.transaction_status == TransactionStatus::Pending {
            self.transaction_status = TransactionStatus::Cancelled;
            self.create_hash_event(
                "transaction_status".to_string(),
                self.transaction_status.to_string(),
            );
            Ok(())
        }
        else{
            Err("Transaction is not pending or confirmed".to_string())
        }
    }
    pub fn create_hash_event(&mut self, field_changed: String, value: String) {
        self.hash.push(HashEvents {
            hash: self.hash_generator(),
            timestamp: timestamp_generator(),
            field_changed,
            value,
        })
    }
    pub fn hash_generator(&self) -> String {
        let fee_string = self
            .fee
            .iter()
            .map(|x| x.amount.to_string())
            .collect::<Vec<String>>()
            .join("");
        let hash_string = match &self.hash_chain {
            Some(hash) => hash.clone(),
            None => "None".to_string(),
        };
        let block_string = match &self.block {
            Some(block) => block.clone(),
            None => "None".to_string(),
        };
        let external_string = match &self.external_id {
            Some(external) => external.clone(),
            None => "None".to_string(),
        };
        let confirmation_string = self
            .confirmations
            .iter()
            .map(|x| x.id_confirmer.clone())
            .collect::<Vec<String>>()
            .join("");
        let mut full_string = self.tx_id.clone();
        full_string.push_str(&external_string);
        full_string.push_str(&self.transaction_type.to_string());
        full_string.push_str(&self.transaction_status.to_string());
        full_string.push_str(&self.asset);
        full_string.push_str(&self.amount.to_string());
        full_string.push_str(&self.from_wallet.clone().unwrap_or("Deposit".to_string()));
        full_string.push_str(&self.to_wallet.clone().unwrap_or("Withdrawal".to_string()));
        full_string.push_str(&self.timestamp);
        full_string.push_str(&fee_string);
        full_string.push_str(&self.memo);
        full_string.push_str(&hash_string);
        full_string.push_str(&block_string);
        full_string.push_str(&confirmation_string);
        full_string.push_str(&self.confirmations_required.to_string());

        let mut hasher = sha2::Sha256::new();
        hasher.update(full_string);
        let result = hasher.finalize();
        let hash_string = format!("{:x}", result);
        hash_string
    }
}
fn transaction_id_generator() -> String {
    let mut rng = rand::thread_rng();
    (0..32)
        .map(|_| rng.sample(Alphanumeric))
        .map(|x| (x) as char)
        .collect()
}
fn timestamp_generator() -> String {
    let now = SystemTime::now();
    let datetime = DateTime::<Utc>::from(now);
    datetime.to_rfc3339()
}
