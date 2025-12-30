# ZecKit Sample Repository

> Example repository demonstrating ZecKit GitHub Action integration

[![ZecKit E2E Tests](https://github.com/YOUR_USERNAME/zeckit-sample/actions/workflows/zcash-e2e.yml/badge.svg)](https://github.com/YOUR_USERNAME/zeckit-sample/actions/workflows/zcash-e2e.yml)

---

## Overview

This repository demonstrates how to integrate ZecKit into your CI pipeline for automated Zcash testing with real blockchain transactions.

## Quick Start

1. Fork this repository
2. Enable GitHub Actions
3. Push a commit to trigger the workflow

## What Gets Tested

The CI pipeline runs:

1. **Backend Matrix** - Tests against both light client backends:
   - ✅ `lightwalletd` - Production-ready, required to pass
   - 🧪 `zaino` - Experimental Rust indexer

2. **Golden E2E Flow** - Complete shielded transaction lifecycle:
   - Generate Unified Address (ZIP-316)
   - Fund from faucet (transparent)
   - Autoshield to Orchard
   - Shielded send (Orchard → Orchard)
   - Rescan/sync wallet
   - Verify final balances

## Workflow Configuration

See [`.github/workflows/zcash-e2e.yml`](.github/workflows/zcash-e2e.yml) for the full configuration.

### Minimal Example (5 Lines)

```yaml
steps:
  - uses: actions/checkout@v4
  - uses: xpanvictor/ZecKit@main
    with:
      backend: zaino
      run-e2e: 'true'
```

### Backend Matrix Example

```yaml
strategy:
  matrix:
    backend: [lwd, zaino]
steps:
  - uses: xpanvictor/ZecKit@main
    with:
      backend: ${{ matrix.backend }}
```

## Action Inputs

| Input | Default | Description |
|-------|---------|-------------|
| `backend` | `zaino` | Light client backend: `zaino` or `lwd` |
| `startup-timeout` | `30` | Maximum startup time in minutes |
| `min-blocks` | `101` | Blocks to mine (101 for coinbase maturity) |
| `run-e2e` | `false` | Run golden E2E flow tests |

## Action Outputs

| Output | Description |
|--------|-------------|
| `zebra-rpc` | Zebra RPC endpoint (http://127.0.0.1:8232) |
| `faucet-api` | Faucet API endpoint (http://127.0.0.1:8080) |
| `faucet-address` | Pre-funded wallet address |
| `block-height` | Current block height after startup |
| `e2e-result` | E2E test result: `pass`, `fail`, or `skipped` |

## Artifacts

On failure, the workflow uploads:

- `zeckit-logs-{run_number}/` containing:
  - `zebra.log` - Zebra node logs
  - `zaino.log` or `lightwalletd.log` - Backend logs
  - `zingo-wallet.log` - Wallet logs
  - `faucet.log` - Faucet service logs
  - `containers.log` - Docker container status
  - `e2e-output.log` - E2E test output (if run)

## Troubleshooting

### Common Issues

| Issue | Cause | Solution |
|-------|-------|----------|
| "Zebra not ready" timeout | Slow block mining | Increase `startup-timeout` |
| "No balance" in E2E | Coinbase not mature | Ensure `min-blocks: 101` |
| "Connection refused" | Service not started | Check container logs in artifacts |
| "Shield failed" | Wallet sync issue | Check wallet logs, try rescan |

### Debugging Locally

```bash
# Clone ZecKit
git clone https://github.com/xpanvictor/ZecKit.git
cd ZecKit

# Build CLI
cd cli && cargo build --release && cd ..

# Start devnet
./cli/target/release/zecdev up --backend zaino

# Run E2E tests
./cli/target/release/zecdev e2e

# Check logs
docker compose logs zebra
docker compose logs faucet-zaino
```

## License

MIT / Apache-2.0
