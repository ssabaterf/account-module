use std::{collections::HashMap, borrow::Borrow};
use rand::{Rng, distributions::Alphanumeric};
use serde::{Serialize, Deserialize};
use crate::{asset::asset::{AssetManager, AssetType}};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Wallet {
    pub account_number: String,
    pub accounts_fiat: HashMap<String,String>,
    pub accounts_crypto: HashMap<String,String>,
    pub constraints: HashMap<String,String>,
}
impl Wallet {
    pub fn init(asset_master: &AssetManager, default_assets: &Vec<String>) -> Result<Wallet,String> {
        let id = account_number_generator().to_uppercase();
        let mut wallet = Wallet {
            account_number:id ,
            accounts_fiat: HashMap::new(),
            accounts_crypto: HashMap::new(),
            constraints: HashMap::new(),
        };
        for asset in default_assets {
            let default_asset = match asset_master.get_by_symbol(asset.borrow()){
                Some(asset) => asset,
                None => return Err(format!("{} {}",asset,"Asset not found".to_string())),
            };
            match default_asset.asset_type {
                AssetType::Fiat => wallet.accounts_fiat.insert(default_asset.symbol.clone(),wallet.account_number.clone()),
                AssetType::Crypto => wallet.accounts_crypto.insert(default_asset.symbol.clone(),wallet.account_number.clone()),
            };
        };
        // todo!("Add constraints to wallet");
        // todo!("Add wallet to database");
        // todo!("Crate currency account for each asset");
        return Ok(wallet);
    }
}
fn account_number_generator()->String{
    let mut rng = rand::thread_rng();
    (0..12)
        .map(|_| rng.sample(Alphanumeric))
        .map(|x| (x) as char)
        .collect()
}