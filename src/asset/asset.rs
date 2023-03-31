use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AssetType {
    Fiat,
    Crypto,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Asset {
    pub name: String,
    pub symbol: String,
    pub asset_type: AssetType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssetManager {
    assets: [Asset;20],
}
impl AssetManager {
    pub fn new()-> AssetManager {
    AssetManager{assets: [
            Asset {
                name: "United State Dollar".to_string(),
                symbol: "USD".to_string(),
                asset_type: AssetType::Fiat,
            },
            Asset {
                name: "Euro".to_string(),
                symbol: "EUR".to_string(),
                asset_type: AssetType::Fiat,
            },
            Asset {
                name: "Japanese Yen".to_string(),
                symbol: "JPY".to_string(),
                asset_type: AssetType::Fiat,
            },
            Asset {
                name: "Pound Sterling".to_string(),
                symbol: "GBP".to_string(),
                asset_type: AssetType::Fiat,
            },
            Asset {
                name: "Australian Dollar".to_string(),
                symbol: "AUD".to_string(),
                asset_type: AssetType::Fiat,
            },
            Asset {
                name: "Canadian Dollar".to_string(),
                symbol: "CAD".to_string(),
                asset_type: AssetType::Fiat,
            },
            Asset {
                name: "Swiss Franc".to_string(),
                symbol: "CHF".to_string(),
                asset_type: AssetType::Fiat,
            },
            Asset {
                name: "Chinese Renminbi".to_string(),
                symbol: "CNH".to_string(),
                asset_type: AssetType::Fiat,
            },
            Asset {
                name: "Hong Kong Dollar".to_string(),
                symbol: "GBP".to_string(),
                asset_type: AssetType::Fiat,
            },
            Asset {
                name: "New Zealand Dollar".to_string(),
                symbol: "NZD".to_string(),
                asset_type: AssetType::Fiat,
            },
            Asset {
                name: "Bitcoin".to_string(),
                symbol: "BTC".to_string(),
                asset_type: AssetType::Crypto,
            },
            Asset {
                name: "Ethereum".to_string(),
                symbol: "ETH".to_string(),
                asset_type: AssetType::Crypto,
            },
            Asset {
                name: "Tether".to_string(),
                symbol: "USDT".to_string(),
                asset_type: AssetType::Crypto,
            },
            Asset {
                name: "Binance Coin".to_string(),
                symbol: "BNB".to_string(),
                asset_type: AssetType::Crypto,
            },
            Asset {
                name: "US Dollar Coin".to_string(),
                symbol: "USDC".to_string(),
                asset_type: AssetType::Crypto,
            },
            Asset {
                name: "XRP".to_string(),
                symbol: "XRP".to_string(),
                asset_type: AssetType::Crypto,
            },
            Asset {
                name: "Cardano".to_string(),
                symbol: "ADA".to_string(),
                asset_type: AssetType::Crypto,
            },
            Asset {
                name: "Polygon".to_string(),
                symbol: "MATIC".to_string(),
                asset_type: AssetType::Crypto,
            },
            Asset {
                name: "Dogecoin".to_string(),
                symbol: "DOGE".to_string(),
                asset_type: AssetType::Crypto,
            },
            Asset {
                name: "Solana".to_string(),
                symbol: "SOL".to_string(),
                asset_type: AssetType::Crypto,
            },
        ]}
    }
    pub fn get_by_symbol(&self, symbol:&str)->Option<Asset> {
        for asset in &self.assets {
            if asset.symbol == symbol {
                return Some(asset.clone());
            }
        }
        return None;
    }
}
