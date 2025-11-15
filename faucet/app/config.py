"""
ZecKit Faucet - Configuration Management
Handles environment-based configuration for the faucet service
"""
import os
from typing import Optional


class BaseConfig:
    """Base configuration - common settings for all environments"""
    
    # Flask
    SECRET_KEY = os.environ.get('SECRET_KEY', 'dev-secret-change-in-production')
    JSON_SORT_KEYS = False
    
    # Zebra RPC Connection
    ZEBRA_RPC_URL = os.environ.get('ZEBRA_RPC_URL', 'http://127.0.0.1:8232')
    ZEBRA_RPC_USER = os.environ.get('ZEBRA_RPC_USER', '')
    ZEBRA_RPC_PASS = os.environ.get('ZEBRA_RPC_PASS', '')
    ZEBRA_RPC_TIMEOUT = int(os.environ.get('ZEBRA_RPC_TIMEOUT', '30'))
    
    # Faucet Limits
    FAUCET_AMOUNT_MIN = float(os.environ.get('FAUCET_AMOUNT_MIN', '1.0'))
    FAUCET_AMOUNT_MAX = float(os.environ.get('FAUCET_AMOUNT_MAX', '100.0'))
    FAUCET_AMOUNT_DEFAULT = float(os.environ.get('FAUCET_AMOUNT_DEFAULT', '10.0'))
    FAUCET_LOW_BALANCE_THRESHOLD = float(os.environ.get('FAUCET_LOW_BALANCE_THRESHOLD', '100.0'))
    
    # Rate Limiting (requests per window)
    RATE_LIMIT_ENABLED = os.environ.get('RATE_LIMIT_ENABLED', 'true').lower() == 'true'
    RATE_LIMIT_REQUESTS = int(os.environ.get('RATE_LIMIT_REQUESTS', '10'))
    RATE_LIMIT_WINDOW = int(os.environ.get('RATE_LIMIT_WINDOW', '3600'))  # 1 hour in seconds
    
    # CORS
    CORS_ORIGINS = os.environ.get('CORS_ORIGINS', '*').split(',')
    
    # Logging
    LOG_LEVEL = os.environ.get('LOG_LEVEL', 'INFO')
    LOG_FORMAT = '%(asctime)s - %(name)s - %(levelname)s - %(message)s'
    
    # Health Check
    HEALTH_CHECK_ZEBRA = True
    HEALTH_CHECK_WALLET = True
    
    # Wallet (simple file-based for now)
    WALLET_FILE = os.environ.get('WALLET_FILE', '/var/faucet/wallet.json')
    
    @classmethod
    def validate(cls) -> bool:
        """Validate critical configuration"""
        if not cls.ZEBRA_RPC_URL:
            raise ValueError("ZEBRA_RPC_URL must be set")
        
        if cls.FAUCET_AMOUNT_MIN <= 0:
            raise ValueError("FAUCET_AMOUNT_MIN must be positive")
        
        if cls.FAUCET_AMOUNT_MAX < cls.FAUCET_AMOUNT_MIN:
            raise ValueError("FAUCET_AMOUNT_MAX must be >= FAUCET_AMOUNT_MIN")
        
        if cls.FAUCET_AMOUNT_DEFAULT < cls.FAUCET_AMOUNT_MIN or \
           cls.FAUCET_AMOUNT_DEFAULT > cls.FAUCET_AMOUNT_MAX:
            raise ValueError("FAUCET_AMOUNT_DEFAULT must be between MIN and MAX")
        
        return True


class DevelopmentConfig(BaseConfig):
    """Development configuration"""
    DEBUG = True
    TESTING = False
    LOG_LEVEL = 'DEBUG'
    
    # More lenient rate limiting for dev
    RATE_LIMIT_REQUESTS = 100
    RATE_LIMIT_WINDOW = 60  # 1 minute


class ProductionConfig(BaseConfig):
    """Production configuration"""
    DEBUG = False
    TESTING = False
    
    # Stricter limits
    RATE_LIMIT_REQUESTS = 5
    RATE_LIMIT_WINDOW = 3600  # 1 hour
    
    # Production should use a secure secret key
    @classmethod
    def validate(cls) -> bool:
        super().validate()
        if cls.SECRET_KEY == 'dev-secret-change-in-production':
            raise ValueError("Must set SECRET_KEY in production")
        return True


class TestConfig(BaseConfig):
    """Testing configuration"""
    TESTING = True
    DEBUG = True
    
    # Disable rate limiting for tests
    RATE_LIMIT_ENABLED = False
    
    # Use in-memory or test-specific paths
    WALLET_FILE = '/tmp/faucet-test-wallet.json'
    
    # Shorter timeouts for faster tests
    ZEBRA_RPC_TIMEOUT = 5


# Config selection
config_map = {
    'development': DevelopmentConfig,
    'production': ProductionConfig,
    'testing': TestConfig,
    'default': DevelopmentConfig
}


def get_config(env: Optional[str] = None) -> type[BaseConfig]:
    """
    Get configuration class based on environment
    
    Args:
        env: Environment name (development/production/testing)
             If None, reads from FLASK_ENV or defaults to development
    
    Returns:
        Configuration class
    """
    if env is None:
        env = os.environ.get('FLASK_ENV', 'development')
    
    config_class = config_map.get(env.lower(), DevelopmentConfig)
    
    # Validate configuration
    config_class.validate()
    
    return config_class