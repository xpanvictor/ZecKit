use axum::{
    Router,
    routing::{get, post},
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod wallet;
mod api;
mod validation;
mod error;

use config::Config;
use wallet::WalletManager;

#[derive(Clone)]
pub struct AppState {
    pub wallet: Arc<RwLock<WalletManager>>,
    pub config: Arc<Config>,
    pub start_time: chrono::DateTime<chrono::Utc>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "zeckit_faucet=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("🚀 Starting ZecKit Faucet v0.3.0");

    // Load configuration
    let config = Config::load()?;
    info!("📋 Configuration loaded");
    info!("  Network: regtest");
    info!("  LightwalletD URI: {}", config.lightwalletd_uri);
    info!("  Data dir: {}", config.zingo_data_dir.display());

    // Initialize wallet manager
    info!("💼 Initializing wallet...");
    let wallet = WalletManager::new(
        config.zingo_data_dir.clone(),
        config.lightwalletd_uri.clone(),
    ).await?;

    let wallet = Arc::new(RwLock::new(wallet));

    // Get initial wallet info
    {
        let wallet_lock = wallet.read().await;
        let address = wallet_lock.get_unified_address().await?;
        let balance = wallet_lock.get_balance().await?;
        
        info!("✅ Wallet initialized");
        info!("  Address: {}", address);
        info!("  Balance: {} ZEC", balance.total_zec());
    }

    // Build application state
    let state = AppState {
        wallet,
        config: Arc::new(config.clone()),
        start_time: chrono::Utc::now(),
    };

    // Build router
    let app = Router::new()
        .route("/", get(api::root))
        .route("/health", get(api::health::health_check))
        .route("/stats", get(api::stats::get_stats))
        .route("/history", get(api::stats::get_history))
        .route("/request", post(api::faucet::request_funds))
        .route("/address", get(api::faucet::get_faucet_address))
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    info!("🌐 Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}