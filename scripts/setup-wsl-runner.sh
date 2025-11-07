#!/bin/bash
# Setup GitHub Actions self-hosted runner on WSL
# This script guides you through setting up a runner on your laptop

set -e

# Colors
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

log_step() {
    echo -e "${YELLOW}[STEP]${NC} $1"
}

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  GitHub Actions Self-Hosted Runner Setup (WSL)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

log_info "This script will guide you through setting up a GitHub Actions runner on WSL"
echo ""

# Check if on WSL
if ! grep -qi microsoft /proc/version; then
    log_info "Note: This script is optimized for WSL, but can work on Linux too"
fi

# Step 1: Prerequisites
log_step "Step 1: Prerequisites Check"
log_info "Checking Docker..."
if ! command -v docker &> /dev/null; then
    echo "❌ Docker not found. Please install Docker first:"
    echo "   https://docs.docker.com/desktop/wsl/"
    exit 1
fi
log_success "Docker is installed"

log_info "Checking Docker Compose..."
if ! docker compose version &> /dev/null; then
    echo "❌ Docker Compose v2 not found. Please install it first."
    exit 1
fi
log_success "Docker Compose is installed"
echo ""

# Step 2: Create runner directory
log_step "Step 2: Create Runner Directory"
RUNNER_DIR="$HOME/actions-runner"
log_info "Runner will be installed in: $RUNNER_DIR"

if [ -d "$RUNNER_DIR" ]; then
    log_info "Directory already exists. Skipping creation."
else
    mkdir -p "$RUNNER_DIR"
    log_success "Created directory: $RUNNER_DIR"
fi
echo ""

# Step 3: Get runner token
log_step "Step 3: Get GitHub Runner Token"
echo ""
echo "You need to add a self-hosted runner to your GitHub repository:"
echo ""
echo "1. Go to your repository on GitHub"
echo "2. Click: Settings → Actions → Runners"
echo "3. Click: 'New self-hosted runner'"
echo "4. Select: Linux"
echo "5. Copy the runner registration token"
echo ""
read -p "Have you copied the registration token? (y/n): " confirm

if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
    echo "Please get the token from GitHub and run this script again"
    exit 0
fi
echo ""

# Step 4: Download runner
log_step "Step 4: Download GitHub Actions Runner"
cd "$RUNNER_DIR"

RUNNER_VERSION="2.311.0"  # Update this to latest version
RUNNER_URL="https://github.com/actions/runner/releases/download/v${RUNNER_VERSION}/actions-runner-linux-x64-${RUNNER_VERSION}.tar.gz"

log_info "Downloading runner v${RUNNER_VERSION}..."
if curl -o actions-runner-linux-x64.tar.gz -L "$RUNNER_URL"; then
    log_success "Downloaded successfully"
else
    echo "❌ Download failed. Check your internet connection."
    exit 1
fi

log_info "Extracting runner..."
tar xzf ./actions-runner-linux-x64.tar.gz
log_success "Runner extracted"
echo ""

# Step 5: Configure runner
log_step "Step 5: Configure Runner"
echo ""
echo "Now paste your registration token when prompted by the config script:"
echo ""

./config.sh --url https://github.com/YOUR_ORG/YOUR_REPO

log_success "Runner configured!"
echo ""

# Step 6: Install as service
log_step "Step 6: Install Runner as Service (Optional)"
echo ""
echo "Do you want to install the runner as a service?"
echo "(This makes it start automatically)"
echo ""
read -p "Install as service? (y/n): " install_service

if [[ "$install_service" =~ ^[Yy]$ ]]; then
    sudo ./svc.sh install
    sudo ./svc.sh start
    log_success "Runner installed and started as service"
else
    log_info "You can start the runner manually with: ./run.sh"
fi
echo ""

# Step 7: Verify installation
log_step "Step 7: Verification"
echo ""
echo "To verify your runner is working:"
echo "  1. Go to: Settings → Actions → Runners on GitHub"
echo "  2. You should see your runner listed as 'Idle'"
echo ""
log_success "Setup complete!"
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Runner Details:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Location: $RUNNER_DIR"
echo "  Start:    cd $RUNNER_DIR && ./run.sh"
if [[ "$install_service" =~ ^[Yy]$ ]]; then
    echo "  Status:   sudo ./svc.sh status"
    echo "  Stop:     sudo ./svc.sh stop"
fi
echo ""
echo "Next: Push to your repo to trigger the smoke-test workflow!"
echo ""