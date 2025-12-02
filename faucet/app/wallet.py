import subprocess
import json
import os
import time
from datetime import datetime
from pathlib import Path

class ZingoWallet:
    def __init__(self, data_dir=None, lightwalletd_uri=None):
        # Get from environment with fallback
        self.data_dir = data_dir or os.getenv('WALLET_DATA_DIR', '/var/zingo')
        self.lightwalletd_uri = lightwalletd_uri or os.getenv('LIGHTWALLETD_URI', 'http://lightwalletd:9067')
        self.history_file = Path(self.data_dir) / "faucet-history.json"
        
        print(f"🔧 ZingoWallet initialized:")
        print(f"  Data dir: {self.data_dir}")
        print(f"  Backend URI: {self.lightwalletd_uri}")
        
    def _run_zingo_cmd(self, command, timeout=30):
        """Run zingo-cli command via docker exec to zingo-wallet container"""
        try:
            # Get wallet container name from environment
            wallet_container = os.getenv('WALLET_CONTAINER', 'zeckit-zingo-wallet')
            
            cmd = [
                "docker", "exec", wallet_container,
                "zingo-cli",
                "--data-dir", self.data_dir,
                "--server", self.lightwalletd_uri,
                "--nosync"
            ]
            
            result = subprocess.run(
                cmd,
                input=f"{command}\nquit\n",
                capture_output=True,
                text=True,
                timeout=timeout
            )
            
            if result.returncode != 0:
                raise Exception(f"Command failed: {result.stderr}")
            
            output = result.stdout.strip()
            
            for line in output.split('\n'):
                line = line.strip()
                if line.startswith('{') or line.startswith('['):
                    try:
                        return json.loads(line)
                    except:
                        continue
            
            return {"output": output}
            
        except subprocess.TimeoutExpired:
            raise Exception("Command timed out")
        except Exception as e:
            raise Exception(f"Failed to run command: {str(e)}")
    
    def sync_wallet(self, retries=3):
        """Sync wallet with blockchain - CRITICAL before sending funds"""
        wallet_container = os.getenv('WALLET_CONTAINER', 'zeckit-zingo-wallet')
        
        for attempt in range(retries):
            try:
                print(f"🔄 Syncing wallet (attempt {attempt + 1}/{retries})...")
                
                cmd = [
                    "docker", "exec", "-i", wallet_container,
                    "zingo-cli",
                    "--data-dir", self.data_dir,
                    "--server", self.lightwalletd_uri
                ]
                
                result = subprocess.run(
                    cmd,
                    input="sync\nquit\n",
                    capture_output=True,
                    text=True,
                    timeout=60
                )
                
                # Check for sync errors
                if "wallet height is more than 100 blocks ahead" in result.stdout:
                    print(f"⚠️ Wallet sync timing issue - waiting for blocks...")
                    if attempt < retries - 1:
                        time.sleep(15)
                        continue
                    else:
                        print(f"✅ Continuing despite sync warning (attempt {attempt + 1})")
                        return True
                
                print(f"✅ Wallet sync completed (attempt {attempt + 1})")
                time.sleep(5)
                return True
                
            except Exception as e:
                print(f"⚠️ Sync attempt {attempt + 1} failed: {e}")
                if attempt < retries - 1:
                    time.sleep(10)
                    continue
                else:
                    print(f"❌ Sync failed after {retries} attempts")
                    return False
        
        return False
    
    def get_balance(self):
        """Get wallet balance in ZEC"""
        try:
            result = self._run_zingo_cmd("balance")
            
            total_zatoshis = 0
            if isinstance(result, dict):
                total_zatoshis += result.get('transparent_balance', 0)
                total_zatoshis += result.get('sapling_balance', 0)
                total_zatoshis += result.get('orchard_balance', 0)
                
                if total_zatoshis == 0 and 'balance' in result:
                    total_zatoshis = result.get('balance', 0)
            
            return total_zatoshis / 100_000_000
            
        except Exception as e:
            print(f"Error getting balance: {e}")
            return 0.0
    
    def get_address(self, address_type="unified"):
        """Get wallet address"""
        try:
            result = self._run_zingo_cmd("addresses")
            
            if isinstance(result, list):
                for addr in result:
                    if isinstance(addr, dict) and addr.get('address', '').startswith('u1'):
                        return addr.get('address')
            
            address_file = Path(self.data_dir) / "faucet-address.txt"
            if address_file.exists():
                return address_file.read_text().strip()
            
            return None
            
        except Exception as e:
            print(f"Error getting address: {e}")
            return None
    
    def send_to_address(self, to_address, amount, memo=""):
        """
        Send ZEC to address - FIXED with proper sync and balance check
        Returns transaction details with TXID
        """
        try:
            print(f"💰 Preparing to send {amount} ZEC to {to_address}")
            
            # STEP 1: Sync wallet BEFORE attempting send
            print("STEP 1/4: Syncing wallet...")
            if not self.sync_wallet(retries=3):
                print("⚠️ Wallet sync had issues but continuing...")
            
            # STEP 2: Verify balance
            print("STEP 2/4: Checking balance...")
            balance = self.get_balance()
            print(f"  Current balance: {balance} ZEC")
            
            if balance == 0:
                raise Exception("Wallet has zero balance. Wait for mining rewards.")
            
            if balance < amount:
                raise Exception(f"Insufficient balance. Have {balance} ZEC, need {amount} ZEC")
            
            # STEP 3: Send transaction
            print("STEP 3/4: Sending transaction...")
            zatoshis = int(amount * 100_000_000)
            
            if memo:
                command = f'send {to_address} {zatoshis} "{memo}"'
            else:
                command = f'send {to_address} {zatoshis}'
            
            result = self._run_zingo_cmd(command, timeout=60)
            
            if isinstance(result, dict):
                txid = result.get('txid')
                if txid:
                    print(f"✅ Transaction successful: {txid}")
                    
                    # STEP 4: Record transaction
                    print("STEP 4/4: Recording transaction...")
                    timestamp = datetime.utcnow().isoformat() + "Z"
                    self._record_transaction(to_address, amount, txid, memo)
                    
                    # Final sync
                    self.sync_wallet(retries=1)
                    
                    return {
                        "success": True,
                        "txid": txid,
                        "timestamp": timestamp
                    }
            
            raise Exception("No TXID returned from send command")
            
        except Exception as e:
            return {
                "success": False,
                "error": str(e)
            }
    
    def _record_transaction(self, to_address, amount, txid, memo=""):
        """Record transaction to history file"""
        try:
            history = []
            if self.history_file.exists():
                history = json.loads(self.history_file.read_text())
            
            history.append({
                "timestamp": datetime.utcnow().isoformat() + "Z",
                "to_address": to_address,
                "amount": amount,
                "txid": txid,
                "memo": memo
            })
            
            self.history_file.write_text(json.dumps(history, indent=2))
            
        except Exception as e:
            print(f"Warning: Failed to record transaction: {e}")
    
    def get_transaction_history(self, limit=100):
        """Get transaction history"""
        try:
            if not self.history_file.exists():
                return []
            
            history = json.loads(self.history_file.read_text())
            return history[-limit:]
            
        except Exception as e:
            print(f"Error reading history: {e}")
            return []
    
    def get_stats(self):
        """Get wallet statistics"""
        try:
            balance = self.get_balance()
            address = self.get_address()
            history = self.get_transaction_history(limit=10)
            
            return {
                "balance": balance,
                "address": address,
                "transactions_count": len(history),
                "recent_transactions": history[-5:] if history else []
            }
        except Exception as e:
            print(f"Error getting stats: {e}")
            return {
                "balance": 0.0,
                "address": None,
                "transactions_count": 0,
                "recent_transactions": []
            }

# Singleton wallet instance
_wallet = None

def get_wallet():
    """Get wallet singleton - reads from environment"""
    global _wallet
    if _wallet is None:
        _wallet = ZingoWallet()
    return _wallet