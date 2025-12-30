#!/bin/bash
# ========================================
# ZecKit Linux Test Environment
# ========================================
# Spins up an Ubuntu container with Docker to test ZecKit
# as it would run on a Linux CI runner.
#
# Usage: ./scripts/linux-test-env.sh
# ========================================

set -e

CONTAINER_NAME="zeckit-linux-test"
IMAGE="ubuntu:22.04"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  ZecKit Linux Test Environment"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Check if container already exists
if docker ps -a --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
    echo "Container ${CONTAINER_NAME} already exists."
    read -p "Remove and recreate? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        docker rm -f ${CONTAINER_NAME}
    else
        echo "Attaching to existing container..."
        docker start ${CONTAINER_NAME} 2>/dev/null || true
        docker exec -it ${CONTAINER_NAME} /bin/bash
        exit 0
    fi
fi

echo "Starting Linux test environment..."
echo ""

# Run Ubuntu container with:
# - Docker socket mounted (Docker-from-Docker)
# - Project directory mounted
# - Interactive shell
docker run -it \
    --name ${CONTAINER_NAME} \
    --hostname zeckit-linux \
    -v /var/run/docker.sock:/var/run/docker.sock \
    -v "$(pwd)":/workspace \
    -w /workspace \
    --network host \
    ${IMAGE} \
    /bin/bash -c '
        echo "Installing dependencies..."
        apt-get update -qq
        apt-get install -y -qq curl git ca-certificates gnupg lsb-release jq \
            build-essential gcc g++ pkg-config libssl-dev cmake > /dev/null

        # Install Docker CLI
        echo "Installing Docker CLI..."
        mkdir -p /etc/apt/keyrings
        curl -fsSL https://download.docker.com/linux/ubuntu/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg
        echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null
        apt-get update -qq
        apt-get install -y -qq docker-ce-cli docker-compose-plugin > /dev/null

        # Install Rust
        echo "Installing Rust..."
        curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal > /dev/null 2>&1
        source $HOME/.cargo/env

        echo ""
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        echo "  Linux Test Environment Ready!"
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        echo ""
        echo "Commands to try:"
        echo "  cd cli && cargo build --release"
        echo "  ./target/release/zecdev up --backend zaino"
        echo "  ./target/release/zecdev test"
        echo ""
        echo "Or pull pre-built images:"
        echo "  docker compose -f docker-compose.prebuilt.yml --profile zaino up -d"
        echo ""
        
        exec /bin/bash
    '
