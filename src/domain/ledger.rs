use serde::{Deserialize, Serialize};

use super::asset::{Asset, AssetType};

pub trait Accounting {
    fn get_account_number(&self)->String;
}
pub trait FungibleTradeable {
    fn deposit(&mut self, amount: f64)->Result<(),String>;
    fn withdraw(&mut self, amount: f64)->Result<(),String>;
    fn confirm_deposit(&mut self, amount: f64)->Result<(),String>;
    fn confirm_withdraw(&mut self, amount: f64)->Result<(),String>;
    fn cancel_deposit(&mut self, amount: f64)->Result<(),String>;
    fn cancel_withdraw(&mut self, amount: f64)->Result<(),String>;
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Fiat {
    pub id: String,
    pub account_number: String,
    pub asset_type: AssetType,
    pub asset: Asset,
    pub balance: f64,
    pub hold: f64,
}
impl Fiat {
    pub fn new(account_id:String, asset: Asset)->Result<Fiat,String> {
        if account_id.is_empty() {
            return Err("Account ID cannot be empty".to_string());
        }
        let mut id = "".to_string();
        id.push_str(&account_id);
        id.push('_');
        id.push_str(&asset.symbol);
        match asset.asset_type {
            AssetType::Fiat => Ok(Fiat {
                id,
                account_number: account_id,
                asset,
                asset_type: AssetType::Fiat,
                balance: 0.0,
                hold: 0.0,
            }),
            _ => Err("Asset type must be Fiat".to_string()),
        }
    }
}
impl Accounting for Fiat {
    fn get_account_number(&self)->String {
        self.account_number.clone()
    }
}
impl FungibleTradeable for Fiat {
    fn deposit(&mut self, amount: f64)->Result<(),String> {
        if amount < 0.0 {
            return Err("Amount must be positive".to_string());
        }
        self.hold += amount;
        Ok(())
    }
    fn withdraw(&mut self, amount: f64)->Result<(),String> {
        if amount < 0.0 {
            return Err("Amount must be positive".to_string());
        }
        self.balance -= amount;
        self.hold += amount;
        Ok(())
    }
    fn confirm_deposit(&mut self, amount: f64)->Result<(),String> {
        if amount < 0.0 {
            return Err("Amount must be positive".to_string());
        }
        self.hold -= amount;
        self.balance += amount;
        Ok(())
    }
    fn confirm_withdraw(&mut self, amount: f64)->Result<(),String> {
        if amount < 0.0 {
            return Err("Amount must be positive".to_string());
        }
        self.hold -= amount;
        Ok(())
    }
    fn cancel_deposit(&mut self, amount: f64)->Result<(),String> {
        if amount < 0.0 {
            return Err("Amount must be positive".to_string());
        }
        self.hold -= amount;
        Ok(())
    }
    fn cancel_withdraw(&mut self, amount: f64)->Result<(),String> {
        if amount < 0.0 {
            return Err("Amount must be positive".to_string());
        }
        self.hold -= amount;
        self.balance += amount;
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Crypto {
    pub id: String,
    pub account_number: String,
    pub asset_type: AssetType,
    pub network: String,
    pub address_in_chain: String,
    pub asset: Asset,
    pub balance: f64,
    pub hold: f64,
}
impl Crypto {
    pub fn new(account_id:String, asset: Asset, network: String, address_in_chain: String)->Result<Crypto,String> {
        if account_id.is_empty() {
            panic!("Account ID cannot be empty");
        }
        let mut id = "".to_string();
        id.push_str(&account_id);
        id.push_str(&asset.symbol);
        match asset.asset_type {
            AssetType::Crypto => Ok(Crypto {
                id,
                account_number: account_id,
                address_in_chain,
                network,
                asset,
                asset_type: AssetType::Crypto,
                balance: 0.0,
                hold: 0.0,
            }),
            _ => Err("Asset type must be Crypto".to_string()),
        }
    }
}
impl Accounting for Crypto {
    fn get_account_number(&self)->String {
        self.account_number.clone()
    }
}
impl FungibleTradeable for Crypto {
    fn deposit(&mut self, amount: f64)->Result<(),String> {
        if amount < 0.0 {
            return Err("Amount must be positive".to_string());
        }
        self.hold += amount;
        Ok(())
    }
    fn withdraw(&mut self, amount: f64)->Result<(),String> {
        if amount < 0.0 {
            return Err("Amount must be positive".to_string());
        }
        self.balance -= amount;
        self.hold += amount;
        Ok(())
    }
    fn confirm_deposit(&mut self, amount: f64)->Result<(),String> {
        if amount < 0.0 {
            return Err("Amount must be positive".to_string());
        }
        self.hold -= amount;
        self.balance += amount;
        Ok(())
    }
    fn confirm_withdraw(&mut self, amount: f64)->Result<(),String> {
        if amount < 0.0 {
            return Err("Amount must be positive".to_string());
        }
        self.hold -= amount;
        Ok(())
    }
    fn cancel_deposit(&mut self, amount: f64)->Result<(),String> {
        if amount < 0.0 {
            return Err("Amount must be positive".to_string());
        }
        self.hold -= amount;
        Ok(())
    }
    fn cancel_withdraw(&mut self, amount: f64)->Result<(),String> {
        if amount < 0.0 {
            return Err("Amount must be positive".to_string());
        }
        self.hold -= amount;
        self.balance += amount;
        Ok(())
    }
}