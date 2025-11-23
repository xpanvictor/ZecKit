"""
ZecKit Faucet - Statistics Endpoint (REAL Transactions)
"""
from flask import Blueprint, jsonify, current_app, request
from datetime import datetime

stats_bp = Blueprint('stats', __name__)


def _format_uptime(seconds: float) -> str:
    """Convert seconds to readable format"""
    if seconds < 0:
        return "0s"

    days = int(seconds // 86400)
    hours = int((seconds % 86400) // 3600)
    minutes = int((seconds % 3600) // 60)
    secs = int(seconds % 60)

    parts = []
    if days:   parts.append(f"{days}d")
    if hours:  parts.append(f"{hours}h")
    if minutes: parts.append(f"{minutes}m")
    parts.append(f"{secs}s")
    return " ".join(parts)


@stats_bp.route('/stats', methods=['GET'])
def get_stats():
    """Get faucet statistics"""
    wallet = current_app.faucet_wallet
    
    if not wallet:
        return jsonify({"error": "Faucet wallet not available"}), 503
    
    wallet_stats = wallet.get_stats()
    tx_history = wallet.get_transaction_history()
    
    # Get last request timestamp
    last_request = tx_history[-1].get('timestamp') if tx_history else None
    
    # Calculate uptime
    uptime_seconds = (datetime.utcnow() - current_app.start_time).total_seconds()

    return jsonify({
        "faucet_address": wallet_stats['faucet_address'],
        "current_balance": wallet_stats['current_balance'],
        "total_requests": wallet_stats['total_requests'],
        "total_sent": wallet_stats['total_sent'],
        "last_request": last_request,
        "uptime": _format_uptime(uptime_seconds),
        "uptime_seconds": int(uptime_seconds),
        "transaction_mode": "REAL_BLOCKCHAIN",
        "wallet_backend": "zingo-cli",
        "version": "0.2.0"
    }), 200


@stats_bp.route('/history', methods=['GET'])
def get_history():
    """Get transaction history"""
    wallet = current_app.faucet_wallet
    
    if not wallet:
        return jsonify({"error": "Faucet wallet not available"}), 503
    
    try:
        limit = int(request.args.get('limit', 100))
        limit = min(max(1, limit), 1000)
    except ValueError:
        limit = 100
    
    history = wallet.get_transaction_history()
    
    return jsonify({
        "count": len(history),
        "limit": limit,
        "transactions": history[-limit:]
    }), 200