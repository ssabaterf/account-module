use std::{borrow::Borrow, collections::HashMap};

use rand::Rng;

use crate::{
    asset::asset::{AssetManager, AssetType},
    currency::currency::{Crypto, Fiat, FungibleTradeable},
    mongo::Crud,
    mongo::Repository,
    wallet::wallet::Wallet,
};

use super::transaction::{Confirmed, Transaction, TransactionStatus, TransactionType};

pub struct Balance {
    pub asset: String,
    pub balance: f64,
    pub hold: f64,
}
pub struct Bank {
    pub name: String,
    pub bank_id: String,
    pub asset: AssetManager,
    pub wallet: Repository<Wallet>,
    pub fiat_vault: Repository<Fiat>,
    pub crypto_vault: Repository<Crypto>,
    pub transaction: Repository<Transaction>,
}
impl Bank {
    pub fn new(
        name: String,
        id: String,
        wallet_repo: Repository<Wallet>,
        fiat_repo: Repository<Fiat>,
        crypto_repo: Repository<Crypto>,
        tx_repo: Repository<Transaction>,
    ) -> Bank {
        let am = AssetManager::new();
        let bank = Bank {
            name,
            bank_id: id,
            asset: am,
            wallet: wallet_repo,
            fiat_vault: fiat_repo,
            crypto_vault: crypto_repo,
            transaction: tx_repo,
        };
        bank
    }
    pub async fn balance(&self, account: String) -> HashMap<String, Balance> {
        let mut balances = HashMap::new();
        let balances_result = match self
            .fiat_vault
            .get_by_fields(vec!["account_number".to_string()], vec![account.clone()])
            .await
        {
            Ok(b) => b,
            Err(_) => return balances,
        };
        for fiat in balances_result {
            if fiat.account_number == account {
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
        }
        let balances_result = match self
            .crypto_vault
            .get_by_fields(vec!["account_number".to_string()], vec![account.clone()])
            .await
        {
            Ok(b) => b,
            Err(_) => return balances,
        };
        for crypto in balances_result {
            if crypto.account_number == account {
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
        }
        balances
    }
    pub async fn create_wallet(&mut self) -> Result<Wallet, String> {
        let default_assets = vec!["USD".to_string(), "EUR".to_string(), "BTC".to_string()];
        let wallet = match Wallet::init(&self.asset, &default_assets) {
            Ok(w) => w,
            Err(e) => return Err(e),
        };
        let mut temp_fiat: Vec<Fiat> = Vec::new();
        let mut temp_crypto: Vec<Crypto> = Vec::new();

        for asset in default_assets {
            let default_asset = match self.asset.get_by_symbol(asset.borrow()) {
                Some(asset) => asset,
                None => return Err(format!("{} {}", asset, "Asset not found".to_string())),
            };
            match default_asset.asset_type {
                AssetType::Fiat => match Fiat::new(wallet.account_number.clone(), default_asset) {
                    Ok(acc) => temp_fiat.push(acc),
                    Err(e) => return Err(e),
                },
                AssetType::Crypto => {
                    let network: String = match self.get_network(&default_asset.symbol) {
                        Ok(n) => n,
                        Err(e) => return Err(e),
                    };
                    let address = match self.get_address_for(&network) {
                        Ok(a) => a,
                        Err(e) => return Err(e),
                    };
                    match Crypto::new(
                        wallet.account_number.clone(),
                        default_asset,
                        network,
                        address,
                    ) {
                        Ok(acc) => temp_crypto.push(acc),
                        Err(e) => return Err(e),
                    }
                }
            };
        }
        self.fiat_vault.create_many(temp_fiat).await?;
        self.crypto_vault.create_many(temp_crypto).await?;
        self.wallet.create(wallet.clone()).await?;
        Ok(wallet)
    }
    pub async fn deposit_in(
        &mut self,
        account_id: &str,
        amount: f64,
        asset: String,
    ) -> Result<(), String> {
        match self
            .wallet
            .get_by_fields(
                vec!["account_number".to_string()],
                vec![account_id.to_string()],
            )
            .await
        {
            Ok(_) => (),
            Err(_) => return Err("Wallet does not exist".to_string()),
        }
        let asset = match self.asset.get_by_symbol(asset.borrow()) {
            Some(asset) => asset,
            None => return Err("Asset not found".to_string()),
        };
        if asset.asset_type == AssetType::Crypto {
            match self
                .crypto_vault
                .get_by_fields(
                    vec!["account_number".to_string(), "asset.symbol".to_string()],
                    vec![account_id.to_string(), asset.symbol.clone()],
                )
                .await
            {
                Ok(mut c) => {
                    if c.len() == 0 {
                        return Err("Account not found".to_string());
                    } else {
                        let acc = c.get_mut(0).unwrap();
                        acc.deposit(amount)?;
                        match self
                            .crypto_vault
                            .update_by_id(&acc.account_number, acc.to_owned())
                            .await
                        {
                            Ok(_) => Ok(()),
                            Err(_) => Err("Error updating account".to_string()),
                        }
                    }
                }
                Err(_) => return Err("Account not found".to_string()),
            }
        } else if asset.asset_type == AssetType::Fiat {
            match self
                .fiat_vault
                .get_by_fields(
                    vec!["account_number".to_string(), "asset.symbol".to_string()],
                    vec![account_id.to_string(), asset.symbol.clone()],
                )
                .await
            {
                Ok(mut c) => {
                    if c.len() == 0 {
                        return Err("Account not found".to_string());
                    } else {
                        let acc = c.get_mut(0).unwrap();
                        acc.deposit(amount)?;
                        match self
                            .fiat_vault
                            .update_by_id(&acc.account_number, acc.to_owned())
                            .await
                        {
                            Ok(_) => Ok(()),
                            Err(e) => Err(format!("Error updating account: {}", e)),
                        }
                    }
                }
                Err(_) => return Err("Account not found".to_string()),
            }
        } else {
            return Err("Asset type not found".to_string());
        }
    }
    pub async fn withdraw_from(
        &mut self,
        account_id: &str,
        amount: f64,
        asset: String,
    ) -> Result<(), String> {
        if !self
            .wallet
            .get_by_fields(
                vec!["account_number".to_string()],
                vec![account_id.to_string()],
            )
            .await?
            .len()
            < 1
        {
            return Err("Wallet does not exist".to_string());
        }
        let asset = match self.asset.get_by_symbol(asset.borrow()) {
            Some(asset) => asset,
            None => return Err("Asset not found".to_string()),
        };
        if asset.asset_type == AssetType::Crypto {
            match self
                .crypto_vault
                .get_by_fields(
                    vec!["account_number".to_string(), "asset.symbol".to_string()],
                    vec![account_id.to_string(), asset.symbol.to_string()],
                )
                .await
            {
                Ok(mut c) => {
                    if c.len() == 1 {
                        let acc = c.get_mut(0).unwrap();
                        acc.withdraw(amount)?;
                        match self
                            .crypto_vault
                            .update_by_id(&acc.account_number, acc.to_owned())
                            .await
                        {
                            Ok(_) => Ok(()),
                            Err(_) => Err("Error updating account".to_string()),
                        }
                    } else {
                        return Err("Account not found".to_string());
                    }
                }
                Err(_) => return Err("Account not found".to_string()),
            }
        } else if asset.asset_type == AssetType::Fiat {
            match self
                .fiat_vault
                .get_by_fields(
                    vec!["account_number".to_string(), "asset.symbol".to_string()],
                    vec![account_id.to_string(), asset.symbol.to_string()],
                )
                .await
            {
                Ok(mut c) => {
                    if c.len() == 1 {
                        let acc = c.get_mut(0).unwrap();
                        acc.withdraw(amount)?;
                        match self
                            .fiat_vault
                            .update_by_id(&acc.account_number, acc.to_owned())
                            .await
                        {
                            Ok(_) => Ok(()),
                            Err(_) => Err("Error updating account".to_string()),
                        }
                    } else {
                        return Err("Account not found".to_string());
                    }
                }
                Err(_) => return Err("Account not found".to_string()),
            }
        } else {
            return Err("Asset type not found".to_string());
        }
    }
    pub async fn confirm_deposit(
        &mut self,
        account_id: &str,
        amount: f64,
        asset: String,
    ) -> Result<(), String> {
        if !self
            .wallet
            .get_by_fields(
                vec!["account_number".to_string()],
                vec![account_id.to_string()],
            )
            .await?
            .len()
            < 1
        {
            return Err("Wallet does not exist".to_string());
        }
        let asset = match self.asset.get_by_symbol(asset.borrow()) {
            Some(asset) => asset,
            None => return Err("Asset not found".to_string()),
        };
        if asset.asset_type == AssetType::Crypto {
            match self
                .crypto_vault
                .get_by_fields(
                    vec!["account_number".to_string(), "asset.symbol".to_string()],
                    vec![account_id.to_string(), asset.symbol.to_string()],
                )
                .await
            {
                Ok(mut c) => {
                    if c.len() == 1 {
                        let acc = c.get_mut(0).unwrap();
                        acc.confirm_deposit(amount)?;
                        match self
                            .crypto_vault
                            .update_by_id(&acc.account_number, acc.to_owned())
                            .await
                        {
                            Ok(_) => Ok(()),
                            Err(_) => Err("Error updating account".to_string()),
                        }
                    } else {
                        return Err("Account not found".to_string());
                    }
                }
                Err(_) => return Err("Account not found".to_string()),
            }
        } else if asset.asset_type == AssetType::Fiat {
            match self
                .fiat_vault
                .get_by_fields(
                    vec!["account_number".to_string(), "asset.symbol".to_string()],
                    vec![account_id.to_string(), asset.symbol.to_string()],
                )
                .await
            {
                Ok(mut c) => {
                    if c.len() == 1 {
                        let acc = c.get_mut(0).unwrap();
                        acc.confirm_deposit(amount)?;
                        match self
                            .fiat_vault
                            .update_by_id(&acc.account_number, acc.to_owned())
                            .await
                        {
                            Ok(_) => Ok(()),
                            Err(_) => Err("Error updating account".to_string()),
                        }
                    } else {
                        return Err("Account not found".to_string());
                    }
                }
                Err(_) => return Err("Account not found".to_string()),
            }
        } else {
            return Err("Asset type not found".to_string());
        }
    }
    pub async fn confirm_withdraw(
        &mut self,
        account_id: &str,
        amount: f64,
        asset: String,
    ) -> Result<(), String> {
        if !self
            .wallet
            .get_by_fields(
                vec!["account_number".to_string()],
                vec![account_id.to_string()],
            )
            .await?
            .len()
            < 1
        {
            return Err("Wallet does not exist".to_string());
        }
        let asset = match self.asset.get_by_symbol(asset.borrow()) {
            Some(asset) => asset,
            None => return Err("Asset not found".to_string()),
        };
        if asset.asset_type == AssetType::Crypto {
            match self
                .crypto_vault
                .get_by_fields(
                    vec!["account_number".to_string(), "asset.symbol".to_string()],
                    vec![account_id.to_string(), asset.symbol.to_string()],
                )
                .await
            {
                Ok(mut c) => {
                    if c.len() == 1 {
                        let acc = c.get_mut(0).unwrap();
                        acc.confirm_withdraw(amount)?;
                        match self
                            .crypto_vault
                            .update_by_id(&acc.account_number, acc.to_owned())
                            .await
                        {
                            Ok(_) => Ok(()),
                            Err(_) => Err("Error updating account".to_string()),
                        }
                    } else {
                        return Err("Account not found".to_string());
                    }
                }
                Err(_) => return Err("Account not found".to_string()),
            }
        } else if asset.asset_type == AssetType::Fiat {
            match self
                .fiat_vault
                .get_by_fields(
                    vec!["account_number".to_string(), "asset.symbol".to_string()],
                    vec![account_id.to_string(), asset.symbol.to_string()],
                )
                .await
            {
                Ok(mut c) => {
                    if c.len() == 1 {
                        let acc = c.get_mut(0).unwrap();
                        acc.confirm_withdraw(amount)?;
                        match self
                            .fiat_vault
                            .update_by_id(&acc.account_number, acc.to_owned())
                            .await
                        {
                            Ok(_) => Ok(()),
                            Err(_) => Err("Error updating account".to_string()),
                        }
                    } else {
                        return Err("Account not found".to_string());
                    }
                }
                Err(_) => return Err("Account not found".to_string()),
            }
        } else {
            return Err("Asset type not found".to_string());
        }
    }
    pub async fn transfer(
        &mut self,
        from: String,
        to: String,
        amount: f64,
        asset: String,
    ) -> Result<String, String> {
        if !self
            .wallet
            .get_by_fields(vec!["account_number".to_string()], vec![from.to_string()])
            .await?
            .len()
            < 1
        {
            return Err("Wallet From does not exist".to_string());
        }
        if !self
            .wallet
            .get_by_fields(vec!["account_number".to_string()], vec![to.to_string()])
            .await?
            .len()
            < 1
        {
            return Err("Wallet To does not exist".to_string());
        }
        let asset = match self.asset.get_by_symbol(asset.borrow()) {
            Some(asset) => asset,
            None => return Err("Asset not found".to_string()),
        };
        if asset.asset_type == AssetType::Crypto {
            let (mut account_from,mut account_to) =
                match self.get_account_transfer_crypto(&from, &to, &asset.symbol).await {
                    Ok((w1, w2)) => (w1, w2),
                    Err(e) => return Err(e),
                };
            account_from.withdraw(amount)?;
            account_to.deposit(amount)?;

            let tx = Transaction::new(
                TransactionType::Transfer,
                asset.symbol.clone(),
                amount,
                from,
                account_from.account_number.clone(),
                to,
                account_to.account_number.clone(),
                "Transfer internal".to_owned(),
                2,
            );
            let tx_id = tx.tx_id.clone();
            self.transaction.create(tx).await?;
            Ok(tx_id)
        } else if asset.asset_type == AssetType::Fiat {
            let (mut account_from, mut account_to) =
                match self.get_account_transfer_fiat(&from, &to, &asset.symbol).await {
                    Ok((w1, w2)) => (w1, w2),
                    Err(e) => return Err(e),
                };
            account_from.withdraw(amount)?;
            account_to.deposit(amount)?;

            let tx = Transaction::new(
                TransactionType::Transfer,
                asset.symbol.clone(),
                amount,
                from,
                account_from.account_number.clone(),
                to,
                account_to.account_number.clone(),
                "Transfer internal".to_owned(),
                2,
            );
            let tx_id = tx.tx_id.clone();
            self.transaction.create(tx).await?;
            Ok(tx_id)
        } else {
            return Err("Asset type not found".to_string());
        }
    }
    pub async fn confirm_transfer(&mut self, tx_id: String, id_confirmer: String) -> Result<(), String> {
        let mut tx = match self
            .transaction
            .get_by_fields(vec!["tx_id".to_string()], vec![tx_id.clone()]).await
        {
            Ok(mut t) => {
                if t.len() == 1 {
                    t.get_mut(0).unwrap().to_owned()
                } else {
                    return Err("Transaction not found".to_string());
                }
            },
            Err(_) => return Err("Transaction not found".to_string()),
        };
        if tx
            .confirmations
            .iter()
            .any(|e| e.id_confirmer.eq(&id_confirmer))
        {
            return Err("Transaction already confirmed".to_string());
        }
        if tx.confirmations.len() as u32 == tx.confirmations_required {
            return Err("Transaction already confirmed".to_string());
        }
        tx.confirmations.push(Confirmed::new(id_confirmer));
        if tx.confirmations.len() as u32 == tx.confirmations_required {
            tx.transaction_status = TransactionStatus::Confirmed;
            let from_wallet = tx.from_wallet.clone();
            let to_wallet = tx.to_wallet.clone();
            let amount = tx.amount;
            let asset = tx.asset.clone();
            match self.release_transfer(from_wallet, to_wallet, amount, asset).await {
                Ok(_) => {
                    println!("Transfer released")
                }
                Err(e) => return Err(e),
            }
            Ok(())
        } else {
            Ok(())
        }
    }
    pub async fn release_transfer(
        &mut self,
        from_wallet: String,
        to_wallet: String,
        amount: f64,
        asset: String,
    ) -> Result<(), String> {
        if !self
            .wallet
            .get_by_fields(
                vec!["account_number".to_string()],
                vec![from_wallet.to_string()],
            )
            .await?
            .len()
            < 1
        {
            return Err("Wallet From does not exist".to_string());
        }
        if !self
            .wallet
            .get_by_fields(
                vec!["account_number".to_string()],
                vec![to_wallet.to_string()],
            )
            .await?
            .len()
            < 1
        {
            return Err("Wallet To does not exist".to_string());
        }
        let asset = match self.asset.get_by_symbol(asset.borrow()) {
            Some(asset) => asset,
            None => return Err("Asset not found".to_string()),
        };
        if asset.asset_type == AssetType::Crypto {
            let (mut account_from, mut account_to) =
                match self.get_account_transfer_crypto(&from_wallet, &to_wallet, &asset.symbol).await {
                    Ok((w1, w2)) => (w1, w2),
                    Err(e) => return Err(e),
                };
            account_from.confirm_withdraw(amount)?;
            account_to.confirm_deposit(amount)?;
            Ok(())
        } else if asset.asset_type == AssetType::Fiat {
            let (mut account_from, mut account_to) =
                match self.get_account_transfer_fiat(&from_wallet, &to_wallet, &asset.symbol).await {
                    Ok((w1, w2)) => (w1, w2),
                    Err(e) => return Err(e),
                };
            account_from.confirm_withdraw(amount)?;
            account_to.confirm_deposit(amount)?;
            Ok(())
        } else {
            return Err("Asset type not found".to_string());
        }
    }
    pub async fn get_account_transfer_fiat(
        &mut self,
        wallet_from: &str,
        wallet_for: &str,
        asset: &str,
    ) -> Result<(Fiat, Fiat), String> {
        let account_f = self.fiat_vault.
        get_by_fields(vec!["asset.symbol".to_string(), "account_number".to_string()], 
        vec![asset.to_string(), wallet_from.to_string()]).await?;
        let account_t = self.fiat_vault.
        get_by_fields(vec!["asset.symbol".to_string(), "account_number".to_string()], 
        vec![asset.to_string(), wallet_for.to_string()]).await?;

       if account_f.len() == 1 && account_t.len() == 1 {
           Ok((account_f.get(0).unwrap().clone(), account_t.get(0).unwrap().clone()))
       } else {
           Err("Wallet not found".to_string())
       }
    }
    pub async fn get_account_transfer_crypto(
        &mut self,
        wallet_from: &str,
        wallet_for: &str,
        asset: &str,
    ) -> Result<(Crypto, Crypto), String> {
        let account_f = self.crypto_vault.
        get_by_fields(vec!["asset.symbol".to_string(), "account_number".to_string()], 
        vec![asset.to_string(), wallet_from.to_string()]).await?;
        let account_t = self.crypto_vault.
        get_by_fields(vec!["asset.symbol".to_string(), "account_number".to_string()], 
        vec![asset.to_string(), wallet_for.to_string()]).await?;

       if account_f.len() == 1 && account_t.len() == 1 {
           Ok((account_f.get(0).unwrap().clone(), account_t.get(0).unwrap().clone()))
       } else {
           Err("Wallet not found".to_string())
       }
    }
    fn get_network(&self, asset: &str) -> Result<String, String> {
        match asset {
            "BTC" => Ok("bitcoin".to_string()),
            "ETH" => Ok("ethereum".to_string()),
            "LTC" => Ok("litecoin".to_string()),
            "BCH" => Ok("bitcoin-cash".to_string()),
            "XRP" => Ok("ripple".to_string()),
            "XLM" => Ok("stellar".to_string()),
            "EOS" => Ok("eos".to_string()),
            "TRX" => Ok("tron".to_string()),
            "DASH" => Ok("dash".to_string()),
            "ZEC" => Ok("zcash".to_string()),
            _ => Err("Invalid Asset".to_string()),
        }
    }
    fn get_address_for(&self, _: &str) -> Result<String, String> {
        let fake_random_address = fake_address_generator();
        Ok(fake_random_address)
    }
}

fn fake_address_generator() -> String {
    let mut rng = rand::thread_rng();
    let mut address = String::new();
    for _ in 0..34 {
        let random_char = rng.gen_range(0..35);
        address.push_str(&random_char.to_string());
    }
    address
}
