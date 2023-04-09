use api::{account::*, crypto::*, fiat::*, transaction::*, auth::*};
use chrono::Local;
use domain::{
    account::Account,
    asset::AssetManager,
    ledger::{Crypto, Fiat},
    transaction::Transaction, user::User,
};
use dotenv::dotenv;
use mongo::Data;
use response::error::ErrorResponse;
use revolt_rocket_okapi::{
    openapi_get_routes,
    rapidoc::{make_rapidoc, GeneralConfig, HideShowConfig, RapiDocConfig},
    settings::UrlObject,
    swagger_ui::{make_swagger_ui, SwaggerUIConfig},
};
use rocket::{catch, catchers, http::Method, launch, serde::json::Json, Request};
use rocket_cors::{AllowedOrigins, CorsOptions};
use serde::Serialize;
use std::env;
mod api;
mod domain;
mod dto;
mod mongo;
mod response;
mod fairings;
mod security;

#[launch]
async fn rocket() -> _ {
    dotenv().ok();

    let db_uri = match env::var("DBURI") {
        Ok(v) => {
            println!("DBURI: {}", &v);
            v
        }
        Err(_) => panic!("Error loading env variable: DBURI"),
    };
    let db_name = match env::var("DBNAME") {
        Ok(v) => {
            println!("DBNAME: {}", &v);
            v
        }
        Err(_) => panic!("Error loading env variable: DBNAME"),
    };

    let client = match Data::new(&db_uri, "My Bank", &db_name).await {
        Ok(client) => client,
        Err(e) => {
            panic!("Error creating client: {}", e);
        }
    };
    let fiat_db = client
        .get_repo::<Fiat>("fiat_vault", "id".to_string())
        .unwrap();
    let crypto_db = client
        .get_repo::<Crypto>("crypto_vault", "id".to_string())
        .unwrap();
    let wallet_db = client
        .get_repo::<Account>("wallet", "account_number".to_string())
        .unwrap();
    let transaction_db = client
        .get_repo::<Transaction>("transaction", "tx_id".to_string())
        .unwrap();
    let user_db = client
        .get_repo::<User>("user", "id".to_string())
        .unwrap();
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

    let unique_v1_api = openapi_get_routes![
        register,
        login,
        refresh_tokens,

        create_account,
        get_accounts,
        get_account,
        disable_account,
        enable_account,
        get_fiats,
        get_cryptos,
        balances,

        create_crypto,
        get_crypto,
        crypto_deposit,
        crypto_confirm_deposit,
        crypto_withdrawal,
        crypto_release_withdrawal,

        create_fiat,
        get_fiat,
        fiat_deposit,
        fiat_confirm_deposit,
        fiat_withdrawal,
        fiat_release_withdrawal,

        submit_transaction,
        confirm_transaction,
        complete_transaction,
        fail_transaction,
        cancel_transaction
    ];
    
    rocket::build()
        .manage(cors.to_cors())
        .manage(fiat_db)
        .manage(crypto_db)
        .manage(asset_manager)
        .manage(wallet_db)
        .manage(transaction_db)
        .manage(user_db)
        .mount(
            "/v1", unique_v1_api
        )
        .mount(
            "/swagger-ui/",
            make_swagger_ui(&SwaggerUIConfig {
                url: "../v1/openapi.json".to_owned(),
                ..Default::default()
            }),
        )
        .mount(
            "/rapidoc/",
            make_rapidoc(&RapiDocConfig {
                general: GeneralConfig {
                    spec_urls: vec![UrlObject::new("General", "../v1/openapi.json")],
                    ..Default::default()
                },
                hide_show: HideShowConfig {
                    allow_spec_url_load: false,
                    allow_spec_file_load: false,
                    ..Default::default()
                },
                ..Default::default()
            }),
        )
        .register(
            "/",
            catchers![unauthorized, not_found, internal_sever_error, bad_format],
        )
}

#[catch(401)]
pub fn unauthorized() -> Json<ErrorResponse> {
    Json(ErrorResponse {
        cause: "UNAUTHORIZED".to_string(),
        message: "Endpoint call without valid token".to_string(),
        date: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    })
}

#[catch(404)]
pub fn not_found(_req: &Request) -> Json<ErrorResponse> {
    println!("{}", _req);
    let content_type = _req
        .headers()
        .get_one("content-type")
        .unwrap_or("unknown/type");
    let mut message = "Endpoint not found ".to_string();
    message.push_str(content_type);
    Json(ErrorResponse {
        cause: "NOT FOUND".to_string(),
        message,
        date: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    })
}
#[catch(422)]
pub fn bad_format(_req: &Request) -> Json<ErrorResponse> {
    let mut message = "Bad formated body in the request ".to_string();
    message.push_str(_req.uri().path().as_str());
    Json(ErrorResponse {
        cause: "BAD FORMAT".to_string(),
        message,
        date: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    })
}
#[catch(500)]
pub fn internal_sever_error() -> Json<ErrorResponse> {
    Json(ErrorResponse {
        cause: "INTERNAL SERVER ERROR".to_string(),
        message: "Something went wrong :(".to_string(),
        date: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    })
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
