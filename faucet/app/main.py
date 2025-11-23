"""
ZecKit Faucet - Main Application (REAL Transactions via Zingo-CLI)
"""
from flask import Flask, jsonify
from flask_cors import CORS
from datetime import datetime
import logging
import sys

from .config import get_config
from .wallet import get_wallet
from .routes.health import health_bp
from .routes.faucet import faucet_bp
from .routes.stats import stats_bp


def setup_logging(log_level: str = "INFO"):
    """Configure application logging"""
    logging.basicConfig(
        level=getattr(logging, log_level.upper()),
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
        handlers=[logging.StreamHandler(sys.stdout)]
    )


def create_app(config_name: str = None) -> Flask:
    """Application factory"""
    app = Flask(__name__)
    
    # Load configuration
    config = get_config(config_name)
    app.config.from_object(config)
    
    # Setup logging
    setup_logging(app.config['LOG_LEVEL'])
    logger = logging.getLogger(__name__)
    
    # Enable CORS
    CORS(app, origins=app.config['CORS_ORIGINS'])
    
    # Initialize Wallet (Zingo-CLI wrapper)
    try:
        app.faucet_wallet = get_wallet()
        balance = app.faucet_wallet.get_balance()
        address = app.faucet_wallet.get_address("unified")
        
        logger.info(f"✓ Faucet wallet loaded (ZingoLib)")
        logger.info(f"  Address: {address}")
        logger.info(f"  Balance: {balance} ZEC")
        
    except Exception as e:
        logger.error(f"Failed to initialize wallet: {e}")
        app.faucet_wallet = None
    
    # Register blueprints
    app.register_blueprint(health_bp)
    app.register_blueprint(faucet_bp)
    app.register_blueprint(stats_bp)
    
    # Root endpoint
    @app.route('/', methods=['GET'])
    def root():
        return jsonify({
            "name": "ZecKit Faucet",
            "version": "0.2.0",
            "description": "Zcash Regtest Faucet with REAL transactions (ZingoLib)",
            "transaction_mode": "REAL_BLOCKCHAIN",
            "wallet_backend": "zingo-cli",
            "endpoints": {
                "health": "/health",
                "stats": "/stats",
                "request": "/request",
                "address": "/address",
                "sync": "/sync",
                "history": "/history"
            }
        }), 200
    
    # Store app start time
    app.start_time = datetime.utcnow()
    
    logger.info("✓ ZecKit Faucet initialized (REAL TRANSACTIONS)")
    
    return app


if __name__ == '__main__':
    app = create_app()
    app.run(host='0.0.0.0', port=8080, debug=True)