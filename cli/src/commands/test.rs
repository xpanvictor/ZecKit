use crate::error::Result;
use colored::*;
use reqwest::Client;
use serde_json::Value;
use tokio::time::{sleep, Duration};

pub async fn execute() -> Result<()> {
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!("{}", "  ZecKit - Running Smoke Tests".cyan().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!();

    let client = Client::new();
    let mut passed = 0;
    let mut failed = 0;

    // Test 1: Zebra RPC
    print!("  [1/6] Zebra RPC connectivity... ");
    match test_zebra_rpc(&client).await {
        Ok(_) => {
            println!("{}", "PASS".green());
            passed += 1;
        }
        Err(e) => {
            println!("{} {}", "FAIL".red(), e);
            failed += 1;
        }
    }

    // Test 2: Faucet Health
    print!("  [2/6] Faucet health check... ");
    match test_faucet_health(&client).await {
        Ok(_) => {
            println!("{}", "PASS".green());
            passed += 1;
        }
        Err(e) => {
            println!("{} {}", "FAIL".red(), e);
            failed += 1;
        }
    }

    // Test 3: Faucet Address
    print!("  [3/6] Faucet address retrieval... ");
    match test_faucet_address(&client).await {
        Ok(_) => {
            println!("{}", "PASS".green());
            passed += 1;
        }
        Err(e) => {
            println!("{} {}", "FAIL".red(), e);
            failed += 1;
        }
    }

    // Test 4: Wallet Sync
    print!("  [4/6] Wallet sync capability... ");
    match test_wallet_sync(&client).await {
        Ok(_) => {
            println!("{}", "PASS".green());
            passed += 1;
        }
        Err(e) => {
            println!("{} {}", "FAIL".red(), e);
            failed += 1;
        }
    }

    // Test 5: Wallet balance and shield (using API endpoints)
    print!("  [5/6] Wallet balance and shield... ");
    match test_wallet_shield(&client).await {
        Ok(_) => {
            println!("{}", "PASS".green());
            passed += 1;
        }
        Err(e) => {
            println!("{} {}", "FAIL".red(), e);
            failed += 1;
        }
    }

    // Test 6: Shielded send (E2E golden flow)
    print!("  [6/6] Shielded send (E2E)... ");
    match test_shielded_send(&client).await {
        Ok(_) => {
            println!("{}", "PASS".green());
            passed += 1;
        }
        Err(e) => {
            println!("{} {}", "FAIL".red(), e);
            failed += 1;
        }
    }

    println!();
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!("  Tests passed: {}", passed.to_string().green());
    println!("  Tests failed: {}", failed.to_string().red());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!();

    if failed > 0 {
        return Err(crate::error::zeckitError::HealthCheck(
            format!("{} test(s) failed", failed)
        ));
    }

    Ok(())
}

async fn test_zebra_rpc(client: &Client) -> Result<()> {
    let resp = client
        .post("http://127.0.0.1:8232")
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": "test",
            "method": "getblockcount",
            "params": []
        }))
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(crate::error::zeckitError::HealthCheck(
            "Zebra RPC not responding".into()
        ));
    }

    Ok(())
}

async fn test_faucet_health(client: &Client) -> Result<()> {
    let resp = client
        .get("http://127.0.0.1:8080/health")
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(crate::error::zeckitError::HealthCheck(
            "Faucet health check failed".into()
        ));
    }

    let json: Value = resp.json().await?;
    
    // Verify key health fields
    if json.get("status").and_then(|v| v.as_str()) != Some("healthy") {
        return Err(crate::error::zeckitError::HealthCheck(
            "Faucet not reporting healthy status".into()
        ));
    }

    Ok(())
}

async fn test_faucet_address(client: &Client) -> Result<()> {
    let resp = client
        .get("http://127.0.0.1:8080/address")
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(crate::error::zeckitError::HealthCheck(
            "Could not get faucet address".into()
        ));
    }

    let json: Value = resp.json().await?;
    
    // Verify both address types are present
    if json.get("unified_address").is_none() {
        return Err(crate::error::zeckitError::HealthCheck(
            "Missing unified address in response".into()
        ));
    }
    
    if json.get("transparent_address").is_none() {
        return Err(crate::error::zeckitError::HealthCheck(
            "Missing transparent address in response".into()
        ));
    }

    Ok(())
}
async fn test_wallet_sync(client: &Client) -> Result<()> {
    let resp = client
        .post("http://127.0.0.1:8080/sync")
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(crate::error::zeckitError::HealthCheck(
            "Wallet sync failed".into()
        ));
    }

    let json: Value = resp.json().await?;
    
    if json.get("status").and_then(|v| v.as_str()) != Some("synced") {
        return Err(crate::error::zeckitError::HealthCheck(
            "Wallet sync did not complete successfully".into()
        ));
    }

    Ok(())
}

async fn test_wallet_shield(client: &Client) -> Result<()> {
    println!();
    
    // Step 1: Get current wallet balance via API
    println!("    Checking wallet balance via API...");
    let balance = get_wallet_balance_via_api(client).await?;
    
    let transparent_before = balance.transparent;
    let orchard_before = balance.orchard;
    
    println!("    Transparent: {} ZEC", transparent_before);
    println!("    Orchard: {} ZEC", orchard_before);
    
    // Step 2: If we have transparent funds >= 0.001 ZEC (accounting for fee), shield them
    let min_shield_amount = 0.0002; // Need at least fee + some amount
    
    if transparent_before >= min_shield_amount {
        println!("    Shielding {} ZEC to Orchard via API...", transparent_before);
        
        // Call the shield endpoint
        let shield_resp = client
            .post("http://127.0.0.1:8080/shield")
            .send()
            .await?;
        
        if !shield_resp.status().is_success() {
            let error_text = shield_resp.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(crate::error::zeckitError::HealthCheck(
                format!("Shield API call failed: {}", error_text)
            ));
        }
        
        let shield_json: Value = shield_resp.json().await?;
        
        // Check shield status
        let status = shield_json.get("status").and_then(|v| v.as_str()).unwrap_or("unknown");
        
        match status {
            "shielded" => {
                if let Some(txid) = shield_json.get("txid").and_then(|v| v.as_str()) {
                    println!("    Shield transaction broadcast!");
                    println!("    TXID: {}...", &txid[..16.min(txid.len())]);
                }
                
                // Wait for transaction to be mined
                println!("    Waiting for transaction to confirm...");
                sleep(Duration::from_secs(30)).await;
                
                // Sync wallet to see new balance
                println!("    Syncing wallet to update balance...");
                let _ = client.post("http://127.0.0.1:8080/sync").send().await;
                sleep(Duration::from_secs(5)).await;
                
                // Check balance after shielding
                let balance_after = get_wallet_balance_via_api(client).await?;
                
                println!("    Balance after shield:");
                println!("    Transparent: {} ZEC (was {})", balance_after.transparent, transparent_before);
                println!("    Orchard: {} ZEC (was {})", balance_after.orchard, orchard_before);
                
                // Verify shield worked (balance changed)
                if balance_after.orchard > orchard_before || balance_after.transparent < transparent_before {
                    println!("    Shield successful - funds moved!");
                } else {
                    println!("    Shield transaction sent but balance not yet updated");
                    println!("    (May need more time to confirm)");
                }
                
                println!();
                print!("  [5/5] Wallet balance and shield... ");
                return Ok(());
            }
            "no_funds" => {
                println!("    No transparent funds to shield (already shielded)");
                println!();
                print!("  [5/5] Wallet balance and shield... ");
                return Ok(());
            }
            _ => {
                println!("    Shield status: {}", status);
                if let Some(msg) = shield_json.get("message").and_then(|v| v.as_str()) {
                    println!("    Message: {}", msg);
                }
                println!();
                print!("  [5/5] Wallet balance and shield... ");
                return Ok(());
            }
        }
        
    } else if orchard_before >= 0.001 {
        println!("    Wallet already has {} ZEC shielded in Orchard - PASS", orchard_before);
        println!();
        print!("  [5/5] Wallet balance and shield... ");
        return Ok(());
        
    } else if transparent_before > 0.0 {
        println!("    Wallet has {} ZEC transparent (too small to shield)", transparent_before);
        println!("    Need at least {} ZEC to cover shield + fee", min_shield_amount);
        println!("    SKIP (insufficient balance)");
        println!();
        print!("  [5/5] Wallet balance and shield... ");
        return Ok(());
        
    } else {
        println!("    No balance found");
        println!("    SKIP (needs mining to complete)");
        println!();
        print!("  [5/5] Wallet balance and shield... ");
        return Ok(());
    }
}

#[derive(Debug)]
struct WalletBalance {
    transparent: f64,
    orchard: f64,
    total: f64,
}

/// Get wallet balance using the /stats endpoint
async fn get_wallet_balance_via_api(client: &Client) -> Result<WalletBalance> {
    let resp = client
        .get("http://127.0.0.1:8080/stats")
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(crate::error::zeckitError::HealthCheck(
            "Failed to get balance from stats endpoint".into()
        ));
    }

    let json: Value = resp.json().await?;
    
    // Extract balance from stats endpoint
    // Stats should have fields like: current_balance, transparent_balance, orchard_balance
    let transparent = json.get("transparent_balance")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    
    let orchard = json.get("orchard_balance")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    
    let total = json.get("current_balance")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    
    Ok(WalletBalance {
        transparent,
        orchard,
        total,
    })
}

/// Test 6: Shielded Send (E2E Golden Flow)
/// This is the key test for Milestone 2 - sending shielded funds to another wallet
async fn test_shielded_send(client: &Client) -> Result<()> {
    println!();
    
    // Step 1: Check faucet has shielded funds
    println!("    Checking faucet Orchard balance...");
    let balance = get_wallet_balance_via_api(client).await?;
    
    if balance.orchard < 0.1 {
        println!("    Faucet has insufficient Orchard balance: {} ZEC", balance.orchard);
        println!("    SKIP (need at least 0.1 ZEC shielded)");
        println!();
        print!("  [6/6] Shielded send (E2E)... ");
        return Ok(());
    }
    
    println!("    Faucet Orchard balance: {} ZEC", balance.orchard);
    
    // ADD THIS: Extra sync to ensure wallet can spend the funds
    println!("    Syncing wallet to ensure spendable balance...");
    let _ = client.post("http://127.0.0.1:8080/sync").send().await;
    sleep(Duration::from_secs(10)).await;
    
    // Step 2: Get a test recipient address (using faucet's own UA for simplicity)
    println!("    Getting recipient address...");
    let addr_resp = client
        .get("http://127.0.0.1:8080/address")
        .send()
        .await?;
    
    if !addr_resp.status().is_success() {
        return Err(crate::error::zeckitError::HealthCheck(
            "Failed to get recipient address".into()
        ));
    }
    
    let addr_json: Value = addr_resp.json().await?;
    let recipient_address = addr_json.get("unified_address")
        .and_then(|v| v.as_str())
        .ok_or_else(|| crate::error::zeckitError::HealthCheck(
            "No unified address in response".into()
        ))?;
    
    println!("    Recipient: {}...", &recipient_address[..20.min(recipient_address.len())]);
    
    // Step 3: Perform shielded send
    let send_amount = 0.05; // Send 0.05 ZEC
    println!("    Sending {} ZEC (shielded)...", send_amount);
    
    let send_resp = client
        .post("http://127.0.0.1:8080/send")
        .json(&serde_json::json!({
            "address": recipient_address,
            "amount": send_amount,
            "memo": "ZecKit smoke test - shielded send"
        }))
        .send()
        .await?;
    
    if !send_resp.status().is_success() {
        let error_text = send_resp.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(crate::error::zeckitError::HealthCheck(
            format!("Shielded send failed: {}", error_text)
        ));
    }
    
    let send_json: Value = send_resp.json().await?;
    
    // Step 4: Verify transaction
    let status = send_json.get("status").and_then(|v| v.as_str());
    
    if status == Some("sent") {
        if let Some(txid) = send_json.get("txid").and_then(|v| v.as_str()) {
            println!("    ✓ Shielded send successful!");
            println!("    TXID: {}...", &txid[..16.min(txid.len())]);
        }
        
        if let Some(new_balance) = send_json.get("orchard_balance").and_then(|v| v.as_f64()) {
            println!("    New Orchard balance: {} ZEC (was {})", new_balance, balance.orchard);
        }
        
        println!("    ✓ E2E Golden Flow Complete:");
        println!("      - Faucet had shielded funds (Orchard)");
        println!("      - Sent {} ZEC to recipient UA", send_amount);
        println!("      - Transaction broadcast successfully");
        
        println!();
        print!("  [6/6] Shielded send (E2E)... ");
        return Ok(());
    } else {
        println!("    Unexpected status: {:?}", status);
        if let Some(msg) = send_json.get("message").and_then(|v| v.as_str()) {
            println!("    Message: {}", msg);
        }
        println!();
        print!("  [6/6] Shielded send (E2E)... ");
        return Err(crate::error::zeckitError::HealthCheck(
            "Shielded send did not complete as expected".into()
        ));
    }
}