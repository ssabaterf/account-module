
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use std::{borrow::Borrow, collections::HashMap};

use crate::domain::{asset::asset::{AssetManager, AssetType, Asset}, ledger::ledger::{Crypto, Fiat}};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Balance {
    pub asset: String,
    pub balance: f64,
    pub hold: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Account {
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
    ) -> Result<Account, String> {
        let id = account_number_generator().to_uppercase();
        let mut account = Account {
            account_number: id,
            accounts_fiat: HashMap::new(),
            accounts_crypto: HashMap::new(),
            constraints: HashMap::new(),
            active: false,
        };
        for asset in default_assets {
            let default_asset = match asset_master.get_by_symbol(asset.borrow()) {
                Some(asset) => asset,
                None => return Err(format!("{} {}", asset, "Asset not found".to_string())),
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
        // todo!("Add constraints to account");
        // todo!("Add account to database");
        // todo!("Crate currency account for each asset");
        return Ok(account);
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
            break;
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
            break;
        }
        balances
    }
    pub fn add_fiat(&mut self, asset: Asset)->Account{
        match asset.asset_type {
            AssetType::Fiat => self.accounts_fiat.insert(asset.symbol.clone(), self.account_number.clone()),
            AssetType::Crypto => return self.clone(),
        };
        self.clone()
    }
    pub fn add_crypto(&mut self, asset: Asset)->Account{
        match asset.asset_type {
            AssetType::Fiat => return self.clone(),
            AssetType::Crypto => self.accounts_crypto.insert(asset.symbol.clone(), self.account_number.clone()),
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
