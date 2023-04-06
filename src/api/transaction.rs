use rocket::{http::Status, post, serde::{json::Json, DeserializeOwned}, State};
use serde::Serialize;

use crate::{
    domain::{
        asset::{Asset, AssetManager, AssetType},
        ledger::{Crypto, Fiat, FungibleTradeable, Accounting},
        transaction::{Transaction, TransactionType},
    },
    dto::transaction::TransactionRequest,
    mongo::{Crud, Repository},
    response::error::ErrorResponse,
};

#[post("/", format = "json", data = "<transaction>")]
pub async fn submit_transaction(
    transaction: Json<TransactionRequest>,
    transaction_db: &State<Repository<Transaction>>,
    fiat_db: &State<Repository<Fiat>>,
    crypto_db: &State<Repository<Crypto>>,
    asset_master: &State<AssetManager>,
) -> Result<Json<Transaction>, (Status, Json<ErrorResponse>)> {
    let req = transaction.0;
    let asset = match asset_master.get_by_symbol(&req.symbol) {
        Some(asset) => asset,
        None => {
            return Err((
                Status::BadRequest,
                Json(ErrorResponse::new(
                    "Invalid symbol".to_string(),
                    "Symbol is not supported".to_string(),
                )),
            ))
        }
    };
    let mut id_from = req.from.clone();
    id_from.push('_');
    id_from.push_str(&req.symbol);
    let mut id_to = req.to.clone();
    id_to.push('_');
    id_to.push_str(&req.symbol);
    match &asset.asset_type {
        AssetType::Crypto => {
            match process_tx(&id_from, &id_to, crypto_db, transaction_db, asset, req).await
            {
                Ok(transaction) => Ok(Json(transaction)),
                Err(e) => {
                    Err((
                        Status::BadRequest,
                        Json(ErrorResponse::new("Invalid transaction".to_string(), e)),
                    ))
                }
            }
        }
        AssetType::Fiat => {
            match process_tx(&id_from, &id_to, fiat_db, transaction_db, asset, req).await {
                Ok(transaction) => Ok(Json(transaction)),
                Err(e) => {
                    Err((
                        Status::BadRequest,
                        Json(ErrorResponse::new("Invalid transaction".to_string(), e)),
                    ))
                }
            }
        }
    }
}
#[post("/<id>/confirm", format = "json")]
pub async fn confirm_transaction(
    id: String,
    transaction_db: &State<Repository<Transaction>>,
    fiat_db: &State<Repository<Fiat>>,
    crypto_db: &State<Repository<Crypto>>,
    asset_master: &State<AssetManager>,
) -> Result<Json<Transaction>, (Status, Json<ErrorResponse>)> {
    let id_confirmer = "11111".to_string();
    let transaction = match transaction_db.get_by_id(&id).await {
        Ok(transaction) => transaction,
        Err(e) => {
            return Err((
                Status::BadRequest,
                Json(ErrorResponse::new("Invalid transaction".to_string(), e)),
            ))
        }
    };
    let asset = match asset_master.get_by_symbol(&transaction.asset) {
        Some(asset) => asset,
        None => {
            return Err((
                Status::BadRequest,
                Json(ErrorResponse::new(
                    "Invalid symbol".to_string(),
                    "Symbol is not supported".to_string(),
                )),
            ))
        }
    };
    match &asset.asset_type {
        AssetType::Crypto => {
            match confirm_tx(crypto_db, transaction_db, asset, transaction, id_confirmer).await
            {
                Ok(transaction) => Ok(Json(transaction)),
                Err(e) => {
                    Err((
                        Status::BadRequest,
                        Json(ErrorResponse::new("Invalid transaction".to_string(), e)),
                    ))
                }
            }
        }
        AssetType::Fiat => {
            match confirm_tx(fiat_db, transaction_db, asset, transaction, id_confirmer).await {
                Ok(transaction) => Ok(Json(transaction)),
                Err(e) => {
                    Err((
                        Status::BadRequest,
                        Json(ErrorResponse::new("Invalid transaction".to_string(), e)),
                    ))
                }
            }
        }
    }
}
#[post("/<id>/complete", format = "json")]
pub async fn complete_transaction(
    id: String,
    transaction_db: &State<Repository<Transaction>>,
    fiat_db: &State<Repository<Fiat>>,
    crypto_db: &State<Repository<Crypto>>,
    asset_master: &State<AssetManager>,
) -> Result<Json<Transaction>, (Status, Json<ErrorResponse>)> {
    let id_confirmer = "11111".to_string();
    let transaction = match transaction_db.get_by_id(&id).await {
        Ok(transaction) => transaction,
        Err(e) => {
            return Err((
                Status::BadRequest,
                Json(ErrorResponse::new("Invalid transaction".to_string(), e)),
            ))
        }
    };
    let asset = match asset_master.get_by_symbol(&transaction.asset) {
        Some(asset) => asset,
        None => {
            return Err((
                Status::BadRequest,
                Json(ErrorResponse::new(
                    "Invalid symbol".to_string(),
                    "Symbol is not supported".to_string(),
                )),
            ))
        }
    };
    match &asset.asset_type {
        AssetType::Crypto => {
            match complete_tx(crypto_db, transaction_db, asset, transaction, id_confirmer).await
            {
                Ok(transaction) => Ok(Json(transaction)),
                Err(e) => {
                    Err((
                        Status::BadRequest,
                        Json(ErrorResponse::new("Invalid transaction".to_string(), e)),
                    ))
                }
            }
        }
        AssetType::Fiat => {
            match complete_tx(fiat_db, transaction_db, asset, transaction, id_confirmer).await {
                Ok(transaction) => Ok(Json(transaction)),
                Err(e) => {
                    Err((
                        Status::BadRequest,
                        Json(ErrorResponse::new("Invalid transaction".to_string(), e)),
                    ))
                }
            }
        }
    }
}
#[post("/<id>/fail", format = "json")]
pub async fn fail_transaction(
    id: String,
    transaction_db: &State<Repository<Transaction>>,
    fiat_db: &State<Repository<Fiat>>,
    crypto_db: &State<Repository<Crypto>>,
    asset_master: &State<AssetManager>,
) -> Result<Json<Transaction>, (Status, Json<ErrorResponse>)> {
    let transaction = match transaction_db.get_by_id(&id).await {
        Ok(transaction) => transaction,
        Err(e) => {
            return Err((
                Status::BadRequest,
                Json(ErrorResponse::new("Invalid transaction".to_string(), e)),
            ))
        }
    };
    let asset = match asset_master.get_by_symbol(&transaction.asset) {
        Some(asset) => asset,
        None => {
            return Err((
                Status::BadRequest,
                Json(ErrorResponse::new(
                    "Invalid symbol".to_string(),
                    "Symbol is not supported".to_string(),
                )),
            ))
        }
    };
    match &asset.asset_type {
        AssetType::Crypto => {
            match fail_tx(crypto_db, transaction_db, asset, transaction).await
            {
                Ok(transaction) => Ok(Json(transaction)),
                Err(e) => {
                    Err((
                        Status::BadRequest,
                        Json(ErrorResponse::new("Invalid transaction".to_string(), e)),
                    ))
                }
            }
        }
        AssetType::Fiat => {
            match fail_tx(fiat_db, transaction_db, asset, transaction).await {
                Ok(transaction) => Ok(Json(transaction)),
                Err(e) => {
                    Err((
                        Status::BadRequest,
                        Json(ErrorResponse::new("Invalid transaction".to_string(), e)),
                    ))
                }
            }
        }
    }
}
#[post("/<id>/cancel", format = "json")]
pub async fn cancel_transaction(
    id: String,
    transaction_db: &State<Repository<Transaction>>,
    fiat_db: &State<Repository<Fiat>>,
    crypto_db: &State<Repository<Crypto>>,
    asset_master: &State<AssetManager>,
) -> Result<Json<Transaction>, (Status, Json<ErrorResponse>)> {
    let transaction = match transaction_db.get_by_id(&id).await {
        Ok(transaction) => transaction,
        Err(e) => {
            return Err((
                Status::BadRequest,
                Json(ErrorResponse::new("Invalid transaction".to_string(), e)),
            ))
        }
    };
    let asset = match asset_master.get_by_symbol(&transaction.asset) {
        Some(asset) => asset,
        None => {
            return Err((
                Status::BadRequest,
                Json(ErrorResponse::new(
                    "Invalid symbol".to_string(),
                    "Symbol is not supported".to_string(),
                )),
            ))
        }
    };
    match &asset.asset_type {
        AssetType::Crypto => {
            match cancel_tx(crypto_db, transaction_db, asset, transaction).await
            {
                Ok(transaction) => Ok(Json(transaction)),
                Err(e) => {
                    Err((
                        Status::BadRequest,
                        Json(ErrorResponse::new("Invalid transaction".to_string(), e)),
                    ))
                }
            }
        }
        AssetType::Fiat => {
            match cancel_tx(fiat_db, transaction_db, asset, transaction).await {
                Ok(transaction) => Ok(Json(transaction)),
                Err(e) => {
                    Err((
                        Status::BadRequest,
                        Json(ErrorResponse::new("Invalid transaction".to_string(), e)),
                    ))
                }
            }
        }
    }
}
async fn cancel_tx<T:Send+Sync+Clone+Serialize+Unpin+DeserializeOwned+Accounting+FungibleTradeable>(
    ledger_db: &Repository<T>,
    transaction_db: &State<Repository<Transaction>>,
    asset: Asset,
    mut transaction:Transaction) -> Result<Transaction, String> {
        let mut id_from = transaction.from_wallet.clone();
        let mut id_to = transaction.to_wallet.clone();
        id_from.push('_');
        id_from.push_str(&asset.symbol);
        id_to.push('_');
        id_to.push_str(&asset.symbol);
        match get_accounts(ledger_db, &id_from, &id_to).await {
            Ok((mut from, mut to)) => {
                from.confirm_deposit(transaction.total_amount)?;
                to.confirm_withdraw(transaction.total_amount)?;
                transaction.cancel_transaction()?;
                match transaction_db.update_by_id(&transaction.tx_id, transaction.clone()).await {
                    Ok(_) => Ok(transaction),
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(e),
        }
}

async fn fail_tx<T:Send+Sync+Clone+Serialize+Unpin+DeserializeOwned+Accounting+FungibleTradeable>(
    ledger_db: &Repository<T>,
    transaction_db: &State<Repository<Transaction>>,
    asset: Asset,
    mut transaction:Transaction) -> Result<Transaction, String> {
        let mut id_from = transaction.from_wallet.clone();
        let mut id_to = transaction.to_wallet.clone();
        id_from.push('_');
        id_from.push_str(&asset.symbol);
        id_to.push('_');
        id_to.push_str(&asset.symbol);
        match get_accounts(ledger_db, &id_from, &id_to).await {
            Ok((mut from, mut to)) => {
                from.confirm_deposit(transaction.total_amount)?;
                to.confirm_withdraw(transaction.total_amount)?;
                transaction.fail_transaction()?;
                match transaction_db.update_by_id(&transaction.tx_id, transaction.clone()).await {
                    Ok(_) => Ok(transaction),
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(e),
        }
}
async fn confirm_tx<T:Send+Sync+Clone+Serialize+Unpin+DeserializeOwned+Accounting+FungibleTradeable>(
    ledger_db: &Repository<T>,
    transaction_db: &State<Repository<Transaction>>,
    asset: Asset,
    mut transaction:Transaction,
    id_confirmer:String) -> Result<Transaction, String> {
        let mut id_from = transaction.from_wallet.clone();
        let mut id_to = transaction.to_wallet.clone();
        id_from.push('_');
        id_from.push_str(&asset.symbol);
        id_to.push('_');
        id_to.push_str(&asset.symbol);
        match get_accounts(ledger_db, &id_from, &id_to).await {
            Ok(_) => {
                transaction.confirm_transaction(id_confirmer)?;
                match transaction_db.update_by_id(&transaction.tx_id, transaction.clone()).await {
                    Ok(_) => Ok(transaction),
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(e),
        }
}
async fn complete_tx<T:Send+Sync+Clone+Serialize+Unpin+DeserializeOwned+Accounting+FungibleTradeable>(
    ledger_db: &Repository<T>,
    transaction_db: &State<Repository<Transaction>>,
    asset: Asset,
    mut transaction:Transaction,
    id_confirmer:String) -> Result<Transaction, String> {
        let mut id_from = transaction.from_wallet.clone();
        let mut id_to = transaction.to_wallet.clone();
        id_from.push('_');
        id_from.push_str(&asset.symbol);
        id_to.push('_');
        id_to.push_str(&asset.symbol);
        match get_accounts(ledger_db, &id_from, &id_to).await {
            Ok((mut from, mut to)) => {
                from.confirm_withdraw(transaction.total_amount)?;
                to.confirm_deposit(transaction.amount)?;
                ledger_db.update_by_id(&id_from,from).await?;
                ledger_db.update_by_id(&id_to, to).await?;
                transaction.complete_transaction(id_confirmer)?;
                match transaction_db.update_by_id(&transaction.tx_id, transaction.clone()).await {
                    Ok(_) => Ok(transaction),
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(e),
        }
}
async fn process_tx<T:Send+Sync+Clone+Serialize+Unpin+DeserializeOwned+Accounting+FungibleTradeable>(
    id_from: &str,
    id_to: &str,
    ledger_db: &Repository<T>,
    transaction_db: &State<Repository<Transaction>>,
    asset: Asset,
    req: TransactionRequest,
) -> Result<Transaction, String> {
    match get_accounts(ledger_db, id_from, id_to).await {
        Ok((mut from, mut to)) => {
            let mut transaction = Transaction::new(
                TransactionType::Transfer,
                asset.symbol,
                req.amount,
                from.get_account_number(),
                to.get_account_number(),
                "Basic Transfer".to_string(),
                1,
            );
            let source_amount  = req.amount*0.01;
            transaction.add_fee("Tx Fee".to_string(), source_amount);
            from.withdraw(transaction.total_amount)?;
            to.deposit(transaction.amount)?;
            ledger_db.update_by_id(id_from,from).await?;
            ledger_db.update_by_id(id_to, to).await?;
            match transaction_db.create(transaction.clone()).await {
                Ok(_) => Ok(transaction),
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e),
    }
}
async fn get_accounts<T:Send+Sync+Clone+Serialize+Unpin+DeserializeOwned>(
    ledger_db: &Repository<T>,
    id_from: &str,
    id_to: &str,
) -> Result<(T, T), String> {
    let from_result = ledger_db.get_by_id(id_from).await;
    let to_result = ledger_db.get_by_id(id_to).await;
    let from = match from_result {
        Ok(from) => from,
        Err(e) => return Err(e),
    };
    let to = match to_result {
        Ok(to) => to,
        Err(e) => return Err(e),
    };
    Ok((from, to))
}
