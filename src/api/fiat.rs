use rocket::{State, http::Status, serde::json::Json, post, get};

use crate::{domain::{account::Account, ledger::{Fiat, FungibleTradeable}, asset::AssetManager}, mongo::{Repository, Crud}, response::error::ErrorResponse, dto::deposit::Deposit};


#[post("/<id>/ledgers/<symbol>", format = "json")]
pub async fn create_fiat(
    id: String,
    symbol: String,
    account_db: &State<Repository<Account>>,
    fiat_db: &State<Repository<Fiat>>,
    asset_master: &State<AssetManager>,
) -> Result<Json<Fiat>, (Status, Json<ErrorResponse>)> {
    let asset = match asset_master.get_by_symbol(&symbol){
        Some(asset) => asset,
        None => return Err((Status::BadRequest, Json(ErrorResponse::new("Fiat".to_string(), "Asset not found".to_string())))),
    };
    let mut account = match account_db.get_by_id(&id).await {
        Ok(account) => account,
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Fiat".to_string(), e)))),
    };
    let account = account.add_fiat(asset.clone());
    let fiat = match Fiat::new(account.clone().account_number, asset){
        Ok(fiat) => fiat,
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Fiat".to_string(), e)))),
    };
    match account_db.update_by_id(&account.account_number, account.clone()).await {
        Ok(_) => (),
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Fiat".to_string(), e)))),
    };
    match fiat_db.create(fiat.clone()).await {
        Ok(_) => (),
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Fiat".to_string(), e)))),
    };
    Ok(Json(fiat))
}

#[get("/<id>/ledgers/<symbol>")]
pub async fn get_fiat(
    id: String,
    symbol: String,
    account_db: &State<Repository<Account>>,
    fiat_db: &State<Repository<Fiat>>,
) -> Result<Json<Vec<Fiat>>, (Status, Json<ErrorResponse>)> {
    let account = match account_db.get_by_id(&id).await {
        Ok(account) => account,
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Fiat".to_string(), e)))),
    };
    if !account.accounts_fiat.contains_key(&symbol){
        return Err((Status::BadRequest, Json(ErrorResponse::new("Fiat".to_string(), "Fiat not found".to_string()))));
    };
    let fiat = match fiat_db.get_by_fields(vec!["account_number".to_string(), "asset.symbol".to_string()],vec![id,symbol]).await {
        Ok(fiat) => fiat,
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Fiat".to_string(), e)))),
    };
    Ok(Json(fiat))
}

#[post("/<id>/deposit", format = "json", data = "<deposit>")]
pub async fn fiat_deposit(
    id: String,
    deposit: Json<Deposit>,
    account_db: &State<Repository<Account>>,
    fiat_db: &State<Repository<Fiat>>,
) -> Result<Json<Fiat>, (Status, Json<ErrorResponse>)> {
    match account_db.get_by_id(&id).await {
        Ok(account) => account,
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Fiat".to_string(), e)))),
    };
    let mut fiat = match fiat_db.get_by_id(&id).await {
        Ok(fiat) => fiat,
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Fiat".to_string(), e)))),
    };
    match fiat.deposit(deposit.amount){
        Ok(_) => (),
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Fiat".to_string(), e)))),
    };

    match fiat_db.update_by_id(&fiat.id, fiat.clone()).await {
        Ok(_) => (),
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Fiat".to_string(), e)))),
    };
    Ok(Json(fiat))
}

#[post("/<id>/confirm", format = "json", data = "<deposit>")]
pub async fn fiat_confirm_deposit(
    id: String,
    deposit: Json<Deposit>,
    account_db: &State<Repository<Account>>,
    fiat_db: &State<Repository<Fiat>>,
) -> Result<Json<Fiat>, (Status, Json<ErrorResponse>)> {
    match account_db.get_by_id(&id).await {
        Ok(account) => account,
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Fiat".to_string(), e)))),
    };
    let mut fiat = match fiat_db.get_by_id(&id).await {
        Ok(fiat) => fiat,
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Fiat".to_string(), e)))),
    };
    match fiat.confirm_deposit(deposit.amount){
        Ok(_) => (),
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Fiat".to_string(), e)))),
    };

    match fiat_db.update_by_id(&fiat.id, fiat.clone()).await {
        Ok(_) => (),
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Fiat".to_string(), e)))),
    };
    Ok(Json(fiat))
}

#[post("/<id>/withdrawal", format = "json", data = "<deposit>")]
pub async fn fiat_withdrawal(
    id: String,
    deposit: Json<Deposit>,
    account_db: &State<Repository<Account>>,
    fiat_db: &State<Repository<Fiat>>,
) -> Result<Json<Fiat>, (Status, Json<ErrorResponse>)> {
    match account_db.get_by_id(&id).await {
        Ok(account) => account,
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Fiat".to_string(), e)))),
    };
    let mut fiat = match fiat_db.get_by_id(&id).await {
        Ok(fiat) => fiat,
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Fiat".to_string(), e)))),
    };
    match fiat.withdraw(deposit.amount){
        Ok(_) => (),
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Fiat".to_string(), e)))),
    };

    match fiat_db.update_by_id(&fiat.id, fiat.clone()).await {
        Ok(_) => (),
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Fiat".to_string(), e)))),
    };
    Ok(Json(fiat))
}

#[post("/<id>/release", format = "json", data = "<deposit>")]
pub async fn fiat_release_withdrawal(
    id: String,
    deposit: Json<Deposit>,
    account_db: &State<Repository<Account>>,
    fiat_db: &State<Repository<Fiat>>,
) -> Result<Json<Fiat>, (Status, Json<ErrorResponse>)> {
    match account_db.get_by_id(&id).await {
        Ok(account) => account,
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Fiat".to_string(), e)))),
    };
    let mut fiat = match fiat_db.get_by_id(&id).await {
        Ok(fiat) => fiat,
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Fiat".to_string(), e)))),
    };
    match fiat.confirm_withdraw(deposit.amount){
        Ok(_) => (),
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Fiat".to_string(), e)))),
    };

    match fiat_db.update_by_id(&fiat.id, fiat.clone()).await {
        Ok(_) => (),
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Fiat".to_string(), e)))),
    };
    Ok(Json(fiat))
}