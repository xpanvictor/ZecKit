#!/bin/bash
# Development environment setup for ZecDev Launchpad
# Sets up Docker, dependencies, and validates the environment

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

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
    
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        log_success "Running on Linux"
        PLATFORM="linux"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        log_warn "Running on macOS (best-effort support)"
        PLATFORM="macos"
    elif grep -qi microsoft /proc/version 2>/dev/null; then
        log_success "Running on WSL (Windows Subsystem for Linux)"
        PLATFORM="wsl"
    else
        log_error "Unsupported platform: $OSTYPE"
        log_error "ZecDev Launchpad officially supports Linux. macOS/Windows are best-effort."
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
    
    local docker_version=$(docker --version)
    log_success "Docker found: $docker_version"
    
    # Check if Docker daemon is running
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
        log_error "Docker Compose v2 is not installed"
        log_info "Please install Docker Compose: https://docs.docker.com/compose/install/"
        exit 1
    fi
    
    local compose_version=$(docker compose version)
    log_success "Docker Compose found: $compose_version"
}

# Check system resources
check_resources() {
    log_info "Checking system resources..."
    
    # Check available memory (Linux/WSL)
    if [[ "$PLATFORM" == "linux" ]] || [[ "$PLATFORM" == "wsl" ]]; then
        local total_mem=$(free -g | awk '/^Mem:/{print $2}')
        if [ "$total_mem" -lt 4 ]; then
            log_warn "System has less than 4GB RAM ($total_mem GB available)"
            log_warn "Recommended: 4GB+ for smooth operation"
        else
            log_success "Memory: ${total_mem}GB available"
        fi
    fi
    
    # Check available disk space
    local available_space=$(df -BG . | tail -1 | awk '{print $4}' | sed 's/G//')
    if [ "$available_space" -lt 5 ]; then
        log_warn "Less than 5GB disk space available"
        log_warn "Recommended: 5GB+ for Docker images and blockchain data"
    else
        log_success "Disk space: ${available_space}GB available"
    fi
}

# Check required tools
check_tools() {
    log_info "Checking required tools..."
    
    # curl
    if ! command -v curl &> /dev/null; then
        log_error "curl is not installed"
        exit 1
    fi
    log_success "curl found"
    
    # jq (optional but recommended)
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
    
    mkdir -p logs
    mkdir -p docker/data
    mkdir -p tests/smoke
    mkdir -p faucet
    
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

# Pull Docker images
pull_images() {
    log_info "Pulling Docker images..."
    log_info "This may take a few minutes on first run..."
    
    if docker compose pull; then
        log_success "Docker images pulled successfully"
    else
        log_error "Failed to pull Docker images"
        exit 1
    fi
}

# Main setup
main() {
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  ZecDev Launchpad - Development Setup"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
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
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
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