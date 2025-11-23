"""
ZecKit Faucet - Funding Request Endpoint (REAL Transactions)
"""
from flask import Blueprint, jsonify, request, current_app
from datetime import datetime
import re

faucet_bp = Blueprint('faucet', __name__)


def validate_address(address: str) -> tuple:
    """Validate Zcash address format"""
    if not address:
        return False, "Address is required"
    
    # Transparent: t1 or t3 (mainnet), tm (testnet/regtest)
    # Shielded Sapling: zs1
    # Unified: u1
    
    if address.startswith('t'):
        if not re.match(r'^t[13m][a-zA-Z0-9]{33}$', address):
            return False, "Invalid transparent address format"
    elif address.startswith('zs1'):
        if len(address) < 78:
            return False, "Invalid sapling address format"
    elif address.startswith('u1'):
        if len(address) < 100:
            return False, "Invalid unified address format"
    else:
        return False, "Unsupported address type"
    
    return True, ""


@faucet_bp.route('/request', methods=['POST'])
def request_funds():
    """
    Request test funds from faucet - REAL BLOCKCHAIN TRANSACTION!
    """
    data = request.get_json()
    if not data:
        return jsonify({"error": "Invalid JSON"}), 400
    
    # Validate address
    to_address = data.get('address')
    is_valid, error_msg = validate_address(to_address)
    if not is_valid:
        return jsonify({"error": error_msg}), 400
    
    # Get amount
    try:
        amount = float(data.get('amount', current_app.config['FAUCET_AMOUNT_DEFAULT']))
        
        min_amount = current_app.config['FAUCET_AMOUNT_MIN']
        max_amount = current_app.config['FAUCET_AMOUNT_MAX']
        
        if amount < min_amount or amount > max_amount:
            return jsonify({
                "error": f"Amount must be between {min_amount} and {max_amount} ZEC"
            }), 400
    
    except (ValueError, TypeError):
        return jsonify({"error": "Invalid amount"}), 400
    
    # Check wallet ready
    wallet = current_app.faucet_wallet
    if not wallet:
        return jsonify({"error": "Faucet wallet not available"}), 503
    
    # Check balance
    balance = wallet.get_balance()
    if balance < amount:
        return jsonify({
            "error": f"Insufficient faucet balance (available: {balance} ZEC)"
        }), 503
    
    # Send REAL transaction
    try:
        result = wallet.send_to_address(
            to_address=to_address,
            amount=amount,
            memo=data.get('memo')
        )
        
        if not result.get("success"):
            return jsonify({
                "error": f"Transaction failed: {result.get('error')}"
            }), 500
        
        new_balance = wallet.get_balance()
        
        return jsonify({
            "success": True,
            "txid": result["txid"],
            "address": to_address,
            "amount": amount,
            "new_balance": float(new_balance),
            "timestamp": result["timestamp"],
            "message": f"Successfully sent {amount} ZEC. Verify TXID: {result['txid']}"
        }), 200
    
    except Exception as e:
        return jsonify({"error": str(e)}), 500


@faucet_bp.route('/address', methods=['GET'])
def get_faucet_address():
    """Get the faucet's receiving address"""
    wallet = current_app.faucet_wallet
    
    if not wallet:
        return jsonify({"error": "Faucet wallet not available"}), 503
    
    return jsonify({
        "address": wallet.get_address("unified"),
        "balance": float(wallet.get_balance())
    }), 200


@faucet_bp.route('/sync', methods=['POST'])
def sync_wallet():
    """Manually trigger wallet sync"""
    wallet = current_app.faucet_wallet
    
    if not wallet:
        return jsonify({"error": "Faucet wallet not available"}), 503
    
    try:
        wallet.sync_wallet()
        return jsonify({
            "success": True,
            "message": "Wallet synced successfully",
            "current_balance": float(wallet.get_balance())
        }), 200
    except Exception as e:
        return jsonify({"error": str(e)}), 500