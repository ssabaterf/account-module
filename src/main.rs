
use bank::{bank::Bank, transaction::Transaction};
use currency::currency::{Fiat, Crypto};
use mongo::Data;
use wallet::wallet::Wallet;

pub mod currency;
pub mod asset;
pub mod wallet;
mod bank;
mod mongo;


#[tokio::main]
async fn main() {
    let client = match Data::new("mongodb://localhost:27017", "My Bank", "bank").await {
        Ok(client) => client,
        Err(e) => {
            println!("Error creating client: {}", e);
            return;
        }
    };
    let fiat_db = client.get_repo::<Fiat>("fiat_vault", "account_number".to_string()).unwrap();
    let crypto_db = client.get_repo::<Crypto>("crypto_vault", "account_number".to_string()).unwrap();
    // let asset_db = client.get_repo::<Asset>("asset_vault").unwrap();
    let wallet_db = client.get_repo::<Wallet>("wallet", "account_number".to_string()).unwrap();
    let transaction_db = client.get_repo::<Transaction>("transaction", "tx_id".to_string()).unwrap();
    let mut my_awesome_bank = Bank::new("MAB".to_string(),"123".to_string(), wallet_db, fiat_db, crypto_db, transaction_db);
    
    for _ in 0..2{
        let wallet = my_awesome_bank.create_wallet().await.unwrap();
        my_awesome_bank.deposit_in(&wallet.account_number, 100.0, "USD".to_owned()).await.unwrap();
        my_awesome_bank.confirm_deposit(&wallet.account_number, 75.0, "USD".to_owned()).await.unwrap();
        my_awesome_bank.withdraw_from(&wallet.account_number, 50.0, "USD".to_owned()).await.unwrap();
        my_awesome_bank.confirm_withdraw(&wallet.account_number, 25.0, "USD".to_owned()).await.unwrap();
    }
}
