"""
ZecKit Faucet Wallet - Real Blockchain Transactions via Zingo-CLI
"""
import os
import json
import subprocess
from datetime import datetime
from typing import Dict, Optional, List
from decimal import Decimal


class ZingoWallet:
    """
    Real Zcash wallet using Zingo-CLI for actual blockchain transactions.
    """
    
    def __init__(self):
        self.data_dir = os.getenv("ZINGO_DATA_DIR", "/var/zingo")
        self.cli_path = os.getenv("ZINGO_CLI_PATH", "/usr/local/bin/zingo-cli")
        self.server = os.getenv("LIGHTWALLETD_URI", "http://lightwalletd:9067")
        
        # Transaction history
        self.history_file = os.path.join(self.data_dir, "faucet-history.json")
        self.load_history()
    
    def load_history(self):
        """Load transaction history from disk"""
        if os.path.exists(self.history_file):
            with open(self.history_file, 'r') as f:
                self.history = json.load(f)
        else:
            self.history = {
                "total_sent": 0.0,
                "total_requests": 0,
                "transactions": []
            }
    
    def save_history(self):
        """Save transaction history to disk"""
        with open(self.history_file, 'w') as f:
            json.dump(self.history, f, indent=2)
    
    def _run_zingo_cmd(self, command: str, *args) -> Dict:
        """
        Execute a zingo-cli command and return parsed JSON output.
        """
        cmd = [
            self.cli_path,
            "--data-dir", self.data_dir,
            "--server", self.server,
            command
        ]
        cmd.extend(args)
        
        try:
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                check=True,
                timeout=30
            )
            
            return json.loads(result.stdout)
            
        except subprocess.CalledProcessError as e:
            print(f"❌ Zingo-CLI Error: {e.stderr}")
            raise Exception(f"Zingo-CLI command failed: {e.stderr}")
        except json.JSONDecodeError as e:
            print(f"❌ Failed to parse Zingo-CLI output: {result.stdout}")
            raise Exception(f"Invalid JSON from Zingo-CLI: {str(e)}")
        except Exception as e:
            print(f"❌ Unexpected error: {str(e)}")
            raise
    
    def get_balance(self) -> Decimal:
        """Get current wallet balance from Zingo-CLI"""
        try:
            balance_data = self._run_zingo_cmd("balance")
            
            # Sum all pool balances
            total_zatoshis = 0
            
            if "transparent_balance" in balance_data:
                total_zatoshis += balance_data["transparent_balance"]
            if "sapling_balance" in balance_data:
                total_zatoshis += balance_data["sapling_balance"]
            if "orchard_balance" in balance_data:
                total_zatoshis += balance_data["orchard_balance"]
            
            # Convert zatoshis to ZEC
            return Decimal(total_zatoshis) / Decimal(100_000_000)
            
        except Exception as e:
            print(f"❌ Failed to get balance: {str(e)}")
            return Decimal(0)
    
    def get_address(self, address_type: str = "unified") -> str:
        """Get a wallet address"""
        try:
            addresses = self._run_zingo_cmd("addresses")
            
            if isinstance(addresses, list) and len(addresses) > 0:
                for addr in addresses:
                    if address_type == "unified" and addr.get("address", "").startswith("u"):
                        return addr["address"]
                    elif address_type == "sapling" and addr.get("address", "").startswith("z"):
                        return addr["address"]
                    elif address_type == "transparent" and addr.get("address", "").startswith("t"):
                        return addr["address"]
                
                return addresses[0].get("address", "")
            
            raise Exception("No addresses found in wallet")
            
        except Exception as e:
            print(f"❌ Failed to get address: {str(e)}")
            raise
    
    def send_to_address(
        self,
        to_address: str,
        amount: float,
        memo: Optional[str] = None
    ) -> Dict:
        """
        Send ZEC to an address - REAL BLOCKCHAIN TRANSACTION!
        """
        try:
            # Convert ZEC to zatoshis
            zatoshis = int(amount * 100_000_000)
            
            # Prepare send command
            send_args = [to_address, str(zatoshis)]
            if memo:
                send_args.append(memo)
            
            print(f"📤 Sending {amount} ZEC to {to_address}...")
            tx_result = self._run_zingo_cmd("send", *send_args)
            
            txid = tx_result.get("txid")
            
            if not txid:
                raise Exception("No transaction ID returned from Zingo-CLI")
            
            # Record transaction in history
            tx_record = {
                "txid": txid,
                "to_address": to_address,
                "amount": amount,
                "timestamp": datetime.utcnow().isoformat() + "Z",
                "memo": memo
            }
            
            self.history["transactions"].append(tx_record)
            self.history["total_sent"] += amount
            self.history["total_requests"] += 1
            self.save_history()
            
            print(f"✅ Transaction sent! TXID: {txid}")
            
            return {
                "success": True,
                "txid": txid,
                "amount": amount,
                "to_address": to_address,
                "timestamp": tx_record["timestamp"]
            }
            
        except Exception as e:
            print(f"❌ Failed to send transaction: {str(e)}")
            return {
                "success": False,
                "error": str(e)
            }
    
    def get_transaction_history(self) -> List[Dict]:
        """Get transaction history"""
        return self.history.get("transactions", [])
    
    def get_stats(self) -> Dict:
        """Get wallet statistics"""
        return {
            "total_sent": self.history.get("total_sent", 0.0),
            "total_requests": self.history.get("total_requests", 0),
            "current_balance": float(self.get_balance()),
            "faucet_address": self.get_address("unified")
        }
    
    def sync_wallet(self):
        """Sync wallet with blockchain"""
        try:
            print("🔄 Syncing wallet with blockchain...")
            self._run_zingo_cmd("sync")
            print("✅ Wallet synced!")
        except Exception as e:
            print(f"❌ Sync failed: {str(e)}")


# Singleton instance
_wallet_instance = None


def get_wallet() -> ZingoWallet:
    """Get or create the wallet singleton"""
    global _wallet_instance
    if _wallet_instance is None:
        _wallet_instance = ZingoWallet()
    return _wallet_instance