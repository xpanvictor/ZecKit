use crate::docker::compose::DockerCompose;
use crate::docker::health::HealthChecker;
use crate::error::{Result, zeckitError};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use serde_json::json;
use std::process::Command;
use std::fs;
use std::io::{self, Write};
use tokio::time::{sleep, Duration};

const MAX_WAIT_SECONDS: u64 = 60000;

pub async fn execute(backend: String, fresh: bool) -> Result<()> {
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!("{}", "  ZecKit - Starting Devnet".cyan().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!();
    
    let compose = DockerCompose::new()?;
    
    if fresh {
        println!("{}", "Cleaning up old data...".yellow());
        compose.down(true)?;
    }
    
    let services = match backend.as_str() {
        "lwd" => vec!["zebra", "faucet"],
        "zaino" => vec!["zebra", "faucet"],
        "none" => vec!["zebra", "faucet"],
        _ => {
            return Err(zeckitError::Config(format!(
                "Invalid backend: {}. Use 'lwd', 'zaino', or 'none'", 
                backend
            )));
        }
    };
    
    println!("Starting services: {}", services.join(", "));
    println!();
    
    // Build and start services with progress
    if backend == "lwd" {
        println!("Building Docker images...");
        println!();
        
        println!("[1/3] Building Zebra...");
        println!("[2/3] Building Lightwalletd...");
        println!("[3/3] Building Faucet...");
        
        compose.up_with_profile("lwd")?;
        println!();
    } else if backend == "zaino" {
        println!("Building Docker images...");
        println!();
        
        println!("[1/3] Building Zebra...");
        println!("[2/3] Building Zaino...");
        println!("[3/3] Building Faucet...");
        
        compose.up_with_profile("zaino")?;
        println!();
    } else {
        compose.up(&services)?;
    }
    
    println!("Starting services...");
    println!();
    
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
    );
    
    // [1/3] Zebra with percentage
    let checker = HealthChecker::new();
    let start = std::time::Instant::now();
    
    loop {
        pb.tick();
        
        if checker.wait_for_zebra(&pb).await.is_ok() {
            println!("[1/3] Zebra ready (100%)");
            break;
        }
        
        let elapsed = start.elapsed().as_secs();
        if elapsed < 120 {
            let progress = (elapsed as f64 / 120.0 * 100.0).min(99.0) as u32;
            print!("\r[1/3] Starting Zebra... {}%", progress);
            io::stdout().flush().ok();
            sleep(Duration::from_secs(1)).await;
        } else {
            return Err(zeckitError::ServiceNotReady("Zebra not ready".into()));
        }
    }
    println!();
    
    // [2/3] Backend with percentage
    if backend == "lwd" || backend == "zaino" {
        let backend_name = if backend == "lwd" { "Lightwalletd" } else { "Zaino" };
        let start = std::time::Instant::now();
        
        loop {
            pb.tick();
            
            if checker.wait_for_backend(&backend, &pb).await.is_ok() {
                println!("[2/3] {} ready (100%)", backend_name);
                break;
            }
            
            let elapsed = start.elapsed().as_secs();
            if elapsed < 180 {
                let progress = (elapsed as f64 / 180.0 * 100.0).min(99.0) as u32;
                print!("\r[2/3] Starting {}... {}%", backend_name, progress);
                io::stdout().flush().ok();
                sleep(Duration::from_secs(1)).await;
            } else {
                return Err(zeckitError::ServiceNotReady(format!("{} not ready", backend_name)));
            }
        }
        println!();
    }
    
    // [3/3] Faucet with percentage (faucet now contains zingolib)
    let start = std::time::Instant::now();
    loop {
        pb.tick();
        
        if checker.wait_for_faucet(&pb).await.is_ok() {
            println!("[3/3] Faucet ready (100%)");
            break;
        }
        
        let elapsed = start.elapsed().as_secs();
        if elapsed < 120 {
            let progress = (elapsed as f64 / 120.0 * 100.0).min(99.0) as u32;
            print!("\r[3/3] Starting Faucet... {}%", progress);
            io::stdout().flush().ok();
            sleep(Duration::from_secs(1)).await;
        } else {
            return Err(zeckitError::ServiceNotReady("Faucet not ready".into()));
        }
    }
    println!();
    
    pb.finish_and_clear();
    
    // GET WALLET ADDRESS FROM FAUCET API (not from zingo-wallet container)
    println!();
    println!("Configuring Zebra to mine to wallet...");
    
    match get_wallet_transparent_address_from_faucet().await {
        Ok(t_address) => {
            println!("Wallet transparent address: {}", t_address);
            
            if let Err(e) = update_zebra_miner_address(&t_address) {
                println!("{}", format!("Warning: Could not update zebra.toml: {}", e).yellow());
            } else {
                println!("Updated zebra.toml miner_address");
                
                println!("Restarting Zebra with new miner address...");
                if let Err(e) = restart_zebra().await {
                    println!("{}", format!("Warning: Zebra restart had issues: {}", e).yellow());
                }
            }
        }
        Err(e) => {
            println!("{}", format!("Warning: Could not get wallet address: {}", e).yellow());
            println!("  Mining will use default address in zebra.toml");
        }
    }
    
    // NOW WAIT FOR BLOCKS (mining to correct address)
    wait_for_mined_blocks(&pb, 101).await?;
    
    // Wait extra time for coinbase maturity
    println!();
    println!("Waiting for coinbase maturity (100 confirmations)...");
    sleep(Duration::from_secs(120)).await;
    
    // Generate UA fixtures from faucet API
    println!();
    println!("Generating ZIP-316 Unified Address fixtures...");
    
    match generate_ua_fixtures_from_faucet().await {
        Ok(address) => {
            println!("Generated UA: {}...", &address[..20]);
        }
        Err(e) => {
            println!("{}", format!("Warning: Could not generate UA fixture ({})", e).yellow());
            println!("  You can manually update fixtures/unified-addresses.json");
        }
    }
    
    // Sync wallet through faucet API
    println!();
    println!("Syncing wallet with blockchain...");
    if let Err(e) = sync_wallet_via_faucet().await {
        println!("{}", format!("Wallet sync warning: {}", e).yellow());
    } else {
        println!("Wallet synced with blockchain");
    }
    
    // Check balance
    println!();
    println!("Checking wallet balance...");
    match check_wallet_balance().await {
        Ok(balance) if balance > 0.0 => {
            println!("Wallet has {} ZEC available", balance);
        }
        Ok(_) => {
            println!("{}", "Wallet synced but balance not yet available".yellow());
            println!("  Blocks still maturing, wait a few more minutes");
        }
        Err(e) => {
            println!("{}", format!("Could not check balance: {}", e).yellow());
        }
    }
    
    print_connection_info(&backend);
    print_mining_info().await?;
    
    Ok(())
}

async fn wait_for_mined_blocks(pb: &ProgressBar, min_blocks: u64) -> Result<()> {
    let client = Client::new();
    let start = std::time::Instant::now();
    
    println!("Mining blocks to maturity...");
    
    loop {
        match get_block_count(&client).await {
            Ok(height) if height >= min_blocks => {
                println!("Mined {} blocks (coinbase maturity reached)", height);
                println!();
                return Ok(());
            }
            Ok(height) => {
                let progress = (height as f64 / min_blocks as f64 * 100.0) as u64;
                print!("\r  Block {} / {} ({}%)", height, min_blocks, progress);
                io::stdout().flush().ok();
            }
            Err(_) => {}
        }
        
        if start.elapsed().as_secs() > MAX_WAIT_SECONDS {
            return Err(zeckitError::ServiceNotReady(
                "Internal miner timeout - blocks not reaching maturity".into()
            ));
        }
        
        sleep(Duration::from_secs(2)).await;
    }
}

async fn get_block_count(client: &Client) -> Result<u64> {
    let resp = client
        .post("http://127.0.0.1:8232")
        .json(&json!({
            "jsonrpc": "2.0",
            "id": "blockcount",
            "method": "getblockcount",
            "params": []
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await?;
    
    let json: serde_json::Value = resp.json().await?;
    
    json.get("result")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| zeckitError::HealthCheck("Invalid block count response".into()))
}

// NEW: Get wallet address from faucet API instead of zingo-wallet container
async fn get_wallet_transparent_address_from_faucet() -> Result<String> {
    let client = Client::new();
    
    // Call faucet API to get transparent address
    let resp = client
        .get("http://127.0.0.1:8080/address")
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| zeckitError::HealthCheck(format!("Faucet API call failed: {}", e)))?;
    
    let json: serde_json::Value = resp.json().await?;
    
    json.get("transparent_address")
        .and_then(|v| v.as_str())
        .ok_or_else(|| zeckitError::HealthCheck("No transparent address in faucet response".into()))
        .map(|s| s.to_string())
}

fn update_zebra_miner_address(address: &str) -> Result<()> {
    let zebra_config_path = "docker/configs/zebra.toml";
    
    let config = fs::read_to_string(zebra_config_path)
        .map_err(|e| zeckitError::Config(format!("Could not read zebra.toml: {}", e)))?;
    
    let new_config = if config.contains("miner_address") {
        use regex::Regex;
        let re = Regex::new(r#"miner_address = "tm[a-zA-Z0-9]+""#).unwrap();
        re.replace(&config, format!("miner_address = \"{}\"", address)).to_string()
    } else {
        config.replace(
            "[mining]",
            &format!("[mining]\nminer_address = \"{}\"", address)
        )
    };
    
    fs::write(zebra_config_path, new_config)
        .map_err(|e| zeckitError::Config(format!("Could not write zebra.toml: {}", e)))?;
    
    Ok(())
}

async fn restart_zebra() -> Result<()> {
    let output = Command::new("docker")
        .args(&["restart", "zeckit-zebra"])
        .output()
        .map_err(|e| zeckitError::Docker(format!("Failed to restart Zebra: {}", e)))?;
    
    if !output.status.success() {
        return Err(zeckitError::Docker("Zebra restart failed".into()));
    }
    
    sleep(Duration::from_secs(15)).await;
    
    Ok(())
}

// NEW: Get UA from faucet API instead of zingo-wallet container
async fn generate_ua_fixtures_from_faucet() -> Result<String> {
    let client = Client::new();
    
    let resp = client
        .get("http://127.0.0.1:8080/address")
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| zeckitError::HealthCheck(format!("Faucet API call failed: {}", e)))?;
    
    let json: serde_json::Value = resp.json().await?;
    
    let ua_address = json.get("unified_address")
        .and_then(|v| v.as_str())
        .ok_or_else(|| zeckitError::HealthCheck("No unified address in faucet response".into()))?;
    
    let fixture = json!({
        "faucet_address": ua_address,
        "type": "unified",
        "receivers": ["orchard"]
    });
    
    fs::create_dir_all("fixtures")?;
    fs::write(
        "fixtures/unified-addresses.json",
        serde_json::to_string_pretty(&fixture)?
    )?;
    
    Ok(ua_address.to_string())
}

// NEW: Sync wallet via faucet API instead of zingo-wallet container
async fn sync_wallet_via_faucet() -> Result<()> {
    let client = Client::new();
    
    // Call faucet's sync endpoint
    let resp = client
        .post("http://127.0.0.1:8080/sync")
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| zeckitError::HealthCheck(format!("Faucet sync failed: {}", e)))?;
    
    if !resp.status().is_success() {
        return Err(zeckitError::HealthCheck("Wallet sync error via faucet API".into()));
    }
    
    Ok(())
}

async fn check_wallet_balance() -> Result<f64> {
    let client = Client::new();
    let resp = client
        .get("http://127.0.0.1:8080/stats")
        .timeout(Duration::from_secs(5))
        .send()
        .await?;
    
    let json: serde_json::Value = resp.json().await?;
    Ok(json["current_balance"].as_f64().unwrap_or(0.0))
}

async fn print_mining_info() -> Result<()> {
    let client = Client::new();
    
    if let Ok(height) = get_block_count(&client).await {
        println!();
        println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
        println!("{}", "  Blockchain Status".cyan().bold());
        println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
        println!();
        println!("  Block Height: {}", height);
        println!("  Network: Regtest");
        println!("  Mining: Active (internal miner)");
        println!("  Pre-mined Funds: Available");
    }
    
    Ok(())
}

fn print_connection_info(backend: &str) {
    println!();
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!("{}", "  Services Ready".cyan().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!();
    println!("  Zebra RPC: http://127.0.0.1:8232");
    println!("  Faucet API: http://127.0.0.1:8080");
    
    if backend == "lwd" {
        println!("  LightwalletD: http://127.0.0.1:9067");
    } else if backend == "zaino" {
        println!("  Zaino: http://127.0.0.1:9067");
    }
    
    println!();
    println!("Next steps:");
    println!("  • Run tests: zeckit test");
    println!("  • View fixtures: cat fixtures/unified-addresses.json");
    println!();
}