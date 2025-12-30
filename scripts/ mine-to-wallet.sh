#!/bin/bash
set -e

# Configuration
WALLET_ADDR=$1
BLOCKS=${2:-110}
ZEBRA_RPC="http://127.0.0.1:8232"
RPC_USER="zcashrpc"
RPC_PASS="notsecure"

# Validate inputs
if [ -z "$WALLET_ADDR" ]; then
    echo "❌ Error: Wallet address required"
    echo "Usage: $0 <wallet-address> [num-blocks]"
    exit 1
fi

echo "⛏️  Mining $BLOCKS blocks to $WALLET_ADDR..."
echo "📍 Using Zebra RPC: $ZEBRA_RPC"

# Check if address is valid regtest address
if [[ ! $WALLET_ADDR =~ ^(tm|uregtest) ]]; then
    echo "⚠️  Warning: Address doesn't look like a regtest address"
    echo "   Expected prefix: tm... or uregtest..."
fi

# Mine blocks using Zebra's generate method
# Note: Zebra's generate mines to the internal wallet, not to a specific address
# For mining to a specific address, you need to configure Zebra's mining settings

echo "🔨 Starting mining..."

# Use Zebra's generate RPC (not generatetoaddress - that doesn't exist!)
curl -s -u "$RPC_USER:$RPC_PASS" \
    -d "{\"jsonrpc\":\"2.0\",\"id\":\"mine\",\"method\":\"generate\",\"params\":[$BLOCKS]}" \
    -H 'content-type: application/json' \
    "$ZEBRA_RPC" > /tmp/mine-result.json

# Check if mining succeeded
if grep -q '"result"' /tmp/mine-result.json; then
    BLOCK_HASHES=$(jq -r '.result | length' /tmp/mine-result.json 2>/dev/null || echo "0")
    echo "✅ Mining complete! Mined $BLOCK_HASHES blocks"
    
    # Get current block height
    BLOCK_HEIGHT=$(curl -s -u "$RPC_USER:$RPC_PASS" \
        -d '{"jsonrpc":"2.0","id":"count","method":"getblockcount","params":[]}' \
        -H 'content-type: application/json' \
        "$ZEBRA_RPC" | jq -r '.result' 2>/dev/null || echo "unknown")
    
    echo "📊 Current block height: $BLOCK_HEIGHT"
else
    echo "❌ Mining failed:"
    cat /tmp/mine-result.json
    exit 1
fi

# Note about mining address
echo ""
echo "⚠️  Note: Zebra mines blocks internally. To receive rewards at $WALLET_ADDR:"
echo "   1. Configure mining.miner_address in zebra.toml"
echo "   2. Or transfer funds from mined coinbase transactions"

rm -f /tmp/mine-result.json