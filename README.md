# ZecKit

> A Linux-first toolkit for Zcash development on Zebra

[![Smoke Test](https://github.com/Supercoolkayy/ZecKit/actions/workflows/smoke-test.yml/badge.svg)](https://github.com/Supercoolkayy/ZecKit/actions/workflows/smoke-test.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)

---

## 🚀 Project Status: Milestone 1 - Foundation Phase

**Current Milestone:** M1 - Repository Setup & Zebra Devnet  
**Completion:** In Progress

### What Works Now (M1)
- ✅ Zebra regtest node in Docker
- ✅ Health check automation
- ✅ Basic smoke tests
- ✅ CI pipeline (self-hosted runner)
- ✅ Project structure and documentation

### Coming in Future Milestones
- ⏳ M2: CLI tool (`zecdev up/test/down`) + Python faucet
- ⏳ M3: GitHub Action + End-to-end shielded flows
- ⏳ M4: Comprehensive documentation + Quickstarts
- ⏳ M5: 90-day maintenance window

---

## 📋 Table of Contents

- [Overview](#overview)
- [Quick Start](#quick-start)
- [Project Goals](#project-goals)
- [Architecture](#architecture)
- [Development](#development)
- [CI/CD](#cicd)
- [Contributing](#contributing)
- [License](#license)

---

## Overview

**ZecKit** is a developer-first toolkit that provides a fast, reliable, and unified environment for building on Zebra, the new Zcash node implementation replacing zcashd.

In Milestone 1, we establish the foundation: a containerized Zcash regtest devnet with health monitoring and CI integration.

### Key Features (M1)

- **One-Command Startup:** `docker compose up -d` brings up Zebra regtest
- **Health Monitoring:** Automated checks ensure services are ready
- **Smoke Tests:** Verify basic RPC functionality
- **CI Integration:** GitHub Actions on self-hosted runner
- **Linux-First:** Optimized for Linux/WSL environments

---

## Quick Start

### Prerequisites

- **OS:** Linux (Ubuntu 22.04+), WSL, or macOS (best-effort)
- **Docker:** Engine ≥ 24.x + Compose v2
- **Resources:** 2 CPU cores, 4GB RAM, 5GB disk

### Installation

```bash
# Clone the repository
git clone https://github.com/Supercoolkayy/ZecKit.git
cd ZecKit

# Run setup (checks dependencies, pulls images)
chmod +x scripts/setup-dev.sh
./scripts/setup-dev.sh

# Start the devnet
docker compose up -d

# Wait for Zebra to be ready (max 2 minutes)
./docker/healthchecks/check-zebra.sh

# Run smoke tests
./tests/smoke/basic-health.sh
```

### Verify It's Working

```bash
# Check container status
docker compose ps

# Test RPC manually
./scripts/test-zebra-rpc.sh

# View logs
docker compose logs -f zebra
```

### Shutdown

```bash
# Stop services
docker compose down

# Remove volumes (fresh start next time)
docker compose down -v
```

---

## Project Goals

ZecKit aims to solve the critical gap left by zcashd deprecation:

1. **Standardize Zebra Development:** One consistent way to run Zebra + light-client backends locally and in CI
2. **Enable UA-Centric Testing:** Built-in support for Unified Address (ZIP-316) workflows
3. **Support Backend Parity:** Toggle between lightwalletd and Zaino without changing tests
4. **Catch Breakage Early:** Automated E2E tests in CI before code reaches users

### Why This Matters

- Zcash is migrating from zcashd to Zebra (official deprecation in 2025)
- Teams lack a standard, maintained devnet + CI setup
- Fragmented tooling leads to drift, flakiness, and late-discovered bugs
- ZecKit productizes the exact workflow builders need

---

## Architecture

See [specs/architecture.md](specs/architecture.md) for detailed system design.

### High-Level (M1)

```
┌─────────────────────────────────────────┐
│           Docker Compose                 │
│                                          │
│  ┌─────────────┐                        │
│  │   Zebra     │                        │
│  │  (regtest)  │  ← Health Checks       │
│  │   :8232     │                        │
│  └─────────────┘                        │
│                                          │
│  Ports (localhost only):                │
│  - 8232: RPC                             │
│  - 8233: P2P                             │
└─────────────────────────────────────────┘
```

### Components

- **Zebra Node:** Core Zcash regtest node with RPC enabled
- **Health Checks:** Automated validation of service readiness
- **Smoke Tests:** Basic RPC functionality verification
- **CI Pipeline:** GitHub Actions on self-hosted runner

---

## Development

### Repository Structure

```
ZecKit/
├── docker/
│   ├── compose/          # Service definitions
│   ├── configs/          # Zebra configuration
│   └── healthchecks/     # Health check scripts
├── specs/                # Technical specs & architecture
├── tests/
│   └── smoke/            # Smoke test suite
├── scripts/              # Helper scripts
├── faucet/               # Placeholder for M2
└── .github/workflows/    # CI configuration
```

### Common Tasks

```bash
# Start devnet
docker compose up -d

# Check health
./docker/healthchecks/check-zebra.sh

# Run tests
./tests/smoke/basic-health.sh

# View logs
docker compose logs -f

# Stop devnet
docker compose down -v

# Rebuild after changes
docker compose up -d --force-recreate
```

### Manual RPC Testing

```bash
# Use helper script
./scripts/test-zebra-rpc.sh

# Or manually
curl -d '{"method":"getinfo","params":[]}' \
  http://127.0.0.1:8232
```

---

## CI/CD

### GitHub Actions Setup

ZecKit uses a **self-hosted runner** (recommended on WSL/Linux) for CI.

#### Setup Runner

```bash
# Run the setup script
./scripts/setup-wsl-runner.sh

# Follow the prompts to:
# 1. Get runner token from GitHub
# 2. Download and configure runner
# 3. Install as service (optional)
```

#### Manual Setup

1. Go to: **Settings → Actions → Runners** in your GitHub repo
2. Click: **New self-hosted runner**
3. Select: **Linux**
4. Follow instructions to download and configure

### CI Workflow

The smoke test workflow runs automatically on:
- Push to `main` branch
- Pull requests to `main`
- Manual dispatch

See [.github/workflows/smoke-test.yml](.github/workflows/smoke-test.yml)

---

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Quick Guidelines

- **Branch:** Create feature branches from `main`
- **Commits:** Use clear, descriptive messages
- **Tests:** Ensure smoke tests pass before submitting PR
- **Style:** Follow existing code style
- **Documentation:** Update docs for new features

### Development Workflow

1. Fork and clone the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make changes and test locally
4. Run smoke tests: `./tests/smoke/basic-health.sh`
5. Commit and push: `git push origin feature/my-feature`
6. Open a Pull Request

---

## Documentation

- [Architecture](specs/architecture.md) - System design and components
- [Technical Spec](specs/technical-spec.md) - Implementation details
- [Acceptance Tests](specs/acceptance-tests.md) - Test criteria
- [CONTRIBUTING.md](CONTRIBUTING.md) - Contribution guidelines
- [SECURITY.md](SECURITY.md) - Security policy

---

## Roadmap

### Milestone 1: Foundation (Current) ✅
- Repository structure
- Zebra regtest in Docker
- Health checks & smoke tests
- CI pipeline

### Milestone 2: CLI Tool (Next)
- `zecdev` command-line tool
- Python Flask faucet
- Backend toggle (lwd/Zaino prep)
- Pre-mined test funds

### Milestone 3: GitHub Action
- Reusable Action for repos
- End-to-end shielded flows
- UA (ZIP-316) test vectors
- Backend parity testing

### Milestone 4: Documentation
- Quickstart guides
- Video tutorials
- Troubleshooting docs
- Compatibility matrix

### Milestone 5: Maintenance
- 90-day support window
- Version pin updates
- Bug fixes & improvements
- Community handover plan

---

## License

Dual-licensed under your choice of:

- MIT License ([LICENSE-MIT](LICENSE-MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

---

## Acknowledgments

Built by **Dapps over Apps** team

Special thanks to:
- Zcash Foundation (Zebra development)
- Electric Coin Company (Zcash protocol)
- Zingo Labs (Zaino indexer)

---

## Support

- **Issues:** [GitHub Issues](https://github.com/Supercoolkayy/ZecKit/issues)
- **Discussions:** [GitHub Discussions](https://github.com/Supercoolkayy/ZecKit/discussions)
- **Community:** [Zcash Community Forum](https://forum.zcashcommunity.com/)

---

**Status:** 🚧 Milestone 1 - Active Development  
**Last Updated:** November 10, 2025