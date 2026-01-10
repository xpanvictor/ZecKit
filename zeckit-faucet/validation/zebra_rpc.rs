use crate::error::FaucetError;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Debug, Serialize)]
struct ZebraRpcRequest {
    jsonrpc: String,
    id: String,
    method: String,
    params: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ZebraRpcResponse {
    result: Option<ValidateAddressResult>,
    error: Option<RpcError>,
}

#[derive(Debug, Deserialize)]
struct RpcError {
    message: String,
}

#[derive(Debug, Deserialize)]
struct ValidateAddressResult {
    isvalid: bool,
    address: Option<String>,
}

pub async fn validate_address_via_zebra(
    address: &str,
    zebra_rpc_url: &str,
) -> Result<String, FaucetError> {
    debug!("Validating address via Zebra RPC: {}", &address[..12]);

    let client = Client::new();

    let request = ZebraRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: "validate_addr".to_string(),
        method: "validateaddress".to_string(),
        params: vec![address.to_string()],
    };

    let response = client
        .post(zebra_rpc_url)
        .json(&request)
        .send()
        .await
        .map_err(|e| FaucetError::Validation(format!("RPC request failed: {}", e)))?;

    let rpc_result: ZebraRpcResponse = response
        .json()
        .await
        .map_err(|e| FaucetError::Validation(format!("Failed to parse RPC response: {}", e)))?;

    // Check for RPC errors
    if let Some(error) = rpc_result.error {
        return Err(FaucetError::InvalidAddress(format!(
            "RPC validation error: {}",
            error.message
        )));
    }

    // Check validation result
    let result = rpc_result
        .result
        .ok_or_else(|| FaucetError::Validation("No result in RPC response".to_string()))?;

    if !result.isvalid {
        return Err(FaucetError::InvalidAddress(
            "Address is not valid".to_string()
        ));
    }

    // Check for regtest prefix
    let valid_prefixes = ["tm", "uregtest", "zregtestsapling"];
    if !valid_prefixes.iter().any(|p| address.starts_with(p)) {
        return Err(FaucetError::InvalidAddress(
            "Address is not a regtest address".to_string()
        ));
    }

    let validated_address = result.address.unwrap_or_else(|| address.to_string());

    debug!("Address validated: {}", &validated_address[..12]);

    Ok(validated_address)
}
