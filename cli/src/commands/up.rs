use crate::docker::compose::DockerCompose;
use crate::docker::health::HealthChecker;
use crate::error::{Result, ZecDevError};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use serde_json::json;
use std::process::Command;
use std::fs;
use std::io::{self, Write};
use tokio::time::{sleep, Duration};

const MAX_WAIT_SECONDS: u64 = 30000;

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
            return Err(ZecDevError::Config(format!(
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
        
        println!("[1/4] Building Zebra...");
        println!("[2/4] Building Lightwalletd...");
        println!("[3/4] Building Zingo Wallet...");
        println!("[4/4] Building Faucet...");
        
        compose.up_with_profile("lwd")?;
        println!();
    } else if backend == "zaino" {
        println!("Building Docker images...");
        println!();
        
        println!("[1/4] Building Zebra...");
        println!("[2/4] Building Zaino...");
        println!("[3/4] Building Zingo Wallet...");
        println!("[4/4] Building Faucet...");
        
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
    
    // [1/4] Zebra with percentage
    let checker = HealthChecker::new();
    let start = std::time::Instant::now();
    
    loop {
        pb.tick();
        
        if checker.wait_for_zebra(&pb).await.is_ok() {
            println!("[1/4] Zebra ready (100%)");
            break;
        }
        
        let elapsed = start.elapsed().as_secs();
        if elapsed < 120 {
            let progress = (elapsed as f64 / 120.0 * 100.0).min(99.0) as u32;
            print!("\r[1/4] Starting Zebra... {}%", progress);
            io::stdout().flush().ok();
            sleep(Duration::from_secs(1)).await;
        } else {
            return Err(ZecDevError::ServiceNotReady("Zebra not ready".into()));
        }
    }
    println!();
    
    // [2/4] Backend with percentage
    if backend == "lwd" || backend == "zaino" {
        let backend_name = if backend == "lwd" { "Lightwalletd" } else { "Zaino" };
        let start = std::time::Instant::now();
        
        loop {
            pb.tick();
            
            if checker.wait_for_backend(&backend, &pb).await.is_ok() {
                println!("[2/4] {} ready (100%)", backend_name);
                break;
            }
            
            let elapsed = start.elapsed().as_secs();
            if elapsed < 180 {
                let progress = (elapsed as f64 / 180.0 * 100.0).min(99.0) as u32;
                print!("\r[2/4] Starting {}... {}%", backend_name, progress);
                io::stdout().flush().ok();
                sleep(Duration::from_secs(1)).await;
            } else {
                return Err(ZecDevError::ServiceNotReady(format!("{} not ready", backend_name)));
            }
        }
        println!();
    }
    
    // [3/4] Wallet with percentage
    let backend_uri = if backend == "lwd" {
        "http://lightwalletd:9067"
    } else if backend == "zaino" {
        "http://zaino:9067"
    } else {
        "http://lightwalletd:9067"
    };
    
    let start = std::time::Instant::now();
    loop {
        pb.tick();
        
        if wait_for_wallet_ready(&pb, backend_uri).await.is_ok() {
            println!("[3/4] Zingo Wallet ready (100%)");
            break;
        }
        
        let elapsed = start.elapsed().as_secs();
        if elapsed < 60 {
            let progress = (elapsed as f64 / 60.0 * 100.0).min(99.0) as u32;
            print!("\r[3/4] Starting Zingo Wallet... {}%", progress);
            io::stdout().flush().ok();
            sleep(Duration::from_secs(1)).await;
        } else {
            return Err(ZecDevError::ServiceNotReady("Wallet not ready".into()));
        }
    }
    println!();
    
    // [4/4] Faucet with percentage
    let start = std::time::Instant::now();
    loop {
        pb.tick();
        
        if checker.wait_for_faucet(&pb).await.is_ok() {
            println!("[4/4] Faucet ready (100%)");
            break;
        }
        
        let elapsed = start.elapsed().as_secs();
        if elapsed < 60 {
            let progress = (elapsed as f64 / 60.0 * 100.0).min(99.0) as u32;
            print!("\r[4/4] Starting Faucet... {}%", progress);
            io::stdout().flush().ok();
            sleep(Duration::from_secs(1)).await;
        } else {
            return Err(ZecDevError::ServiceNotReady("Faucet not ready".into()));
        }
    }
    println!();
    
    pb.finish_and_clear();
    
    // GET WALLET ADDRESS AND UPDATE ZEBRA CONFIG
    println!();
    println!("Configuring Zebra to mine to wallet...");
    
    match get_wallet_transparent_address(backend_uri).await {
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
    
    // Generate UA fixtures
    println!();
    println!("Generating ZIP-316 Unified Address fixtures...");
    
    match generate_ua_fixtures(backend_uri).await {
        Ok(address) => {
            println!("Generated UA: {}...", &address[..20]);
        }
        Err(e) => {
            println!("{}", format!("Warning: Could not generate UA fixture ({})", e).yellow());
            println!("  You can manually update fixtures/unified-addresses.json");
        }
    }
    
    // Sync wallet
    println!();
    println!("Syncing wallet with blockchain...");
    if let Err(e) = sync_wallet(backend_uri).await {
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

async fn wait_for_wallet_ready(pb: &ProgressBar, backend_uri: &str) -> Result<()> {
    let start = std::time::Instant::now();
    
    loop {
        pb.tick();
        
        let cmd_str = format!(
            "bash -c \"echo -e 't_addresses\\nquit' | zingo-cli --data-dir /var/zingo --server {} --chain regtest --nosync 2>&1\"",
            backend_uri
        );
        
        let output = Command::new("docker")
            .args(&["exec", "zeckit-zingo-wallet", "bash", "-c", &cmd_str])
            .output();
        
        if let Ok(out) = output {
            let output_str = String::from_utf8_lossy(&out.stdout);
            if output_str.contains("tm") && output_str.contains("encoded_address") {
                return Ok(());
            }
        }
        
        if start.elapsed().as_secs() > 60 {
            return Err(ZecDevError::ServiceNotReady("Wallet not ready after 60s".into()));
        }
        
        sleep(Duration::from_secs(2)).await;
    }
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
            return Err(ZecDevError::ServiceNotReady(
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
        .ok_or_else(|| ZecDevError::HealthCheck("Invalid block count response".into()))
}

async fn get_wallet_transparent_address(backend_uri: &str) -> Result<String> {
    let cmd_str = format!(
        "bash -c \"echo -e 't_addresses\\nquit' | zingo-cli --data-dir /var/zingo --server {} --chain regtest --nosync 2>&1\"",
        backend_uri
    );
    
    let output = Command::new("docker")
        .args(&["exec", "zeckit-zingo-wallet", "bash", "-c", &cmd_str])
        .output()
        .map_err(|e| ZecDevError::HealthCheck(format!("Docker exec failed: {}", e)))?;
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    
    for line in output_str.lines() {
        if line.contains("\"encoded_address\"") && line.contains("tm") {
            if let Some(start) = line.find("tm") {
                let addr_part = &line[start..];
                let end = addr_part.find(|c: char| c == '"' || c == '\n' || c == ' ')
                    .unwrap_or(addr_part.len());
                let address = &addr_part[..end];
                
                if address.starts_with("tm") && address.len() > 30 {
                    return Ok(address.to_string());
                }
            }
        }
    }
    
    Err(ZecDevError::HealthCheck("Could not find transparent address in wallet output".into()))
}

fn update_zebra_miner_address(address: &str) -> Result<()> {
    let zebra_config_path = "docker/configs/zebra.toml";
    
    let config = fs::read_to_string(zebra_config_path)
        .map_err(|e| ZecDevError::Config(format!("Could not read zebra.toml: {}", e)))?;
    
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
        .map_err(|e| ZecDevError::Config(format!("Could not write zebra.toml: {}", e)))?;
    
    Ok(())
}

async fn restart_zebra() -> Result<()> {
    let output = Command::new("docker")
        .args(&["restart", "zeckit-zebra"])
        .output()
        .map_err(|e| ZecDevError::Docker(format!("Failed to restart Zebra: {}", e)))?;
    
    if !output.status.success() {
        return Err(ZecDevError::Docker("Zebra restart failed".into()));
    }
    
    sleep(Duration::from_secs(15)).await;
    
    Ok(())
}

async fn generate_ua_fixtures(backend_uri: &str) -> Result<String> {
    let cmd_str = format!(
        "bash -c \"echo -e 'addresses\\nquit' | zingo-cli --data-dir /var/zingo --server {} --chain regtest --nosync 2>&1\"",
        backend_uri
    );
    
    let output = Command::new("docker")
        .args(&["exec", "zeckit-zingo-wallet", "bash", "-c", &cmd_str])
        .output()
        .map_err(|e| ZecDevError::HealthCheck(format!("Docker exec failed: {}", e)))?;
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    
    for line in output_str.lines() {
        if line.contains("uregtest") {
            if let Some(start) = line.find("uregtest") {
                let addr_part = &line[start..];
                let end = addr_part.find(|c: char| c == '"' || c == '\n' || c == ' ')
                    .unwrap_or(addr_part.len());
                let address = &addr_part[..end];
                
                let fixture = json!({
                    "faucet_address": address,
                    "type": "unified",
                    "receivers": ["orchard"]
                });
                
                fs::create_dir_all("fixtures")?;
                fs::write(
                    "fixtures/unified-addresses.json",
                    serde_json::to_string_pretty(&fixture)?
                )?;
                
                return Ok(address.to_string());
            }
        }
    }
    
    Err(ZecDevError::HealthCheck("Could not find wallet address in output".into()))
}

async fn sync_wallet(backend_uri: &str) -> Result<()> {
    let cmd_str = format!(
        "echo 'sync run\nquit' | zingo-cli --data-dir /var/zingo --server {} --chain regtest 2>&1",
        backend_uri
    );
    
    let output = Command::new("docker")
        .args(&[
            "exec", "-i", "zeckit-zingo-wallet",
            "sh", "-c",
            &cmd_str
        ])
        .output()
        .map_err(|e| ZecDevError::HealthCheck(format!("Sync command failed: {}", e)))?;
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    
    if output_str.contains("Sync error") {
        Err(ZecDevError::HealthCheck("Wallet sync error detected".into()))
    } else {
        Ok(())
    }
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
    println!("  • Run tests: zecdev test");
    println!("  • View fixtures: cat fixtures/unified-addresses.json");
    println!();
}
