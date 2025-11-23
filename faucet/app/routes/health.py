"""
ZecKit Faucet - Health Check Endpoint
"""
from flask import Blueprint, jsonify, current_app
from datetime import datetime

health_bp = Blueprint('health', __name__)


@health_bp.route('/health', methods=['GET'])
def health_check():
    """
    Health check endpoint for Kubernetes/Docker
    """
    wallet = current_app.faucet_wallet
    
    if not wallet:
        return jsonify({
            "status": "unhealthy",
            "error": "Wallet not initialized",
            "timestamp": datetime.utcnow().isoformat() + "Z"
        }), 503
    
    try:
        balance = wallet.get_balance()
        
        return jsonify({
            "status": "healthy",
            "wallet_backend": "zingo-cli",
            "transaction_mode": "REAL_BLOCKCHAIN",
            "balance": float(balance),
            "timestamp": datetime.utcnow().isoformat() + "Z",
            "version": "0.2.0"
        }), 200
        
    except Exception as e:
        return jsonify({
            "status": "unhealthy",
            "error": str(e),
            "timestamp": datetime.utcnow().isoformat() + "Z"
        }), 503