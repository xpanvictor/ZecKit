#!/usr/bin/env bash
# Development environment setup for ZecKit
# Sets up Docker, dependencies, and validates the environment
set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

COMPOSE_MAIN="docker-compose.yml"
COMPOSE_ZEBRA="docker/compose/zebra.yml"
# network names used in compose files
EXPECTED_NETWORK_NAME="zecdev-network"
FALLBACK_NETWORK_NAME="zecdev"

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[OK]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if running on supported platform
check_platform() {
    log_info "Checking platform..."
    if [[ "${OSTYPE:-}" == "linux-gnu"* ]]; then
        log_success "Running on Linux"
        PLATFORM="linux"
    elif [[ "${OSTYPE:-}" == "darwin"* ]]; then
        log_warn "Running on macOS (best-effort support)"
        PLATFORM="macos"
    elif grep -qi microsoft /proc/version 2>/dev/null; then
        log_success "Running on WSL (Windows Subsystem for Linux)"
        PLATFORM="wsl"
    else
        log_error "Unsupported platform: ${OSTYPE:-unknown}"
        log_error "ZecKit officially supports Linux. macOS/Windows are best-effort."
        exit 1
    fi
}

# Check Docker installation
check_docker() {
    log_info "Checking Docker installation..."
    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed"
        log_info "Please install Docker: https://docs.docker.com/get-docker/"
        exit 1
    fi

    local docker_version
    docker_version=$(docker --version 2>/dev/null || true)
    log_success "Docker found: ${docker_version}"

    if ! docker ps &> /dev/null; then
        log_error "Docker daemon is not running"
        log_info "Please start Docker and try again"
        exit 1
    fi

    log_success "Docker daemon is running"
}

# Check Docker Compose
check_docker_compose() {
    log_info "Checking Docker Compose..."
    if ! docker compose version &> /dev/null; then
        log_error "Docker Compose v2 is not installed or not available via 'docker compose'"
        log_info "Please install Docker Compose v2: https://docs.docker.com/compose/install/"
        exit 1
    fi

    local compose_version
    compose_version=$(docker compose version 2>/dev/null || true)
    log_success "Docker Compose found: ${compose_version}"
}

# Check system resources
check_resources() {
    log_info "Checking system resources..."
    if [[ "${PLATFORM}" == "linux" ]] || [[ "${PLATFORM}" == "wsl" ]]; then
        local total_mem
        total_mem=$(free -g | awk '/^Mem:/{print $2}' || echo 0)
        if [ "${total_mem}" -lt 4 ]; then
            log_warn "System has less than 4GB RAM (${total_mem} GB available)"
            log_warn "Recommended: 4GB+ for smooth operation"
        else
            log_success "Memory: ${total_mem}GB available"
        fi
    fi

    local available_space
    available_space=$(df -BG . | tail -1 | awk '{print $4}' | sed 's/G//' || echo 0)
    if [ "${available_space}" -lt 5 ]; then
        log_warn "Less than 5GB disk space available (${available_space}GB)"
        log_warn "Recommended: 5GB+ for Docker images and blockchain data"
    else
        log_success "Disk space: ${available_space}GB available"
    fi
}

# Check required tools
check_tools() {
    log_info "Checking required tools..."
    if ! command -v curl &> /dev/null; then
        log_error "curl is not installed"
        exit 1
    fi
    log_success "curl found"

    if ! command -v jq &> /dev/null; then
        log_warn "jq is not installed (optional, but recommended for JSON parsing)"
        log_info "Install: sudo apt install jq  (Ubuntu/Debian)"
    else
        log_success "jq found"
    fi
}

# Create necessary directories
setup_directories() {
    log_info "Setting up directories..."
    mkdir -p logs docker/data tests/smoke faucet
    log_success "Directories created"
}

# Make scripts executable
setup_permissions() {
    log_info "Setting script permissions..."
    chmod +x scripts/*.sh 2>/dev/null || true
    chmod +x docker/healthchecks/*.sh 2>/dev/null || true
    chmod +x tests/smoke/*.sh 2>/dev/null || true
    log_success "Script permissions set"
}

# Ensure docker network exists (last-resort fallback)
ensure_network() {
    # If neither expected nor fallback network exists, create fallback network
    if docker network ls --format "{{.Name}}" | grep -qx "${EXPECTED_NETWORK_NAME}"; then
        log_info "Network '${EXPECTED_NETWORK_NAME}' already exists"
        return 0
    fi
    if docker network ls --format "{{.Name}}" | grep -qx "${FALLBACK_NETWORK_NAME}"; then
        log_info "Network '${FALLBACK_NETWORK_NAME}' already exists"
        return 0
    fi

    log_warn "Neither '${EXPECTED_NETWORK_NAME}' nor '${FALLBACK_NETWORK_NAME}' network exists. Creating '${FALLBACK_NETWORK_NAME}' as fallback."
    if docker network create "${FALLBACK_NETWORK_NAME}" >/dev/null 2>&1; then
        log_success "Created network '${FALLBACK_NETWORK_NAME}'"
    else
        log_warn "Failed to create fallback network '${FALLBACK_NETWORK_NAME}'. Continuing — compose may still succeed."
    fi
}

# Decide which compose file set to use and return the args
select_compose_files() {
    # Prefer merged set if zebra compose exists and validates when merged
    if [ -f "${COMPOSE_ZEBRA}" ]; then
        if docker compose -f "${COMPOSE_MAIN}" -f "${COMPOSE_ZEBRA}" config >/dev/null 2>&1; then
            echo "-f ${COMPOSE_MAIN} -f ${COMPOSE_ZEBRA}"
            return 0
        else
            log_warn "Merged compose validation failed for ${COMPOSE_MAIN} + ${COMPOSE_ZEBRA}. Falling back to ${COMPOSE_MAIN} only."
        fi
    fi

    # If zebra file not present or merge failed, fall back to main compose only
    if docker compose -f "${COMPOSE_MAIN}" config >/dev/null 2>&1; then
        echo "-f ${COMPOSE_MAIN}"
        return 0
    fi

    # As a last resort, return empty and let caller handle it
    echo ""
    return 1
}

# Pull Docker images (robust: uses merged files when available)
pull_images() {
    log_info "Pulling Docker images..."
    log_info "This may take a few minutes on first run..."

    local compose_args
    compose_args=$(select_compose_files) || compose_args=""

    if [ -z "${compose_args}" ]; then
        log_warn "Could not validate any compose file. Attempting 'docker compose pull' with default context."
        if docker compose pull; then
            log_success "Docker images pulled successfully (default compose context)"
            return 0
        else
            log_error "Failed to pull Docker images using default compose context"
            ensure_network
            exit 1
        fi
    fi

    # shellcheck disable=SC2086
    if docker compose ${compose_args} pull; then
        log_success "Docker images pulled successfully"
        return 0
    fi

    # If pull failed, attempt to ensure network exists then retry once more
    log_warn "docker compose pull failed. Ensuring expected network exists and retrying once."
    ensure_network

    # retry
    # shellcheck disable=SC2086
    if docker compose ${compose_args} pull; then
        log_success "Docker images pulled successfully on retry"
        return 0
    fi

    log_error "Failed to pull Docker images after retry"
    log_info "Try running: docker compose ${compose_args} config  (to inspect the merged config)"
    exit 1
}

# Main setup
main() {
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  ZecKit - Development Setup"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""

    check_platform
    check_docker
    check_docker_compose
    check_resources
    check_tools
    setup_directories
    setup_permissions
    pull_images

    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo -e "${GREEN}✓ Development environment setup complete!${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "Next steps:"
    echo "  1. Start the devnet:  docker compose up -d"
    echo "  2. Check health:      ./docker/healthchecks/check-zebra.sh"
    echo "  3. Run smoke tests:   ./tests/smoke/basic-health.sh"
    echo ""
    echo "For more information, see README.md"
    echo ""
}

main "$@"
