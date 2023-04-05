use std::collections::HashMap;

use rocket::{post, State, serde::json::Json, http::Status, get};

use crate::{domain::{account::{Account, Balance}, ledger::{Fiat, Crypto}, asset::AssetManager}, mongo::{Repository, Crud}, response::{error::ErrorResponse, custom::Pagination}};

#[post("/", format = "json")]
pub async fn create_account(
    account_db: &State<Repository<Account>>,
    fiat_db: &State<Repository<Fiat>>,
    crypto_db: &State<Repository<Crypto>>,
    asset_master: &State<AssetManager>,
) -> Result<Json<Account>, (Status, Json<ErrorResponse>)> {
    let mut account = match Account::init(asset_master, &vec!["USD".to_string(), "BTC".to_string(), "EUR".to_string()]) {
        Ok(account) => account,
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Account".to_string(), e)))),
    };
    
    match account_db.create(account.clone()).await {
        Ok(_) => (),
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Account".to_string(), e)))),
    }

    for key in account.accounts_fiat.keys() {
        let asset = match asset_master.get_by_symbol(key){
            Some(symbol) => symbol,
            None => return Err((Status::BadRequest, Json(ErrorResponse::new("Account".to_string(), "Asset not found".to_string())))),
        };
        let fiat_result = Fiat::new(account.account_number.clone(), asset);
        let fiat = match fiat_result {
            Ok(fiat) => fiat,
            Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Account".to_string(), e)))),
        };
        match fiat_db.create(fiat).await {
            Ok(_) => (),
            Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Account".to_string(), e)))),
        }
    }

    for key in account.accounts_crypto.keys() {
        let asset = match asset_master.get_by_symbol(key){
            Some(symbol) => symbol,
            None => return Err((Status::BadRequest, Json(ErrorResponse::new("Account".to_string(), "Asset not found".to_string())))),
        };
        let crypto_result = Crypto::new(account.account_number.clone(), asset, "network".to_string(),"address".to_string());
        let crypto = match crypto_result {
            Ok(crypto) => crypto,
            Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Account".to_string(), e)))),
        };
        match crypto_db.create(crypto).await {
            Ok(_) => (),
            Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Account".to_string(), e)))),
        }
    }

    account.active = true;
    match account_db.update_by_id(&account.account_number, account.clone()).await {
        Ok(_) => (),
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Account".to_string(), e)))),
    }
    Ok(Json(account))
}

#[get("/?<skip>&<limit>", format = "json")]
pub async fn get_accounts(
    account_db: &State<Repository<Account>>,
    skip: Option<usize>,
    limit: Option<usize>,
) -> Result<Json<Pagination<Account>>, (Status, Json<ErrorResponse>)> {
    let skip_value = skip.unwrap_or(0);
    let limit_value = limit.unwrap_or(10);
    let accounts = match account_db.get_all(skip_value, limit_value).await {
        Ok(accounts) => accounts,
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Account".to_string(), e)))),
    };
    let pagination = Pagination{
        skip: skip_value as u64,
        limit: limit_value as u64,
        count: account_db.count().await,
        result: accounts,
    };
    Ok(Json(pagination))
}

#[get("/<id>", format = "json")]
pub async fn get_account(
    account_db: &State<Repository<Account>>,
    id: String,
) -> Result<Json<Account>, (Status, Json<ErrorResponse>)> {
    let account = match account_db.get_by_id(&id).await {
        Ok(account) => account,
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Account".to_string(), e)))),
    };
    Ok(Json(account))
}

#[get("/<id>/disable", format = "json")]
pub async fn disable_account(
    account_db: &State<Repository<Account>>,
    id: String,
) -> Result<Json<Account>, (Status, Json<ErrorResponse>)> {
    let mut account = match account_db.get_by_id(&id).await {
        Ok(account) => account,
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Account".to_string(), e)))),
    };
    account.active = false;
    match account_db.update_by_id(&id, account.clone()).await {
        Ok(_) => (),
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Account".to_string(), e)))),
    };
    Ok(Json(account))
}

#[get("/<id>/enable", format = "json")]
pub async fn enable_account(
    account_db: &State<Repository<Account>>,
    id: String,
) -> Result<Json<Account>, (Status, Json<ErrorResponse>)> {
    let mut account = match account_db.get_by_id(&id).await {
        Ok(account) => account,
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Account".to_string(), e)))),
    };
    account.active = true;
    match account_db.update_by_id(&id, account.clone()).await {
        Ok(_) => (),
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Account".to_string(), e)))),
    };
    Ok(Json(account))
}

#[get("/<id>/fiats", format = "json")]
pub async fn get_fiats(
    id: String,
    fiat_db: &State<Repository<Fiat>>,
) -> Result<Json<Vec<Fiat>>, (Status, Json<ErrorResponse>)> {
    match fiat_db.get_by_fields(vec!["account_number".to_string()],vec![id]).await {
        Ok(fiats) => Ok(Json(fiats)),
        Err(e) => Err((Status::BadRequest, Json(ErrorResponse::new("Fiat".to_string(), e)))),
    }
}

#[get("/<id>/cryptos", format = "json")]
pub async fn get_cryptos(
    id: String,
    crypto_db: &State<Repository<Crypto>>,
) -> Result<Json<Vec<Crypto>>, (Status, Json<ErrorResponse>)> {
    match crypto_db.get_by_fields(vec!["account_number".to_string()],vec![id]).await {
        Ok(fiats) => Ok(Json(fiats)),
        Err(e) => Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e)))),
    }
}

#[get("/<id>/balances", format = "json")]
pub async fn balances(
    id: String,
    crypto_db: &State<Repository<Crypto>>,
    fiat_db: &State<Repository<Fiat>>,
    account_db: &State<Repository<Account>>,
) -> Result<Json<HashMap<String,Balance>>, (Status, Json<ErrorResponse>)> {
    match account_db.get_by_id(&id).await {
        Ok(account) => account,
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Account".to_string(), e)))),
    };

    let cryptos= match crypto_db.get_by_fields(vec!["account_number".to_string()],vec![id.clone()]).await {
        Ok(cryptos) => cryptos,
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Crypto".to_string(), e)))),
    };

    let fiats= match fiat_db.get_by_fields(vec!["account_number".to_string()],vec![id]).await {
        Ok(fiats) => fiats,
        Err(e) => return Err((Status::BadRequest, Json(ErrorResponse::new("Fiat".to_string(), e)))),
    };

    Ok(Json(Account::balance(fiats, cryptos)))
}