#!/bin/bash
set -e

echo "🔧 Initializing Zingo Wallet..."

# Wait for lightwalletd
echo "⏳ Waiting for lightwalletd..."
until curl -s http://lightwalletd:9067 > /dev/null 2>&1; do
    sleep 2
done
echo "✅ Lightwalletd is ready!"

# Create wallet if doesn't exist
if [ ! -f "/var/zingo/zingo-wallet.dat" ]; then
    echo "📝 Creating new wallet..."
    zingo-cli --data-dir /var/zingo \
              --server http://lightwalletd:9067 \
              --no-sync \
              new
    
    WALLET_ADDRESS=$(zingo-cli --data-dir /var/zingo \
                               --server http://lightwalletd:9067 \
                               --no-sync \
                               addresses | grep -oP '(?<="address": ")[^"]*' | head -1)
    
    echo "✅ Wallet created!"
    echo "📍 Wallet Address: $WALLET_ADDRESS"
    echo "$WALLET_ADDRESS" > /var/zingo/faucet-address.txt
else
    echo "✅ Existing wallet found"
fi

# Sync wallet
echo "🔄 Syncing wallet..."
zingo-cli --data-dir /var/zingo \
          --server http://lightwalletd:9067 \
          sync

echo "✅ Wallet is ready!"
tail -f /dev/null