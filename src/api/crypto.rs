use rocket::{State, http::Status, serde::json::Json, post, get};

use crate::{mongo::{Crud, Repository}, domain::{account::{account::Account}, ledger::ledger::{Crypto, FungibleTradeable}, 
asset::asset::AssetManager}, response::error::ErrorResponse, dto::deposit::Deposit};

#[post("/<id>/ledgers/<symbol>", format = "json")]
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
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e.to_string())))),
    };
    let account = account.add_crypto(asset.clone());
    let crypto = match Crypto::new(account.clone().account_number, asset, "network".to_string(), "address".to_string()){
        Ok(crypto) => crypto,
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e.to_string())))),
    };
    match account_db.update_by_id(&account.account_number, account.clone()).await {
        Ok(_) => (),
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e.to_string())))),
    };
    match crypto_db.create(crypto.clone()).await {
        Ok(_) => (),
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e.to_string())))),
    };
    Ok(Json(crypto))
}

#[get("/<id>/ledgers/<symbol>")]
pub async fn get_crypto(
    id: String,
    symbol: String,
    account_db: &State<Repository<Account>>,
    crypto_db: &State<Repository<Crypto>>,
) -> Result<Json<Vec<Crypto>>, (Status, Json<ErrorResponse>)> {
    let account = match account_db.get_by_id(&id).await {
        Ok(account) => account,
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e.to_string())))),
    };
    if !account.accounts_crypto.contains_key(&symbol){
        return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), "Crypto not found".to_string()))));
    };
    let crypto = match crypto_db.get_by_fields(vec!["account_number".to_string(), "asset.symbol".to_string()],vec![id,symbol]).await {
        Ok(crypto) => crypto,
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e.to_string())))),
    };
    Ok(Json(crypto))
}

#[post("/<id>/deposit", format = "json", data = "<deposit>")]
pub async fn crypto_deposit(
    id: String,
    deposit: Json<Deposit>,
    account_db: &State<Repository<Account>>,
    crypto_db: &State<Repository<Crypto>>,
) -> Result<Json<Crypto>, (Status, Json<ErrorResponse>)> {
    match account_db.get_by_id(&id).await {
        Ok(account) => account,
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e.to_string())))),
    };
    let mut crypto = match crypto_db.get_by_id(&id).await {
        Ok(crypto) => crypto,
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e.to_string())))),
    };
    match crypto.deposit(deposit.amount){
        Ok(_) => (),
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e.to_string())))),
    };

    match crypto_db.update_by_id(&crypto.id, crypto.clone()).await {
        Ok(_) => (),
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e.to_string())))),
    };
    Ok(Json(crypto))
}

#[post("/<id>/confirm", format = "json", data = "<deposit>")]
pub async fn crypto_confirm_deposit(
    id: String,
    deposit: Json<Deposit>,
    account_db: &State<Repository<Account>>,
    crypto_db: &State<Repository<Crypto>>,
) -> Result<Json<Crypto>, (Status, Json<ErrorResponse>)> {
    match account_db.get_by_id(&id).await {
        Ok(account) => account,
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e.to_string())))),
    };
    let mut crypto = match crypto_db.get_by_id(&id).await {
        Ok(crypto) => crypto,
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e.to_string())))),
    };
    match crypto.confirm_deposit(deposit.amount){
        Ok(_) => (),
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e.to_string())))),
    };

    match crypto_db.update_by_id(&crypto.id, crypto.clone()).await {
        Ok(_) => (),
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e.to_string())))),
    };
    Ok(Json(crypto))
}

#[post("/<id>/withdrawal", format = "json", data = "<deposit>")]
pub async fn crypto_withdrawal(
    id: String,
    deposit: Json<Deposit>,
    account_db: &State<Repository<Account>>,
    crypto_db: &State<Repository<Crypto>>,
) -> Result<Json<Crypto>, (Status, Json<ErrorResponse>)> {
    match account_db.get_by_id(&id).await {
        Ok(account) => account,
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e.to_string())))),
    };
    let mut crypto = match crypto_db.get_by_id(&id).await {
        Ok(crypto) => crypto,
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e.to_string())))),
    };
    match crypto.withdraw(deposit.amount){
        Ok(_) => (),
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e.to_string())))),
    };

    match crypto_db.update_by_id(&crypto.id, crypto.clone()).await {
        Ok(_) => (),
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e.to_string())))),
    };
    Ok(Json(crypto))
}

#[post("/<id>/release", format = "json", data = "<deposit>")]
pub async fn crypto_release_withdrawal(
    id: String,
    deposit: Json<Deposit>,
    account_db: &State<Repository<Account>>,
    crypto_db: &State<Repository<Crypto>>,
) -> Result<Json<Crypto>, (Status, Json<ErrorResponse>)> {
    match account_db.get_by_id(&id).await {
        Ok(account) => account,
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e.to_string())))),
    };
    let mut crypto = match crypto_db.get_by_id(&id).await {
        Ok(crypto) => crypto,
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e.to_string())))),
    };
    match crypto.confirm_withdraw(deposit.amount){
        Ok(_) => (),
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e.to_string())))),
    };

    match crypto_db.update_by_id(&crypto.id, crypto.clone()).await {
        Ok(_) => (),
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e.to_string())))),
    };
    Ok(Json(crypto))
}