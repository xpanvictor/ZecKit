use crate::error::FaucetError;
use crate::wallet::history::{TransactionHistory, TransactionRecord};
use std::path::PathBuf;
use tracing::{info, debug, error};
use zingolib::{
    lightclient::LightClient,
    config::{ZingoConfig, ChainType},
};
use zcash_primitives::transaction::components::Amount;

#[derive(Debug, Clone)]
pub struct Balance {
    pub transparent: u64,
    pub sapling: u64,
    pub orchard: u64,
}

impl Balance {
    pub fn total_zatoshis(&self) -> u64 {
        self.transparent + self.sapling + self.orchard
    }

    pub fn total_zec(&self) -> f64 {
        self.total_zatoshis() as f64 / 100_000_000.0
    }

    pub fn orchard_zec(&self) -> f64 {
        self.orchard as f64 / 100_000_000.0
    }

    pub fn transparent_zec(&self) -> f64 {
        self.transparent as f64 / 100_000_000.0
    }
}

pub struct WalletManager {
    client: LightClient,
    history: TransactionHistory,
}

impl WalletManager {
    pub async fn new(
        data_dir: PathBuf,
        server_uri: String,
    ) -> Result<Self, FaucetError> {
        info!("Initializing ZingoLib LightClient");
        
        // Create ZingoConfig for regtest
        let config = ZingoConfig {
            chain: ChainType::Regtest,
            lightwalletd_uri: Some(server_uri.parse().map_err(|e| {
                FaucetError::Wallet(format!("Invalid server URI: {}", e))
            })?),
            data_dir: Some(data_dir.clone()),
            ..Default::default()
        };

        // Create or load wallet
        let client = if data_dir.join("zingo-wallet.dat").exists() {
            info!("Loading existing wallet");
            LightClient::read_wallet_from_disk(&config).await.map_err(|e| {
                FaucetError::Wallet(format!("Failed to load wallet: {}", e))
            })?
        } else {
            info!("Creating new wallet");
            LightClient::create_new_wallet(config).await.map_err(|e| {
                FaucetError::Wallet(format!("Failed to create wallet: {}", e))
            })?
        };

        // Initialize transaction history
        let history = TransactionHistory::load(&data_dir)?;

        // Sync wallet
        info!("Syncing wallet with chain...");
        client.do_sync(true).await.map_err(|e| {
            FaucetError::Wallet(format!("Sync failed: {}", e))
        })?;

        info!("Wallet initialized successfully");

        Ok(Self { client, history })
    }

    pub async fn get_unified_address(&self) -> Result<String, FaucetError> {
        let addresses = self.client.do_addresses().await;
        
        // Parse the JSON response to get the first unified address
        let addr_json: serde_json::Value = serde_json::from_str(&addresses)
            .map_err(|e| FaucetError::Wallet(format!("Failed to parse addresses: {}", e)))?;

        addr_json
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|obj| obj.get("address"))
            .and_then(|a| a.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| FaucetError::Wallet("No unified address found".to_string()))
    }

    pub async fn get_balance(&self) -> Result<Balance, FaucetError> {
        let balance_str = self.client.do_balance().await;
        
        let balance_json: serde_json::Value = serde_json::from_str(&balance_str)
            .map_err(|e| FaucetError::Wallet(format!("Failed to parse balance: {}", e)))?;

        Ok(Balance {
            transparent: balance_json["transparent_balance"]
                .as_u64()
                .unwrap_or(0),
            sapling: balance_json["sapling_balance"]
                .as_u64()
                .unwrap_or(0),
            orchard: balance_json["orchard_balance"]
                .as_u64()
                .unwrap_or(0),
        })
    }

    pub async fn send_transaction(
        &mut self,
        to_address: &str,
        amount_zec: f64,
        memo: Option<String>,
    ) -> Result<String, FaucetError> {
        info!("Sending {} ZEC to {}", amount_zec, &to_address[..16]);

        // Convert ZEC to zatoshis
        let amount_zatoshis = (amount_zec * 100_000_000.0) as u64;

        // Check balance
        let balance = self.get_balance().await?;
        if balance.orchard < amount_zatoshis {
            return Err(FaucetError::InsufficientBalance(format!(
                "Need {} ZEC, have {} ZEC in Orchard pool",
                amount_zec,
                balance.orchard_zec()
            )));
        }

        // Build transaction
        let send_result = if let Some(memo_text) = memo {
            self.client
                .do_send(vec![(to_address, amount_zatoshis, Some(memo_text))])
                .await
        } else {
            self.client
                .do_send(vec![(to_address, amount_zatoshis, None)])
                .await
        };

        // Parse result
        let result_json: serde_json::Value = serde_json::from_str(&send_result)
            .map_err(|e| FaucetError::TransactionFailed(format!("Failed to parse result: {}", e)))?;

        // Check for errors
        if let Some(error) = result_json.get("error") {
            return Err(FaucetError::TransactionFailed(
                error.as_str().unwrap_or("Unknown error").to_string()
            ));
        }

        // Extract TXID
        let txid = result_json["txid"]
            .as_str()
            .ok_or_else(|| FaucetError::TransactionFailed("No TXID in response".to_string()))?
            .to_string();

        info!("Transaction successful: {}", txid);

        // Record transaction
        let record = TransactionRecord {
            timestamp: chrono::Utc::now(),
            to_address: to_address.to_string(),
            amount: amount_zec,
            txid: txid.clone(),
            memo: memo.unwrap_or_default(),
        };

        self.history.add_transaction(record)?;

        Ok(txid)
    }

    pub async fn sync(&self) -> Result<(), FaucetError> {
        self.client.do_sync(true).await.map_err(|e| {
            FaucetError::Wallet(format!("Sync failed: {}", e))
        })?;
        Ok(())
    }

    pub fn get_transaction_history(&self, limit: usize) -> Vec<TransactionRecord> {
        self.history.get_recent(limit)
    }

    pub fn get_stats(&self) -> (usize, f64) {
        let txs = self.history.get_all();
        let count = txs.len();
        let total_sent: f64 = txs.iter().map(|tx| tx.amount).sum();
        (count, total_sent)
    }
}

