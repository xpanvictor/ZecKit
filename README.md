# ZecKit

> A Linux-first toolkit for Zcash development on Zebra with real blockchain transactions

[![Smoke Test](https://github.com/Supercoolkayy/ZecKit/actions/workflows/smoke-test.yml/badge.svg)](https://github.com/Supercoolkayy/ZecKit/actions/workflows/smoke-test.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)

---

## Project Status

**Current Milestone:** M3 Complete - GitHub Action + Golden E2E  

### What's Delivered

**✅ M1 - Foundation**
- Zebra regtest node in Docker
- Health check automation  
- Basic smoke tests
- CI pipeline (self-hosted runner)
- Project structure and documentation

**✅ M2 - Real Transactions**
- `zecdev` CLI tool with automated setup
- Real blockchain transactions via ZingoLib
- Faucet API with actual on-chain broadcasting
- Backend toggle (lightwalletd ↔ Zaino)
- Automated mining address configuration
- UA (ZIP-316) address generation
- Comprehensive test suite (M1 + M2)

**✅ M3 - GitHub Action**
- Reusable GitHub Action for any repository
- Golden E2E shielded flows (generate UA → fund → shield → send → verify)
- Pre-built Docker images on GHCR
- Backend matrix testing (lwd + zaino)

---

## Quick Start

### Prerequisites

- **OS:** Linux (Ubuntu 22.04+), WSL2, or macOS with Docker Desktop 4.34+
- **Docker:** Engine ≥ 24.x + Compose v2
- **Resources:** 2 CPU cores, 4GB RAM, 5GB disk

### Installation

```bash
# Clone repository
git clone https://github.com/Supercoolkayy/ZecKit.git
cd ZecKit

# Build CLI (one time)
cd cli
cargo build --release
cd ..

# Start devnet with automatic setup
./cli/target/release/zecdev up --backend zaino
#  First run takes 10-15 minutes (mining 101+ blocks)
# ✓ Automatically extracts wallet address
# ✓ Configures Zebra mining address
# ✓ Waits for coinbase maturity

# Run test suite
./cli/target/release/zecdev test

# Verify faucet has funds
curl http://localhost:8080/stats
```

### Alternative: Manual Setup (M1 Style)

```bash
# For users who prefer manual Docker Compose control

# 1. Setup mining address
./scripts/setup-mining-address.sh zaino

# 2. Start services manually
docker-compose --profile zaino up -d

# 3. Wait for 101 blocks (manual monitoring)
curl -s http://localhost:8232 -X POST \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"1.0","id":"1","method":"getblockcount","params":[]}' | jq .result

# 4. Run tests
./cli/target/release/zecdev test
```

### Verify It's Working

```bash
# M1 tests - Basic health
curl http://localhost:8232  # Zebra RPC
curl http://localhost:8080/health  # Faucet health

# M2 tests - Real transactions
curl http://localhost:8080/stats  # Should show balance
curl -X POST http://localhost:8080/request \
  -H "Content-Type: application/json" \
  -d '{"address": "tmXXXXX...", "amount": 10.0}'  # Real TXID returned!
```

---

## GitHub Action (5-Line CI)

Add ZecKit to your repository's CI in 5 lines:

```yaml
# .github/workflows/zcash-tests.yml
name: Zcash E2E
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: xpanvictor/ZecKit@main
        with:
          backend: zaino
          run-e2e: 'true'
```

### Action Inputs

| Input | Default | Description |
|-------|---------|-------------|
| `backend` | `zaino` | Light client backend: `zaino` or `lwd` |
| `startup-timeout` | `30` | Max startup time in minutes |
| `min-blocks` | `101` | Blocks to mine (101 for coinbase maturity) |
| `run-e2e` | `false` | Run golden E2E flow tests |

### Action Outputs

| Output | Description |
|--------|-------------|
| `zebra-rpc` | Zebra RPC endpoint (http://127.0.0.1:8232) |
| `faucet-api` | Faucet API endpoint (http://127.0.0.1:8080) |
| `faucet-address` | Pre-funded wallet address |
| `block-height` | Current block height |
| `e2e-result` | E2E test result (pass/fail/skipped) |

### Backend Matrix Example

Test against both backends:

```yaml
jobs:
  e2e-test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        backend: [zaino, lwd]
    steps:
      - uses: actions/checkout@v4
      - uses: xpanvictor/ZecKit@main
        id: devnet
        with:
          backend: ${{ matrix.backend }}
          run-e2e: 'true'
      - name: Run your tests
        run: |
          curl ${{ steps.devnet.outputs.faucet-api }}/stats
```

### Golden E2E Flow

The E2E tests exercise the complete shielded transaction lifecycle:

1. **Generate UA** - Create Unified Address (ZIP-316)
2. **Fund** - Request ZEC from faucet (transparent)
3. **Autoshield** - Shield transparent funds to Orchard
4. **Shielded Send** - Send Orchard → Orchard
5. **Rescan/Sync** - Wallet rescan
6. **Verify** - Check final balances

Run locally:
```bash
./cli/target/release/zecdev e2e
```

---

## CLI Usage

### zecdev Commands (M2)

**Start Devnet (Automated):**
```bash
# Build CLI first (one time)
cd cli && cargo build --release && cd ..

# Start with Zaino backend (recommended - faster)
./cli/target/release/zecdev up --backend zaino

# OR start with Lightwalletd backend
./cli/target/release/zecdev up --backend lwd
```

**What happens automatically:**
1. ✓ Starts Zebra regtest + backend + wallet + faucet
2. ✓ Waits for wallet initialization
3. ✓ Extracts wallet's transparent address
4. ✓ Updates `zebra.toml` with correct miner_address
5. ✓ Restarts Zebra to apply changes
6. ✓ Mines 101+ blocks for coinbase maturity
7. ✓ **Ready to use!**

**Stop Services:**
```bash
./cli/target/release/zecdev down
```

**Run Test Suite (M1 + M2):**
```bash
./cli/target/release/zecdev test

# Expected output:
# [1/5] Zebra RPC connectivity... ✓ PASS (M1 test)
# [2/5] Faucet health check... ✓ PASS (M1 test)
# [3/5] Faucet stats endpoint... ✓ PASS (M2 test)
# [4/5] Faucet address retrieval... ✓ PASS (M2 test)
# [5/5] Faucet funding request... ✓ PASS (M2 test - real tx!)
```

### Manual Docker Compose (M1 Style)

**For users who want direct control:**

```bash
# Setup mining address first
./scripts/setup-mining-address.sh zaino

# Start with Zaino profile
docker-compose --profile zaino up -d

# OR start with Lightwalletd profile
docker-compose --profile lwd up -d

# Stop services
docker-compose --profile zaino down
# or
docker-compose --profile lwd down
```

### Complete Workflow

```bash
# 1. Build CLI (one time)
cd cli && cargo build --release && cd ..

# 2. Start devnet (automatic setup!)
./cli/target/release/zecdev up --backend zaino
#  Takes 10-15 minutes on first run (mining + sync)

# 3. Run test suite
./cli/target/release/zecdev test

# 4. Check faucet balance
curl http://localhost:8080/stats

# 5. Request funds (real transaction!)
curl -X POST http://localhost:8080/request \
  -H "Content-Type: application/json" \
  -d '{"address": "tmXXXXX...", "amount": 10.0}'

# 6. Stop when done
./cli/target/release/zecdev down
```

### Fresh Start (Reset Everything)

```bash
# Stop services
./cli/target/release/zecdev down

# Remove volumes
docker volume rm zeckit_zebra-data zeckit_zaino-data

# Start fresh (automatic setup again)
./cli/target/release/zecdev up --backend zaino
```

### Switch Backends

```bash
# Stop current backend
./cli/target/release/zecdev down

# Start with different backend
./cli/target/release/zecdev up --backend lwd

# Or back to Zaino
./cli/target/release/zecdev up --backend zaino
```

---

## Test Suite (M1 + M2)

### Automated Tests

```bash
./cli/target/release/zecdev test
```

**Test Breakdown:**

| Test | Milestone | What It Checks |
|------|-----------|----------------|
| 1/5 Zebra RPC | M1 | Basic node connectivity |
| 2/5 Faucet health | M1 | Service health endpoint |
| 3/5 Faucet stats | M2 | Balance tracking API |
| 4/5 Faucet address | M2 | Address retrieval |
| 5/5 Faucet request | M2 | **Real transaction!** |

**Expected Results:**
- M1 tests (1-2): Always pass if services running
- M2 tests (3-4): Pass after wallet sync
- M2 test 5: Pass after 101+ blocks mined (timing dependent)

### Manual Testing (M1 Style)

```bash
# M1 - Test Zebra RPC
curl -d '{"method":"getinfo","params":[]}' http://localhost:8232

# M1 - Check health
curl http://localhost:8080/health

# M2 - Check balance
curl http://localhost:8080/stats

# M2 - Get address
curl http://localhost:8080/address

# M2 - Real transaction test
curl -X POST http://localhost:8080/request \
  -H "Content-Type: application/json" \
  -d '{"address": "tmXXXXX...", "amount": 10.0}'
```

---

## Faucet API (M2)

### Base URL
```
http://localhost:8080
```

### Endpoints

**GET /health (M1)**
```bash
curl http://localhost:8080/health
```
Response:
```json
{
  "status": "healthy"
}
```

**GET /stats (M2)**
```bash
curl http://localhost:8080/stats
```
Response:
```json
{
  "current_balance": 1628.125,
  "transparent_balance": 1628.125,
  "orchard_balance": 0.0,
  "faucet_address": "tmYuH9GAxfWM82Kckyb6kubRdpCKRpcw1ZA",
  "total_requests": 0,
  "uptime": "5m 23s"
}
```

**GET /address (M2)**
```bash
curl http://localhost:8080/address
```
Response:
```json
{
  "address": "tmYuH9GAxfWM82Kckyb6kubRdpCKRpcw1ZA"
}
```

**POST /request (M2 - Real Transaction!)**
```bash
curl -X POST http://localhost:8080/request \
  -H "Content-Type: application/json" \
  -d '{"address": "tmXXXXX...", "amount": 10.0}'
```
Response includes **real TXID** from blockchain:
```json
{
  "success": true,
  "txid": "a1b2c3d4e5f6789...",
  "timestamp": "2025-12-15T12:00:00Z",
  "amount": 10.0
}
```

---

## Architecture

### M1 Architecture (Foundation)
```
┌─────────────────────────────┐
│      Docker Compose         │
│                             │
│  ┌─────────────┐           │
│  │   Zebra     │           │
│  │  (regtest)  │           │
│  │   :8232     │           │
│  └─────────────┘           │
│                             │
│  Health checks + RPC tests  │
└─────────────────────────────┘
```

### M2 Architecture (Real Transactions)
```
┌──────────────────────────────────────────┐
│           Docker Compose                  │
│                                           │
│  ┌──────────┐        ┌──────────┐       │
│  │  Zebra   │◄───────┤  Faucet  │       │
│  │ regtest  │        │  Flask   │       │
│  │  :8232   │        │  :8080   │       │
│  └────┬─────┘        └────┬─────┘       │
│       │                   │              │
│       ▼                   ▼              │
│  ┌──────────┐        ┌──────────┐       │
│  │ Zaino or │◄───────┤  Zingo   │       │
│  │Lightwald │        │  Wallet  │       │
│  │  :9067   │        │(pexpect) │       │
│  └──────────┘        └──────────┘       │
└──────────────────────────────────────────┘
           ▲
           │
      ┌────┴────┐
      │ zecdev  │  (Rust CLI - M2)
      └─────────┘
```

**Components:**
- **Zebra:** Full node with internal miner (M1)
- **Lightwalletd/Zaino:** Light client backends (M2)
- **Zingo Wallet:** Real transaction creation (M2)
- **Faucet:** REST API for test funds (M2)
- **zecdev CLI:** Automated orchestration (M2)

---

## Project Goals

### Why ZecKit?

Zcash is migrating from zcashd to Zebra (official deprecation 2025), but builders lack a standard devnet + CI setup. ZecKit solves this by:

1. **Standardizing Zebra Development** - One consistent way to run Zebra + light-client backends
2. **Enabling UA-Centric Testing** - Built-in ZIP-316 unified address support
3. **Supporting Backend Parity** - Toggle between lightwalletd and Zaino
4. **Catching Breakage Early** - Automated E2E tests in CI

### Progression (M1 → M2 → M3)

**M1 Foundation:**
- Basic Zebra regtest
- Health checks
- Manual Docker Compose

**M2 Real Transactions:**
- Automated CLI (`zecdev`)
- Real on-chain transactions
- Faucet API with pexpect
- Backend toggle

**M3 CI/CD (Next):**
- GitHub Action
- Golden shielded flows
- Pre-mined snapshots

---

## Usage Notes

### First Run Setup

When you run `./cli/target/release/zecdev up` for the first time:

1. **Initial mining takes 10-15 minutes** - This is required for coinbase maturity (Zcash consensus)
2. **Automatic configuration** - The CLI extracts wallet address and configures Zebra automatically
3. **Monitor progress** - Watch the CLI output or check block count:
   ```bash
   curl -s http://localhost:8232 -X POST -H 'Content-Type: application/json' \
     -d '{"jsonrpc":"1.0","id":"1","method":"getblockcount","params":[]}' | jq .result
   ```

### Fresh Restart

To reset everything and start clean:

```bash
# Stop services
./cli/target/release/zecdev down

# Remove volumes (blockchain data)
docker volume rm zeckit_zebra-data zeckit_zaino-data

# Start fresh
./cli/target/release/zecdev up --backend zaino
```

### Switch Backends

```bash
# Stop current backend
./cli/target/release/zecdev down

# Start with different backend
./cli/target/release/zecdev up --backend lwd

# Or back to Zaino
./cli/target/release/zecdev up --backend zaino
```

---

## Troubleshooting

### Common Operations

**Reset blockchain and start fresh:**
```bash
./cli/target/release/zecdev down
docker volume rm zeckit_zebra-data zeckit_zaino-data
./cli/target/release/zecdev up --backend zaino
```

**Check service logs:**
```bash
docker logs zeckit-zebra
docker logs zeckit-faucet
docker logs zeckit-zaino
```

**Check wallet balance manually:**
```bash
docker exec -it zeckit-zingo-wallet zingo-cli \
  --data-dir /var/zingo \
  --server http://zaino:9067 \
  --chain regtest

# At prompt:
balance
addresses
```

**Verify mining progress:**
```bash
# Check block count
curl -s http://localhost:8232 -X POST \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"1.0","id":"1","method":"getblockcount","params":[]}' | jq .result

# Check mempool
curl -s http://localhost:8232 -X POST \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"1.0","id":"1","method":"getrawmempool","params":[]}' | jq
```

**Check port usage:**
```bash
lsof -i :8232  # Zebra
lsof -i :8080  # Faucet
lsof -i :9067  # Backend
```

---

## Documentation

- **[Architecture](specs/architecture.md)** - System design and data flow
- **[Technical Spec](specs/technical-spec.md)** - Implementation details (27 pages!)
- **[Acceptance Tests](specs/acceptance-tests.md)** - Test criteria

---

## Roadmap

###  Milestone 1: Foundation
- Repository structure
- Zebra regtest in Docker
- Health checks & smoke tests
- CI pipeline
- Manual Docker Compose workflow

###  Milestone 2: Real Transactions
- `zecdev` CLI tool with automated setup
- Real blockchain transactions
- Faucet API with balance tracking
- Backend toggle (lightwalletd ↔ Zaino)
- Automated mining address configuration
- UA (ZIP-316) address generation
- Comprehensive test suite

###  Milestone 3: GitHub Action
- Reusable GitHub Action for CI
- Golden E2E shielded flows
- Pre-mined blockchain snapshots
- Backend parity testing
- Auto-shielding workflow

###  Milestone 4: Documentation
- Quickstart guides
- Video tutorials
- Compatibility matrix
- Advanced workflows

###  Milestone 5: Maintenance
- 90-day support window
- Version pin updates
- Community handover

---

## Technical Highlights

### M1 Achievement: Docker Foundation

- Zebra regtest with health checks
- Automated smoke tests
- CI pipeline integration
- Manual service control

### M2 Achievement: Real Transactions

**Pexpect for Wallet Interaction:**
```python
# Reliable PTY control replaces flaky subprocess
child = pexpect.spawn('docker exec -i zeckit-zingo-wallet zingo-cli ...')
child.expect(r'\(test\) Block:\d+', timeout=90)
child.sendline('send [{"address":"tm...", "amount":10.0}]')
child.expect(r'"txid":\s*"([a-f0-9]{64})"')
txid = child.match.group(1)  # Real TXID!
```

**Automated Setup:**
- Wallet address extraction
- Zebra configuration updates
- Service restarts
- Mining to maturity

**Ephemeral Wallet (tmpfs):**
```yaml
zingo-wallet:
  tmpfs:
    - /var/zingo:mode=1777,size=512m
```
Benefits: Fresh state, fast I/O, no corruption

---

## Contributing

Contributions welcome! Please:

1. Fork and create feature branch
2. Test locally: `./cli/target/release/zecdev up --backend zaino && ./cli/target/release/zecdev test`
3. Follow code style (Rust: `cargo fmt`, Python: `black`)
4. Open PR with clear description

---

## FAQ

**Q: What's the difference between M1 and M2?**  
A: M1 = Basic Zebra setup. M2 = Automated CLI + real transactions + faucet API.

**Q: Are these real blockchain transactions?**  
A: Yes! Uses actual ZingoLib wallet with real on-chain transactions (regtest network).

**Q: Can I use this in production?**  
A: No. ZecKit is for development/testing only (regtest mode).

**Q: How do I start the devnet?**  
A: `./cli/target/release/zecdev up --backend zaino` (or `--backend lwd`)

**Q: How long does first startup take?**  
A: 10-15 minutes for mining 101 blocks (coinbase maturity requirement).

**Q: Can I switch between lightwalletd and Zaino?**  
A: Yes! `zecdev down` then `zecdev up --backend [lwd|zaino]`

**Q: How do I reset everything?**  
A: `zecdev down && docker volume rm zeckit_zebra-data zeckit_zaino-data`

**Q: Where can I find the technical details?**  
A: Check [specs/technical-spec.md](specs/technical-spec.md) for the full implementation (27 pages!)

**Q: What tests are included?**  
A: M1 tests (RPC, health) + M2 tests (stats, address, real transactions)

---

## Support

- **Issues:** [GitHub Issues](https://github.com/Supercoolkayy/ZecKit/issues)
- **Discussions:** [GitHub Discussions](https://github.com/Supercoolkayy/ZecKit/discussions)
- **Community:** [Zcash Forum](https://forum.zcashcommunity.com/)

---

## License

Dual-licensed under MIT OR Apache-2.0

---

## Acknowledgments

**Built by:** Dapps over Apps team

**Thanks to:**
- Zcash Foundation (Zebra)
- Electric Coin Company (lightwalletd)
- Zingo Labs (ZingoLib & Zaino)
- Zcash community

---

**Last Updated:** December 16, 2025  
**Status:** M2 Complete - Real Blockchain Transactions Delivered