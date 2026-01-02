#!/bin/bash
# ========================================
# Generate Pre-mined Blockchain Snapshot
# ========================================
# This script creates a blockchain snapshot with 110+ blocks
# for instant coinbase maturity in CI environments.
#
# Usage: ./scripts/generate-snapshot.sh
# Output: fixtures/blockchain-snapshot.tar.gz
# ========================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
SNAPSHOT_FILE="$PROJECT_DIR/fixtures/blockchain-snapshot.tar.gz"
MIN_BLOCKS=110

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Generating Pre-mined Snapshot"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

cd "$PROJECT_DIR"

# Clean up any existing containers
echo "[1/5] Cleaning up existing containers..."
docker compose --profile zaino down -v 2>/dev/null || true

# Start only Zebra
echo "[2/5] Starting Zebra node..."
docker compose up -d zebra

# Wait for Zebra RPC
echo "[3/5] Waiting for Zebra RPC..."
until curl -sf --max-time 5 \
  --data-binary '{"jsonrpc":"2.0","id":"1","method":"getinfo","params":[]}' \
  -H 'content-type: application/json' \
  http://127.0.0.1:8232 > /dev/null 2>&1; do
  echo "  Waiting for Zebra..."
  sleep 5
done
echo "  ✓ Zebra RPC ready"

# Wait for blocks to be mined
echo "[4/5] Mining $MIN_BLOCKS blocks (this takes ~15-20 minutes)..."
while true; do
  BLOCK_HEIGHT=$(curl -sf --max-time 5 \
    --data-binary '{"jsonrpc":"2.0","id":"1","method":"getblockcount","params":[]}' \
    -H 'content-type: application/json' \
    http://127.0.0.1:8232 | jq -r '.result // 0')
  
  if [ "$BLOCK_HEIGHT" -ge "$MIN_BLOCKS" ]; then
    echo "  ✓ Reached $BLOCK_HEIGHT blocks"
    break
  fi
  
  echo "  Current height: $BLOCK_HEIGHT / $MIN_BLOCKS"
  sleep 30
done

# Stop Zebra gracefully
echo "[5/5] Creating snapshot..."
docker compose stop zebra

# Get the volume name
VOLUME_NAME=$(docker volume ls --format '{{.Name}}' | grep zebra-data | head -1)

if [ -z "$VOLUME_NAME" ]; then
  echo "ERROR: Could not find zebra-data volume"
  docker compose down
  exit 1
fi

# Create snapshot directory
mkdir -p "$PROJECT_DIR/fixtures"

# Export volume to tarball
docker run --rm \
  -v "${VOLUME_NAME}:/data:ro" \
  -v "$PROJECT_DIR/fixtures:/backup" \
  alpine:latest \
  tar -czf /backup/blockchain-snapshot.tar.gz -C /data .

# Clean up
docker compose down -v

# Show result
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Snapshot Created!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "  File: $SNAPSHOT_FILE"
echo "  Size: $(ls -lh "$SNAPSHOT_FILE" | awk '{print $5}')"
echo "  Blocks: $BLOCK_HEIGHT"
echo ""
echo "  To use: The action will automatically restore this snapshot"
echo ""
