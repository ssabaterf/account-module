use revolt_rocket_okapi::openapi;
use rocket::{State, http::Status, serde::json::Json, post, get};

use crate::{domain::{account::Account, ledger::{Crypto, FungibleTradeable}, asset::AssetManager, transaction::Transaction}, mongo::{Repository, Crud}, response::error::ErrorResponse, dto::deposit::{Deposit, DepositCreation, DepositConfirmation, Withdrawal, WithdrawalCreation, WithdrawalConfirmation}};

#[openapi(tag = "Fiats")]
#[post("/cryptos/<id>/ledgers/<symbol>", format = "json")]
pub async fn create_crypto(
    id: String,
    symbol: String,
    account_db: &State<Repository<Account>>,
    crypto_db: &State<Repository<Crypto>>,
    asset_master: &State<AssetManager>,
) -> Result<Json<Crypto>, (Status, Json<ErrorResponse>)> {
    let asset = match asset_master.get_by_symbol(&symbol){
        Some(asset) => asset,
        None => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), "Asset not found".to_string())))),
    };
    let mut account = match account_db.get_by_id(&id).await {
        Ok(account) => account,
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e)))),
    };
    let account = account.add_crypto(asset.clone());
    let crypto = match Crypto::new(account.clone().account_number, asset, "network".to_string(), "address".to_string()){
        Ok(crypto) => crypto,
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e)))),
    };
    match account_db.update_by_id(&account.account_number, account.clone()).await {
        Ok(_) => (),
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e)))),
    };
    match crypto_db.create(crypto.clone()).await {
        Ok(_) => (),
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e)))),
    };
    Ok(Json(crypto))
}

#[openapi(tag = "Fiats")]
#[get("/cryptos/<id>/ledgers/<symbol>")]
pub async fn get_crypto(
    id: String,
    symbol: String,
    account_db: &State<Repository<Account>>,
    crypto_db: &State<Repository<Crypto>>,
) -> Result<Json<Vec<Crypto>>, (Status, Json<ErrorResponse>)> {
    let account = match account_db.get_by_id(&id).await {
        Ok(account) => account,
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e)))),
    };
    if !account.accounts_crypto.contains_key(&symbol){
        return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), "Crypto not found".to_string()))));
    };
    let crypto = match crypto_db.get_by_fields(vec!["account_number".to_string(), "asset.symbol".to_string()],vec![id,symbol]).await {
        Ok(crypto) => crypto,
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e)))),
    };
    Ok(Json(crypto))
}

#[openapi(tag = "Fiats")]
#[post("/cryptos/<id>/deposit", format = "json", data = "<deposit>")]
pub async fn crypto_deposit(
    id: String,
    deposit: Json<Deposit>,
    account_db: &State<Repository<Account>>,
    transaction_db: &State<Repository<Transaction>>,
    crypto_db: &State<Repository<Crypto>>,
) -> Result<Json<DepositCreation<Crypto>>,  (Status, Json<ErrorResponse>)> {
    match account_db.get_by_id(&id).await {
        Ok(account) => account,
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e)))),
    };
    let mut id_ledger = id.clone();
    id_ledger.push('_');
    id_ledger.push_str(&deposit.symbol);
    let mut crypto = match crypto_db.get_by_id(&id_ledger).await {
        Ok(crypto) => crypto,
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e)))),
    };
    let tx = Transaction::new_deposit(deposit.symbol.clone(), deposit.amount, id.clone(), 1);
    match crypto.deposit(deposit.amount){
        Ok(_) => (),
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e)))),
    };
    match crypto_db.update_by_id(&crypto.id, crypto.clone()).await {
        Ok(_) => (),
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e)))),
    };
    match transaction_db.create(tx.clone()).await {
        Ok(tx) => tx,
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e)))),
    };
    Ok(Json(DepositCreation{account: crypto, tx_id: tx.tx_id}))
}

#[openapi(tag = "Fiats")]
#[post("/cryptos/<id>/deposit/<tx_id>/confirm", format = "json", data = "<confirmation>")]
pub async fn crypto_confirm_deposit(
    id: String,
    tx_id: String,
    confirmation: Json<DepositConfirmation>,
    account_db: &State<Repository<Account>>,
    transaction_db: &State<Repository<Transaction>>,
    crypto_db: &State<Repository<Crypto>>,
) -> Result<Json<Crypto>, (Status, Json<ErrorResponse>)> {
    match account_db.get_by_id(&id).await {
        Ok(account) => account,
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e)))),
    };
    let mut tx = match transaction_db.get_by_id(&tx_id).await{
        Ok(tx) => tx,
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e)))),
    };
    let mut id_ledger = id.clone();
    id_ledger.push('_');
    id_ledger.push_str(&tx.asset);
    let mut crypto = match crypto_db.get_by_id(&id_ledger).await {
        Ok(crypto) => crypto,
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e)))),
    };
    match crypto.confirm_deposit(tx.amount){
        Ok(_) => (),
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e)))),
    };
    match tx.confirm_transaction(id){
        Ok(_) => (),
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e)))),
    };
    match tx.complete_transaction(confirmation.external_id.clone()){
        Ok(_) => (),
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e)))),
    };
    match crypto_db.update_by_id(&crypto.id, crypto.clone()).await {
        Ok(_) => (),
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e)))),
    };
    Ok(Json(crypto))
}

#[openapi(tag = "Fiats")]
#[post("/cryptos/<id>/withdrawal", format = "json", data = "<withdrawal>")]
pub async fn crypto_withdrawal(
    id: String,
    withdrawal: Json<Withdrawal>,
    transaction_db: &State<Repository<Transaction>>,
    account_db: &State<Repository<Account>>,
    crypto_db: &State<Repository<Crypto>>,
) -> Result<Json<WithdrawalCreation<Crypto>>, (Status, Json<ErrorResponse>)> {
    match account_db.get_by_id(&id).await {
        Ok(account) => account,
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e)))),
    };
    let mut id_ledger = id.clone();
    id_ledger.push('_');
    id_ledger.push_str(&withdrawal.symbol);
    let mut crypto = match crypto_db.get_by_id(&id_ledger).await {
        Ok(crypto) => crypto,
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e)))),
    };
    let mut tx = Transaction::new_withdraw(withdrawal.symbol.clone(), withdrawal.amount, id.clone(), 1);
    tx.add_fee("Withdrawal".to_string(), tx.amount * 0.01);
    match crypto.withdraw(tx.total_amount){
        Ok(_) => (),
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e)))),
    };
    match crypto_db.update_by_id(&crypto.id, crypto.clone()).await {
        Ok(_) => (),
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e)))),
    };
    match transaction_db.create(tx.clone()).await {
        Ok(tx) => tx,
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e)))),
    };
    Ok(Json(WithdrawalCreation{account: crypto, tx_id: tx.tx_id}))
}

#[openapi(tag = "Fiats")]
#[post("/cryptos/<id>/withdraw/<tx_id>/release", format = "json", data = "<confirmation>")]
pub async fn crypto_release_withdrawal(
    id: String,
    tx_id: String,
    confirmation: Json<WithdrawalConfirmation>,
    account_db: &State<Repository<Account>>,
    transaction_db: &State<Repository<Transaction>>,
    crypto_db: &State<Repository<Crypto>>,
) -> Result<Json<Crypto>, (Status, Json<ErrorResponse>)> {
    match account_db.get_by_id(&id).await {
        Ok(account) => account,
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e)))),
    };
    let mut tx = match transaction_db.get_by_id(&tx_id).await {
        Ok(tx) => tx,
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e)))),
    };
    let mut id_ledger = id.clone();
    id_ledger.push('_');
    id_ledger.push_str(&tx.asset);
    let mut crypto = match crypto_db.get_by_id(&id_ledger).await {
        Ok(crypto) => crypto,
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e)))),
    };
    match crypto.confirm_withdraw(tx.total_amount){
        Ok(_) => (),
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e)))),
    };
    match tx.confirm_transaction(id){
        Ok(tx) => tx,
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e)))),
    };
    match tx.complete_transaction(confirmation.external_id.clone()){
        Ok(tx) => tx,
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e)))),
    };
    match crypto_db.update_by_id(&crypto.id, crypto.clone()).await {
        Ok(_) => (),
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e)))),
    };
    match transaction_db.update_by_id(&tx.tx_id.clone(), tx).await {
        Ok(_) => (),
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e)))),
    };
    Ok(Json(crypto))
}