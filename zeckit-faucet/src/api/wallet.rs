use axum::{extract::State, Json};
use serde_json::json;
use crate::{AppState, error::FaucetError};

/// GET /address - Returns wallet addresses
pub async fn get_addresses(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, FaucetError> {
    let wallet = state.wallet.read().await;  // Change from lock() to read()
    
    let unified_address = wallet.get_unified_address().await?;
    let transparent_address = wallet.get_transparent_address().await?;
    
    Ok(Json(json!({
        "unified_address": unified_address,
        "transparent_address": transparent_address
    })))
}

/// POST /sync - Syncs wallet with blockchain
pub async fn sync_wallet(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, FaucetError> {
    let mut wallet = state.wallet.write().await;  // Change from lock() to write()
    wallet.sync().await?;
    
    Ok(Json(json!({
        "status": "synced",
        "message": "Wallet synced with blockchain"
    })))
}