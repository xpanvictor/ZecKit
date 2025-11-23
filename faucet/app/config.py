"""
ZecKit Faucet - Configuration Management
"""
import os


class BaseConfig:
    """Base configuration"""
    
    # Flask
    SECRET_KEY = os.environ.get('SECRET_KEY', 'dev-secret-change-in-production')
    JSON_SORT_KEYS = False
    
    # Zingo Wallet
    ZINGO_DATA_DIR = os.environ.get('ZINGO_DATA_DIR', '/var/zingo')
    ZINGO_CLI_PATH = os.environ.get('ZINGO_CLI_PATH', '/usr/local/bin/zingo-cli')
    LIGHTWALLETD_URI = os.environ.get('LIGHTWALLETD_URI', 'http://lightwalletd:9067')
    
    # Faucet Limits
    FAUCET_AMOUNT_MIN = float(os.environ.get('FAUCET_AMOUNT_MIN', '1.0'))
    FAUCET_AMOUNT_MAX = float(os.environ.get('FAUCET_AMOUNT_MAX', '100.0'))
    FAUCET_AMOUNT_DEFAULT = float(os.environ.get('FAUCET_AMOUNT_DEFAULT', '10.0'))
    
    # CORS
    CORS_ORIGINS = os.environ.get('CORS_ORIGINS', '*').split(',')
    
    # Logging
    LOG_LEVEL = os.environ.get('LOG_LEVEL', 'INFO')


class DevelopmentConfig(BaseConfig):
    """Development configuration"""
    DEBUG = True
    LOG_LEVEL = 'DEBUG'


class ProductionConfig(BaseConfig):
    """Production configuration"""
    DEBUG = False


config_map = {
    'development': DevelopmentConfig,
    'production': ProductionConfig,
    'default': DevelopmentConfig
}


def get_config(env=None):
    """Get configuration class"""
    if env is None:
        env = os.environ.get('FLASK_ENV', 'development')
    return config_map.get(env.lower(), DevelopmentConfig)