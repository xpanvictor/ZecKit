"""
ZecKit Faucet - Main Flask Application
Entry point for the faucet service
"""
from flask import Flask, jsonify
from flask_cors import CORS
from flask_limiter import Limiter
from flask_limiter.util import get_remote_address
import logging
import sys
from typing import Optional

from .config import get_config
from .zebra_rpc import ZebraRPCClient
from .wallet import FaucetWallet
from .routes.health import health_bp


def setup_logging(app: Flask) -> None:
    """Configure application logging"""
    log_level = app.config.get('LOG_LEVEL', 'INFO')
    log_format = app.config.get('LOG_FORMAT', 
                                 '%(asctime)s - %(name)s - %(levelname)s - %(message)s')
    
    # Configure root logger
    logging.basicConfig(
        level=getattr(logging, log_level),
        format=log_format,
        handlers=[
            logging.StreamHandler(sys.stdout)
        ]
    )
    
    # Set Flask logger
    app.logger.setLevel(getattr(logging, log_level))
    
    # Silence some noisy loggers
    logging.getLogger('werkzeug').setLevel(logging.WARNING)
    logging.getLogger('urllib3').setLevel(logging.WARNING)


def create_app(config_name: Optional[str] = None) -> Flask:
    """
    Flask application factory
    
    Args:
        config_name: Configuration environment (development/production/testing)
                    If None, reads from FLASK_ENV
    
    Returns:
        Configured Flask application
    """
    app = Flask(__name__)
    
    # Load configuration
    config_class = get_config(config_name)
    app.config.from_object(config_class)
    
    # Setup logging
    setup_logging(app)
    app.logger.info(f"Starting ZecKit Faucet (env: {config_name or 'default'})")
    
    # Initialize CORS
    CORS(app, origins=app.config.get('CORS_ORIGINS', '*'))
    
    # Initialize rate limiter
    limiter = Limiter(
        app=app,
        key_func=get_remote_address,
        default_limits=[] if not app.config.get('RATE_LIMIT_ENABLED') else [
            f"{app.config.get('RATE_LIMIT_REQUESTS')}/{app.config.get('RATE_LIMIT_WINDOW')} seconds"
        ],
        storage_uri="memory://"  # In-memory storage for now
    )
    app.limiter = limiter
    
    # Initialize Zebra RPC client
    try:
        app.logger.info(f"Connecting to Zebra at {app.config['ZEBRA_RPC_URL']}")
        app.zebra_client = ZebraRPCClient(
            url=app.config['ZEBRA_RPC_URL'],
            username=app.config.get('ZEBRA_RPC_USER'),
            password=app.config.get('ZEBRA_RPC_PASS'),
            timeout=app.config.get('ZEBRA_RPC_TIMEOUT', 30)
        )
        
        # Test connection
        if app.zebra_client.ping():
            app.logger.info("✓ Connected to Zebra")
            height = app.zebra_client.get_block_count()
            app.logger.info(f"✓ Current block height: {height}")
        else:
            app.logger.warning("⚠ Could not connect to Zebra (will retry)")
    
    except Exception as e:
        app.logger.error(f"✗ Failed to initialize Zebra client: {e}")
        app.logger.warning("Faucet will start but may not be functional")
        app.zebra_client = None
    
    # Initialize wallet
    try:
        app.logger.info("Initializing faucet wallet...")
        app.faucet_wallet = FaucetWallet(
            zebra_client=app.zebra_client,
            wallet_file=app.config.get('WALLET_FILE')
        )
        
        if app.faucet_wallet.is_loaded():
            balance = app.faucet_wallet.get_balance()
            address = app.faucet_wallet.get_address()
            app.logger.info(f"✓ Wallet loaded")
            app.logger.info(f"✓ Faucet address: {address}")
            app.logger.info(f"✓ Balance: {balance} ZEC")
            
            # Check if balance is low
            low_threshold = app.config.get('FAUCET_LOW_BALANCE_THRESHOLD', 100.0)
            if balance < low_threshold:
                app.logger.warning(f"⚠ Low balance! ({balance} < {low_threshold} ZEC)")
        else:
            app.logger.warning("⚠ Wallet not loaded (will try to create)")
    
    except Exception as e:
        app.logger.error(f"✗ Failed to initialize wallet: {e}")
        app.logger.warning("Faucet will start but may not be functional")
        app.faucet_wallet = None
    
    # Register blueprints
    app.register_blueprint(health_bp)
    
    # Import and register other routes here as we create them
    # from .routes.faucet import faucet_bp
    # app.register_blueprint(faucet_bp)
    # from .routes.stats import stats_bp
    # app.register_blueprint(stats_bp)
    
    # Error handlers
    @app.errorhandler(404)
    def not_found(error):
        return jsonify({
            "error": "Not found",
            "code": "NOT_FOUND"
        }), 404
    
    @app.errorhandler(500)
    def internal_error(error):
        app.logger.error(f"Internal error: {error}")
        return jsonify({
            "error": "Internal server error",
            "code": "INTERNAL_ERROR"
        }), 500
    
    @app.errorhandler(429)
    def ratelimit_handler(error):
        return jsonify({
            "error": "Rate limit exceeded",
            "code": "RATE_LIMIT_EXCEEDED",
            "message": str(error.description)
        }), 429
    
    # Root endpoint
    @app.route('/')
    def index():
        return jsonify({
            "service": "ZecKit Faucet",
            "version": "0.1.0",
            "status": "running",
            "endpoints": {
                "health": "/health",
                "ready": "/ready",
                "live": "/live",
                "request_funds": "/request (POST)",
                "stats": "/stats (coming soon)"
            }
        })
    
    app.logger.info("✓ ZecKit Faucet initialized successfully")
    return app


# For running directly with python -m app.main
if __name__ == '__main__':
    app = create_app()
    app.run(
        host='0.0.0.0',
        port=8080,
        debug=app.config.get('DEBUG', False)
    )