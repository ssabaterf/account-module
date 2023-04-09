use api::{account::{create_account, get_accounts, get_account, enable_account, disable_account, get_fiats, get_cryptos, balances}, fiat::{create_fiat, get_fiat, fiat_deposit, fiat_confirm_deposit, fiat_withdrawal, fiat_release_withdrawal}, crypto::{get_crypto, create_crypto, crypto_deposit, crypto_confirm_deposit, crypto_withdrawal, crypto_release_withdrawal}, transaction::{submit_transaction, confirm_transaction, complete_transaction, fail_transaction, cancel_transaction}};
use chrono::Local;
use domain::{ledger::{Fiat, Crypto}, account::Account, transaction::Transaction, asset::AssetManager};
use mongo::Data;
use response::error::ErrorResponse;
use rocket::{Request, launch, http::Method, catchers, routes, catch, serde::json::Json};
use dotenv::dotenv;
use rocket_cors::{CorsOptions, AllowedOrigins};
use serde::{Serialize};
use std::env;
mod mongo;
mod domain;
mod response;
mod api;
mod dto;

#[launch]
async fn rocket() -> _ {
    dotenv().ok();
    
    let db_uri = match env::var("DBURI") {
        Ok(v) => {
            println!("DBURI: {}", &v);
            v},
        Err(_) => panic!("Error loading env variable: DBURI"),
    };
    let db_name = match env::var("DBNAME") {
        Ok(v) => {
            println!("DBNAME: {}", &v);
            v},
        Err(_) => panic!("Error loading env variable: DBNAME"),
    };
    
    let client = match Data::new(&db_uri, "My Bank", &db_name).await {
        Ok(client) => client,
        Err(e) => {
            panic!("Error creating client: {}", e);
        }
    };
    let fiat_db = client.get_repo::<Fiat>("fiat_vault", "id".to_string()).unwrap();
    let crypto_db = client.get_repo::<Crypto>("crypto_vault", "id".to_string()).unwrap();
    let wallet_db = client.get_repo::<Account>("wallet", "account_number".to_string()).unwrap();
    let transaction_db = client.get_repo::<Transaction>("transaction", "tx_id".to_string()).unwrap();
    let asset_manager = AssetManager::new();
    let cors = CorsOptions::default()
    .allowed_origins(AllowedOrigins::all())
    .allowed_methods(
        vec![Method::Get, Method::Post, Method::Patch, Method::Delete]
            .into_iter()
            .map(From::from)
            .collect(),
    )
    .allow_credentials(true);

    rocket::build()
    .manage(cors.to_cors())
    .manage(fiat_db)
    .manage(crypto_db)
    .manage(asset_manager)
    .manage(wallet_db)
    .manage(transaction_db)
    .mount("/v1/accounts", routes![create_account, get_accounts, get_account, disable_account, enable_account,get_fiats, get_cryptos, balances])
    .mount("/v1/fiats", routes![create_fiat, get_fiat, fiat_deposit, fiat_confirm_deposit, fiat_withdrawal, fiat_release_withdrawal])
    .mount("/v1/cryptos", routes![create_crypto, get_crypto, crypto_deposit, crypto_confirm_deposit, crypto_withdrawal, crypto_release_withdrawal])
    .mount("/v1/transactions", routes![submit_transaction, confirm_transaction, complete_transaction, fail_transaction, cancel_transaction])
    .register(
        "/",
        catchers![unauthorized, not_found, internal_sever_error, bad_format],
    )
}

#[catch(401)]
pub fn unauthorized() -> Json<ErrorResponse> { 
    Json(ErrorResponse { cause: "UNAUTHORIZED".to_string(), 
    message: "Endpoint call without valid token".to_string(),
    date: Local::now().format("%Y-%m-%d %H:%M:%S").to_string() })
}

#[catch(404)]
pub fn not_found(_req: &Request) -> Json<ErrorResponse> {
    println!("{}",_req);
    let content_type = _req.headers().get_one("content-type").unwrap_or("unknown/type");
    let mut message = "Endpoint not found ".to_string();
    message.push_str(content_type);
    Json(ErrorResponse { cause: "NOT FOUND".to_string(), 
    message,
    date: Local::now().format("%Y-%m-%d %H:%M:%S").to_string() })
}
#[catch(422)]
pub fn bad_format(_req: &Request) -> Json<ErrorResponse> {
    let mut message = "Bad formated body in the request ".to_string();
    message.push_str(_req.uri().path().as_str());
    Json(ErrorResponse { cause: "BAD FORMAT".to_string(), 
    message,
    date: Local::now().format("%Y-%m-%d %H:%M:%S").to_string() })
}
#[catch(500)]
pub fn internal_sever_error() -> Json<ErrorResponse> {
    Json(ErrorResponse { cause: "INTERNAL SERVER ERROR".to_string(), 
    message: "Something went wrong :(".to_string(),
    date: Local::now().format("%Y-%m-%d %H:%M:%S").to_string() })
}

#[derive(Debug, Serialize)]
struct ErrorDetails {
    field: String,
    message: String,
}

#[derive(Debug, Serialize)]
struct UnprocessableEntityError {
    error: Vec<ErrorDetails>,
}