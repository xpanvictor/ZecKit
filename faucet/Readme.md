# ZecKit Faucet - Real Blockchain Transactions

Zcash regtest faucet using **ZingoLib** for real blockchain transactions.

## Features

- ✅ **Real blockchain transactions** via Zingo-CLI
- ✅ **Verifiable TXIDs** on-chain
- ✅ **Unified Address support** (ZIP-316)
- ✅ **Shielded transactions** actually work
- ✅ **No mocking** - everything is real!

## Endpoints

- `GET /health` - Health check
- `GET /stats` - Faucet statistics
- `GET /address` - Get faucet address
- `GET /history` - Transaction history
- `POST /request` - Request funds (REAL transaction)
- `POST /sync` - Sync wallet with blockchain

## Example Request
```bash
curl -X POST http://localhost:8080/request \
  -H "Content-Type: application/json" \
  -d '{"address": "u1...", "amount": 10.0}'
```

## Response
```json
{
  "success": true,
  "txid": "abc123...",
  "amount": 10.0,
  "new_balance": 490.0,
  "message": "Successfully sent 10.0 ZEC. Verify TXID: abc123..."
}
```

## Verify Transaction
```bash
# Get transaction details from Zebra
curl -X POST http://localhost:8232 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":"1","method":"getrawtransaction","params":["<TXID>",1]}'
```

This proves it's a **REAL blockchain transaction**! 🎉
```

---

## ✅ **FINAL FILE STRUCTURE**
```
faucet/
├── app/
│   ├── routes/
│   │   ├── __init__.py      # Empty or minimal
│   │   ├── faucet.py        # REAL transaction handling
│   │   ├── health.py        # Health checks
│   │   └── stats.py         # Statistics
│   ├── __init__.py          # Empty or version
│   ├── config.py            # Configuration
│   ├── main.py              # Flask app factory
│   └── wallet.py            # Zingo-CLI wrapper
├── Dockerfile               # Updated for production
├── Readme.md                # Updated docs
└── requirements.txt         # Simplified dependencies