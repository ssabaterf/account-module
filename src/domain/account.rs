
use rand::{distributions::Alphanumeric, Rng};
use revolt_rocket_okapi::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{borrow::Borrow, collections::HashMap};

use crate::domain::{asset::{AssetManager, AssetType, Asset}};

use super::ledger::{Crypto, Fiat};
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Balance {
    pub asset: String,
    pub balance: f64,
    pub hold: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Account {
    pub user_owner_id: String,
    pub account_number: String,
    pub accounts_fiat: HashMap<String, String>,
    pub accounts_crypto: HashMap<String, String>,
    pub constraints: HashMap<String, String>,
    pub active: bool,
}
impl Account {
    pub fn init(
        asset_master: &AssetManager,
        default_assets: &Vec<String>,
        owner_id: String,
    ) -> Result<Account, String> {
        let id = account_number_generator().to_uppercase();
        let mut account = Account {
            user_owner_id: owner_id,
            account_number: id,
            accounts_fiat: HashMap::new(),
            accounts_crypto: HashMap::new(),
            constraints: HashMap::new(),
            active: false,
        };
        for asset in default_assets {
            let default_asset = match asset_master.get_by_symbol(asset.borrow()) {
                Some(asset) => asset,
                None => return Err(format!("{} {}", asset, "Asset not found")),
            };
            match default_asset.asset_type {
                AssetType::Fiat => account
                    .accounts_fiat
                    .insert(default_asset.symbol.clone(), account.account_number.clone()),
                AssetType::Crypto => account
                    .accounts_crypto
                    .insert(default_asset.symbol.clone(), account.account_number.clone()),
            };
        }
        Ok(account)
    }
    pub fn balance(fiats: Vec<Fiat>, crypto: Vec<Crypto>) -> HashMap<String, Balance> {
        let mut balances = HashMap::new();
        for fiat in fiats {
            balances.insert(
                fiat.asset.symbol.clone(),
                Balance {
                    asset: fiat.asset.symbol.clone(),
                    balance: fiat.balance,
                    hold: fiat.hold,
                },
            );
        }
        for crypto in crypto {
            balances.insert(
                crypto.asset.symbol.clone(),
                Balance {
                    asset: crypto.asset.symbol.clone(),
                    balance: crypto.balance,
                    hold: crypto.hold,
                },
            );
        }
        balances
    }
    pub fn add_fiat(&mut self, asset: Asset)->Account{
        match asset.asset_type {
            AssetType::Fiat => self.accounts_fiat.insert(asset.symbol, self.account_number.clone()),
            AssetType::Crypto => return self.clone(),
        };
        self.clone()
    }
    pub fn add_crypto(&mut self, asset: Asset)->Account{
        match asset.asset_type {
            AssetType::Fiat => return self.clone(),
            AssetType::Crypto => self.accounts_crypto.insert(asset.symbol, self.account_number.clone()),
        };
        self.clone()
    }
}

fn account_number_generator() -> String {
    let mut rng = rand::thread_rng();
    (0..12)
        .map(|_| rng.sample(Alphanumeric))
        .map(|x| (x) as char)
        .collect()
}
