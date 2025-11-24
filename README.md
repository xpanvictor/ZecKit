# ZecKit

> A Zcash developer toolkit built on Zebra with real blockchain transactions

[![Smoke Test](https://github.com/Supercoolkayy/ZecKit/actions/workflows/smoke-test.yml/badge.svg)](https://github.com/Supercoolkayy/ZecKit/actions/workflows/smoke-test.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)

---

## 🚀 Project Status: Milestone 2 Complete

**Current Milestone:** M2 - Real Blockchain Transactions  
**Completion:** ✅ 95% Complete (known limitations documented)

### What Works Now

- ✅ **One-command devnet:** `zecdev up` starts everything
- ✅ **Real blockchain transactions:** Actual ZEC transfers via ZingoLib
- ✅ **Auto-mining:** 101+ blocks mined automatically (coinbase maturity)
- ✅ **Faucet API:** REST API for test funds
- ✅ **UA fixtures:** ZIP-316 unified addresses generated
- ✅ **Smoke tests:** 4/5 tests passing
- ✅ **CI pipeline:** GitHub Actions with self-hosted runner

### Known Issues

- ⚠️ **Wallet sync error** after volume deletion (workaround documented)
- ⚠️ **Test 5/5 reliability** - works manually, needs automation fix
- ⚠️ **Transparent mining only** - Zebra internal miner limitation

---

## Quick Start

### Prerequisites

- **OS:** Linux (Ubuntu 22.04+), WSL2, or macOS/Windows with Docker Desktop 4.34+
- **Docker:** Engine ≥ 24.x + Compose v2
- **Resources:** 2 CPU cores, 4GB RAM, 5GB disk

### Installation

```bash
# Clone repository
git clone https://github.com/Supercoolkayy/ZecKit.git
cd ZecKit

# Build CLI
cd cli
cargo build --release
cd ..

# Start devnet (takes 10-15 minutes for mining)
./cli/target/release/zecdev up --backend=lwd

# Run tests
./cli/target/release/zecdev test
```

### Verify It's Working

```bash
# Check service status
curl http://127.0.0.1:8080/health

# Get faucet stats
curl http://127.0.0.1:8080/stats

# Get UA fixture
cat fixtures/unified-addresses.json

# Request test funds (real transaction!)
curl -X POST http://127.0.0.1:8080/request \
  -H "Content-Type: application/json" \
  -d '{"address": "u1...", "amount": 10.0}'
```

---

## CLI Usage

### Start Devnet

```bash
# Start with lightwalletd
zecdev up --backend=lwd

# Stop services
zecdev down

# Stop and remove volumes (fresh start)
zecdev down --purge
```

### Run Tests

```bash
zecdev test

# Expected: 4/5 tests passing
# [1/5] Zebra RPC connectivity... ✓ PASS
# [2/5] Faucet health check... ✓ PASS
# [3/5] Faucet stats endpoint... ✓ PASS
# [4/5] Faucet address retrieval... ✓ PASS
# [5/5] Faucet funding request... ✗ FAIL or SKIP (known issue)
```

---

## Faucet API

### Base URL
```
http://127.0.0.1:8080
```

### Endpoints

**Get Statistics**
```bash
curl http://127.0.0.1:8080/stats
```

**Get Address**
```bash
curl http://127.0.0.1:8080/address
```

**Request Funds**
```bash
curl -X POST http://127.0.0.1:8080/request \
  -H "Content-Type: application/json" \
  -d '{"address": "u1abc...", "amount": 10.0}'
```

Response includes real TXID from blockchain:
```json
{
  "txid": "a1b2c3d4e5f6...",
  "status": "sent",
  "amount": 10.0
}
```

---

## Architecture

```
┌─────────────────────────────────────┐
│         Docker Compose              │
│                                     │
│  ┌──────────┐      ┌──────────┐   │
│  │  Zebra   │◄─────┤  Faucet  │   │
│  │ regtest  │      │  Flask   │   │
│  │  :8232   │      │  :8080   │   │
│  └────┬─────┘      └────┬─────┘   │
│       │                 │          │
│       ▼                 ▼          │
│  ┌──────────┐      ┌──────────┐   │
│  │Lightwald │◄─────┤  Zingo   │   │
│  │  :9067   │      │  Wallet  │   │
│  └──────────┘      └──────────┘   │
└─────────────────────────────────────┘
           ▲
           │
      ┌────┴────┐
      │ zecdev  │  (Rust CLI)
      └─────────┘
```

**Components:**
- **Zebra:** Full node with internal miner
- **Lightwalletd:** Light client protocol server
- **Zingo Wallet:** Official Zcash wallet (ZingoLib)
- **Faucet:** Python Flask API for test funds
- **CLI:** Rust tool for orchestration

---

## Known Limitations (M2)

### 1. ⚠️ Transparent Mining Address Only

**Issue:** Zebra's internal miner currently requires transparent addresses for coinbase rewards.

**Technical Details:**
- While Zcash protocol supports shielded coinbase since Heartwood (2020) via [ZIP-213](https://zips.z.cash/zip-0213)
- Zebra's internal miner implementation for Orchard unified addresses is still in development
- See [Zebra #5929](https://github.com/ZcashFoundation/zebra/issues/5929) for tracking

**Current Configuration:**
```toml
# docker/configs/zebra.toml
[mining]
miner_address = "t27eWDgjFYJGVXmzrXeVjnb5J3uXDM9xH9v"  # Transparent address
```

**Impact:** Mining rewards go to transparent address. For testing shielded transactions, funds must be manually moved to shielded pool.

**Planned Fix:** M3 will add automatic shielding workflow or wait for Zebra upstream support.

---

### 2. ⚠️ Wallet Sync Error After Volume Deletion

**Problem:**
```
Error: wallet height is more than 100 blocks ahead of best chain height
```

**Root Cause:** Wallet database persists from previous run with higher block height than fresh blockchain.

**Solution:**
```bash
# Complete reset (removes all volumes)
./target/release/zecdev down
docker volume rm zeckit_zingo-data zeckit_zebra-data zeckit_lightwalletd-data
./target/release/zecdev up --backend=lwd
```

**Planned Fix:** M3 will implement ephemeral wallet volumes using tmpfs to prevent state conflicts.

---

### 3. ⚠️ Test 5/5 Automated Reliability

**Problem:** Faucet funding request test fails in automated suite but manual transactions work.

**Workaround - Manual Testing:**
```bash
# 1. Sync wallet first
echo "sync run" | docker exec -i zeckit-zingo-wallet zingo-cli \
  --data-dir /var/zingo --server http://lightwalletd:9067

# 2. Check balance
curl http://127.0.0.1:8080/stats

# 3. Request funds
curl -X POST http://127.0.0.1:8080/request \
  -H "Content-Type: application/json" \
  -d '{"address": "u1...", "amount": 10.0}'
```

**Cause:** Timing issues between wallet sync and test execution. Balance shows 0.0 initially because wallet needs time to see mining rewards.

**Planned Fix:** M3 will improve test reliability with explicit sync steps and better timing.

---

### 4. ⚠️ Long Startup Time (10-15 minutes)

**Cause:** Mining 101 blocks to reach coinbase maturity (consensus requirement).

**Cannot be optimized:** This is an inherent blockchain requirement. Coinbase outputs must mature 100 blocks before spending.

**Alternative for M3:** Pre-mined blockchain snapshots for faster startup in CI.

---

### 5. ⚠️ Windows/macOS Best-Effort Support

**Primary Platform:** Linux / WSL2

**Desktop Support:** 
- Docker Desktop 4.34+ required for host networking
- PowerShell command syntax differs from bash
- Some curl commands may need adjustment

**Recommendation:** Use Linux or WSL2 for best experience.

---

## Troubleshooting

### Wallet Sync Error

**Problem:**
```
Error: wallet height is more than 100 blocks ahead of best chain height
```

**Solution:**
```bash
./target/release/zecdev down
docker volume rm zeckit_zingo-data zeckit_zebra-data zeckit_lightwalletd-data
./target/release/zecdev up --backend=lwd
```

### Test 5/5 Fails

**Problem:** Faucet funding request test fails

**Workaround:** Test manually:
```bash
# Sync wallet first
echo "sync run" | docker exec -i zeckit-zingo-wallet zingo-cli \
  --data-dir /var/zingo --server http://lightwalletd:9067

# Check balance
curl http://127.0.0.1:8080/stats

# Request funds
curl -X POST http://127.0.0.1:8080/request \
  -H "Content-Type: application/json" \
  -d '{"address": "u1...", "amount": 10.0}'
```

### Port Conflicts

```bash
# Check what's using ports
lsof -i :8232
lsof -i :8080
lsof -i :9067

# Or change ports in docker-compose.yml
```

### Zebra Won't Start

**Check logs:**
```bash
docker logs zeckit-zebra

# Common issues:
# - Port 8232 already in use
# - Insufficient disk space
# - Corrupted state database
```

**Solution:**
```bash
# Remove volumes and restart
docker volume rm zeckit_zebra-data
./target/release/zecdev up --backend=lwd
```

---

## Documentation

- **[Architecture](specs/architecture.md)** - System design
- **[Technical Spec](specs/technical-spec.md)** - Implementation details
- **[Acceptance Tests](specs/acceptance-tests.md)** - Test criteria

---

## Roadmap

### ✅ Milestone 1: Foundation (Complete)
- Docker-based Zebra regtest
- CI/CD pipeline
- Health checks

### ✅ Milestone 2: Real Transactions (95% Complete)
- Rust CLI tool (`zecdev`)
- Real blockchain transactions via ZingoLib
- Faucet API with balance tracking
- UA fixture generation
- Smoke tests (4/5 passing)

### ⏳ Milestone 3: GitHub Action (Next)
- Fix wallet sync issue (ephemeral volumes)
- Improve test reliability (5/5 passing)
- Reusable GitHub Action
- Full E2E golden flows
- Backend parity testing (lightwalletd ↔ Zaino)
- Auto-shielding workflow

---

## Contributing

Contributions welcome! Please:

1. Fork and create feature branch
2. Test locally: `zecdev up && zecdev test`
3. Follow code style (Rust: `cargo fmt`, Python: `black`)
4. Open PR with clear description

---

## FAQ

**Q: Are these real blockchain transactions?**  
A: Yes! M2 uses real on-chain transactions via ZingoLib and Zingo wallet.

**Q: Can I use this in production?**  
A: No. ZecKit is for development/testing only (regtest mode).

**Q: Why does startup take so long?**  
A: Mining 101 blocks for coinbase maturity takes 10-15 minutes. This is unavoidable.

**Q: Why does test 5/5 fail?**  
A: Known issue with test timing. Manual transactions work fine. Fix planned for M3.

**Q: How do I reset everything?**  
A: `./target/release/zecdev down --purge` removes all volumes.

**Q: Why use transparent mining address?**  
A: Zebra's internal miner doesn't yet support Orchard unified addresses. This is an upstream limitation being tracked in [Zebra #5929](https://github.com/ZcashFoundation/zebra/issues/5929).

**Q: Can I mine to a shielded address?**  
A: Not with Zebra's internal miner in M2. The protocol supports it (since Heartwood/NU5), but implementation is pending in Zebra.

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
- Zingo Labs (ZingoLib)
- Zcash community

---

**Last Updated:** November 24, 2025  
**Next:** M3 - GitHub Action & E2E Flows