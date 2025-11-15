"""
ZecKit Faucet - Health Check Endpoint
Provides health status for monitoring and orchestration
"""
from flask import Blueprint, jsonify, current_app
from datetime import datetime
import logging

logger = logging.getLogger(__name__)

health_bp = Blueprint('health', __name__)


@health_bp.route('/health', methods=['GET'])
def health_check():
    """
    Health check endpoint
    
    Returns:
        JSON with health status:
        - status: "healthy" | "degraded" | "unhealthy"
        - zebra_connected: bool
        - zebra_height: int (if connected)
        - wallet_loaded: bool
        - balance: float (if wallet loaded)
        - version: str
        - timestamp: str
    
    Status Codes:
        200: Healthy
        503: Unhealthy
    """
    health_status = {
        "status": "healthy",
        "zebra_connected": False,
        "zebra_height": None,
        "wallet_loaded": False,
        "balance": None,
        "version": "0.1.0",
        "timestamp": datetime.utcnow().isoformat() + "Z"
    }
    
    issues = []
    
    # Check Zebra connection
    try:
        zebra_client = current_app.zebra_client
        
        if zebra_client.ping():
            health_status["zebra_connected"] = True
            
            # Get block height
            try:
                height = zebra_client.get_block_count()
                health_status["zebra_height"] = height
            except Exception as e:
                logger.warning(f"Could not get block height: {e}")
                issues.append("block_height_unavailable")
        else:
            issues.append("zebra_not_responding")
    
    except AttributeError:
        logger.error("Zebra client not initialized")
        issues.append("zebra_client_not_initialized")
    except Exception as e:
        logger.error(f"Zebra health check failed: {e}")
        issues.append("zebra_connection_error")
    
    # Check wallet
    try:
        wallet = current_app.faucet_wallet
        
        if wallet and wallet.is_loaded():
            health_status["wallet_loaded"] = True
            
            # Get balance
            try:
                balance = wallet.get_balance()
                health_status["balance"] = balance
                
                # Check if balance is low
                low_threshold = current_app.config.get('FAUCET_LOW_BALANCE_THRESHOLD', 100.0)
                if balance < low_threshold:
                    issues.append(f"low_balance_{balance}")
                    health_status["status"] = "degraded"
            
            except Exception as e:
                logger.warning(f"Could not get balance: {e}")
                issues.append("balance_check_failed")
        else:
            issues.append("wallet_not_loaded")
    
    except AttributeError:
        logger.error("Wallet not initialized")
        issues.append("wallet_not_initialized")
    except Exception as e:
        logger.error(f"Wallet health check failed: {e}")
        issues.append("wallet_error")
    
    # Determine overall status
    if not health_status["zebra_connected"] or not health_status["wallet_loaded"]:
        health_status["status"] = "unhealthy"
        status_code = 503
    elif issues and health_status["status"] != "degraded":
        health_status["status"] = "degraded"
        status_code = 200
    else:
        status_code = 200
    
    # Add issues to response if any
    if issues:
        health_status["issues"] = issues
    
    logger.debug(f"Health check: {health_status['status']} - {issues if issues else 'no issues'}")
    
    return jsonify(health_status), status_code


@health_bp.route('/ready', methods=['GET'])
def readiness_check():
    """
    Readiness check for Kubernetes/Docker
    Simple check that returns 200 if service can accept requests
    
    Returns:
        200: Ready
        503: Not ready
    """
    try:
        # Quick checks only
        zebra_client = current_app.zebra_client
        wallet = current_app.faucet_wallet
        
        if zebra_client.ping() and wallet and wallet.is_loaded():
            return jsonify({"ready": True}), 200
        
        return jsonify({"ready": False, "reason": "services_not_ready"}), 503
    
    except Exception as e:
        logger.error(f"Readiness check failed: {e}")
        return jsonify({"ready": False, "reason": str(e)}), 503


@health_bp.route('/live', methods=['GET'])
def liveness_check():
    """
    Liveness check for Kubernetes/Docker
    Simple check that returns 200 if service is alive
    
    Returns:
        200: Always (unless Flask is dead)
    """
    return jsonify({"alive": True}), 200