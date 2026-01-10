use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::AppState;
use crate::error::FaucetError;
use crate::validation::validate_address_via_zebra;

#[derive(Debug, Deserialize)]
pub struct FaucetRequest {
    address: String,
    amount: Option<f64>,
    memo: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FaucetResponse {
    success: bool,
    txid: String,
    address: String,
    amount: f64,
    new_balance: f64,
    timestamp: String,
    network: String,
    message: String,
}

pub async fn request_funds(
    State(state): State<AppState>,
    Json(payload): Json<FaucetRequest>,
) -> Result<Json<FaucetResponse>, FaucetError> {
    // Validate address via Zebra RPC
    let validated_address = validate_address_via_zebra(
        &payload.address,
        &state.config.zebra_rpc_url,
    ).await?;

    // Get and validate amount
    let amount = payload.amount.unwrap_or(state.config.faucet_amount_default);
    
    if amount < state.config.faucet_amount_min || amount > state.config.faucet_amount_max {
        return Err(FaucetError::InvalidAmount(format!(
            "Amount must be between {} and {} ZEC",
            state.config.faucet_amount_min,
            state.config.faucet_amount_max
        )));
    }

    // Send transaction
    let mut wallet = state.wallet.write().await;
    let txid = wallet.send_transaction(&validated_address, amount, payload.memo).await?;

    // Get new balance
    let new_balance = wallet.get_balance().await?;

    Ok(Json(FaucetResponse {
        success: true,
        txid: txid.clone(),
        address: validated_address,
        amount,
        new_balance: new_balance.total_zec(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        network: "regtest".to_string(),
        message: format!("Sent {} ZEC on regtest. TXID: {}", amount, txid),
    }))
}

pub async fn get_faucet_address(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, FaucetError> {
    let wallet = state.wallet.read().await;
    let address = wallet.get_unified_address().await?;
    let balance = wallet.get_balance().await?;

    Ok(Json(json!({
        "address": address,
        "balance": balance.total_zec(),
        "network": "regtest"
    })))
}

