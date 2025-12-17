use crate::error::{Result, ZecDevError};
use reqwest::Client;
use indicatif::ProgressBar;
use tokio::time::{sleep, Duration};
use serde_json::Value;
use std::net::TcpStream;
use std::time::Duration as StdDuration;

pub struct HealthChecker {
    client: Client,
    max_retries: u32,
    retry_delay: Duration,
    backend_max_retries: u32,
}

impl HealthChecker {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            max_retries: 60,
            retry_delay: Duration::from_secs(2),
            backend_max_retries: 90,
        }
    }

    pub async fn wait_for_zebra(&self, pb: &ProgressBar) -> Result<()> {
        for i in 0..self.max_retries {
            pb.tick();
            
            match self.check_zebra().await {
                Ok(_) => return Ok(()),
                Err(_) if i < self.max_retries - 1 => {
                    sleep(self.retry_delay).await;
                }
                Err(e) => return Err(e),
            }
        }

        Err(ZecDevError::ServiceNotReady("Zebra".into()))
    }

    pub async fn wait_for_faucet(&self, pb: &ProgressBar) -> Result<()> {
        for i in 0..self.max_retries {
            pb.tick();
            
            match self.check_faucet().await {
                Ok(_) => return Ok(()),
                Err(_) if i < self.max_retries - 1 => {
                    sleep(self.retry_delay).await;
                }
                Err(e) => return Err(e),
            }
        }

        Err(ZecDevError::ServiceNotReady("Faucet".into()))
    }

    pub async fn wait_for_backend(&self, backend: &str, pb: &ProgressBar) -> Result<()> {
        for i in 0..self.backend_max_retries {
            pb.tick();
            
            match self.check_backend(backend).await {
                Ok(_) => return Ok(()),
                Err(_) if i < self.backend_max_retries - 1 => {
                    sleep(self.retry_delay).await;
                }
                Err(e) => return Err(e),
            }
        }

        Err(ZecDevError::ServiceNotReady(format!("{} not ready", backend)))
    }

    async fn check_zebra(&self) -> Result<()> {
        let resp = self
            .client
            .post("http://127.0.0.1:8232")
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": "health",
                "method": "getblockcount",
                "params": []
            }))
            .timeout(Duration::from_secs(5))
            .send()
            .await?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(ZecDevError::HealthCheck("Zebra not ready".into()))
        }
    }

    async fn check_faucet(&self) -> Result<()> {
        let resp = self
            .client
            .get("http://127.0.0.1:8080/health")
            .timeout(Duration::from_secs(5))
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(ZecDevError::HealthCheck("Faucet not ready".into()));
        }

        let json: Value = resp.json().await?;
        
        if json.get("status").and_then(|s| s.as_str()) == Some("unhealthy") {
            return Err(ZecDevError::HealthCheck("Faucet unhealthy".into()));
        }

        Ok(())
    }

    async fn check_backend(&self, backend: &str) -> Result<()> {
        // Zaino and Lightwalletd are gRPC services on port 9067
        // They don't respond to HTTP, so we do a TCP connection check
        
        let backend_name = if backend == "lwd" { "lightwalletd" } else { "zaino" };
        
        // Try to connect to localhost:9067 with 2 second timeout
        match TcpStream::connect_timeout(
            &"127.0.0.1:9067".parse().unwrap(),
            StdDuration::from_secs(2)
        ) {
            Ok(_) => {
                // Port is open and accepting connections - backend is ready!
                Ok(())
            }
            Err(_) => {
                // Port not accepting connections yet
                Err(ZecDevError::HealthCheck(format!("{} not ready", backend_name)))
            }
        }
    }
}