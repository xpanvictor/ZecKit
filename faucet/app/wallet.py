import subprocess
import json
import os
import time
import re
from datetime import datetime
from pathlib import Path
import pexpect

class ZingoWallet:
    def __init__(self, data_dir=None, lightwalletd_uri=None):
        self.data_dir = data_dir or os.getenv('WALLET_DATA_DIR', '/var/zingo')
        self.lightwalletd_uri = lightwalletd_uri or os.getenv('LIGHTWALLETD_URI', 'http://zaino:9067')
        self.history_file = Path(self.data_dir) / "faucet-history.json"
        
        print(f"🔧 ZingoWallet initialized:")
        print(f"  Data dir: {self.data_dir}")
        print(f"  Backend URI: {self.lightwalletd_uri}")
        
    def _run_zingo_cmd(self, command, timeout=30, nosync=False):
        """Run zingo-cli command via docker exec"""
        try:
            wallet_container = os.getenv('WALLET_CONTAINER', 'zeckit-zingo-wallet')
            
            sync_flag = "--nosync" if nosync else ""
            cmd_str = f'echo -e "{command}\\nquit" | zingo-cli --data-dir {self.data_dir} --server {self.lightwalletd_uri} --chain regtest {sync_flag}'
            
            cmd = ["docker", "exec", wallet_container, "bash", "-c", cmd_str]
            
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout)
            
            if result.returncode != 0:
                raise Exception(f"Command failed: {result.stderr}")
            
            output = result.stdout.strip()
            
            # Try to parse JSON lines
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
    
    def get_balance(self):
        """Get total wallet balance in ZEC"""
        try:
            result = self._run_zingo_cmd("balance", nosync=False)
            
            total_zatoshis = 0
            
            if isinstance(result, dict) and 'output' in result:
                output = result['output']
                print(f"📊 Balance output (first 500 chars): {output[:500]}")
                
                patterns = [
                    ('transparent', r'confirmed_transparent_balance:\s*([\d_]+)'),
                    ('sapling', r'confirmed_sapling_balance:\s*([\d_]+)'),
                    ('orchard', r'confirmed_orchard_balance:\s*([\d_]+)')
                ]
                
                for pool_name, pattern in patterns:
                    match = re.search(pattern, output)
                    if match:
                        balance_str = match.group(1).replace('_', '')
                        zatoshis = int(balance_str)
                        total_zatoshis += zatoshis
                        print(f"  💰 {pool_name}: {zatoshis} zatoshi ({zatoshis / 100_000_000} ZEC)")
                    else:
                        print(f"  ⚠️ {pool_name}: pattern not found")
            else:
                print(f"❌ Unexpected result format: {type(result)}")
            
            zec_balance = total_zatoshis / 100_000_000
            print(f"📊 Total balance: {zec_balance} ZEC")
            return zec_balance
            
        except Exception as e:
            print(f"❌ Error getting balance: {e}")
            import traceback
            traceback.print_exc()
            return 0.0
    
    def get_orchard_balance(self):
        """Get Orchard balance specifically in ZEC"""
        try:
            result = self._run_zingo_cmd("balance", nosync=False)
            
            if isinstance(result, dict) and 'output' in result:
                output = result['output']
                match = re.search(r'confirmed_orchard_balance:\s*([\d_]+)', output)
                if match:
                    zatoshis = int(match.group(1).replace('_', ''))
                    return zatoshis / 100_000_000
            
            return 0.0
            
        except Exception as e:
            print(f"❌ Error getting Orchard balance: {e}")
            return 0.0
    
    def get_transparent_balance(self):
        """Get Transparent balance specifically in ZEC"""
        try:
            result = self._run_zingo_cmd("balance", nosync=False)
            
            if isinstance(result, dict) and 'output' in result:
                output = result['output']
                match = re.search(r'confirmed_transparent_balance:\s*([\d_]+)', output)
                if match:
                    zatoshis = int(match.group(1).replace('_', ''))
                    return zatoshis / 100_000_000
            
            return 0.0
            
        except Exception as e:
            print(f"❌ Error getting Transparent balance: {e}")
            return 0.0
    
    def get_address(self, address_type="unified"):
        """Get wallet address"""
        try:
            result = self._run_zingo_cmd("addresses", nosync=True)
            
            if isinstance(result, dict) and 'output' in result:
                output = result['output']
                match = re.search(r'uregtest1[a-z0-9]{70,}', output)
                if match:
                    return match.group(0)
            
            return None
            
        except Exception as e:
            print(f"❌ Error getting address: {e}")
            return None
    
    def send_to_address(self, to_address: str, amount: float, memo: str = None):
        """Send using pexpect for proper interactive handling.
        Auto-shields transparent funds to Orchard if needed."""
        try:
            amount_sats = int(amount * 100_000_000)
            wallet_container = os.getenv('WALLET_CONTAINER', 'zeckit-zingo-wallet')
            
            print(f"📤 Sending {amount} ZEC ({amount_sats} sats) to {to_address[:16]}...")
            
            # Spawn interactive zingo-cli session using pexpect
            print("🔄 Starting interactive zingo-cli session...")
            
            cmd = f"docker exec -i {wallet_container} zingo-cli --data-dir {self.data_dir} --server {self.lightwalletd_uri} --chain regtest"
                     
            child = pexpect.spawn(cmd, encoding='utf-8', timeout=120)
            child.logfile_read = open('/tmp/zingo-cli.log', 'w')  # Debug log
            
            # Wait for prompt - handle DEBUG spam with flexible matching and longer timeout
            print("⏳ Waiting for CLI to start (may take 60-90s with DEBUG output)...")
            
            # Use a more flexible regex that just looks for the prompt pattern
            # and give it plenty of time to get through DEBUG spam
            child.expect(r'\(test\) Block:\d+', timeout=90)
            print("✅ CLI ready!")
            
            # Consume rest of prompt line if needed
            try:
                child.expect(r'>>', timeout=2)
            except:
                pass
            
            # Run sync and wait for completion
            print("🔄 Running sync...")
            child.sendline('sync run')
            
            # Wait for "Sync completed successfully" or error
            index = child.expect([
                r'Sync completed succesfully',
                r'sync is already running',
                r'error',
                pexpect.TIMEOUT
            ], timeout=60)
            
            if index == 0:
                print("✅ Sync completed")
            elif index == 1:
                print("⏳ Sync already running, waiting...")
                time.sleep(5)
            elif index == 2:
                print("⚠️  Sync error, continuing anyway")
            else:
                print("⚠️  Sync timeout, continuing anyway")
            
            # Wait for prompt again
            child.expect(r'\(test\) Block:\d+', timeout=15)
            
            # Check balances - both transparent and Orchard
            print("💰 Checking balances...")
            child.sendline('balance')
            child.expect(r'\(test\) Block:\d+', timeout=15)
            
            balance_output = child.before
            print(f"Balance output: {balance_output[:500]}")
            
            # Parse transparent balance
            transparent_match = re.search(r'confirmed_transparent_balance:\s*(\d+)', balance_output)
            transparent_sats = int(transparent_match.group(1)) if transparent_match else 0
            
            # Parse Orchard balance  
            orchard_match = re.search(r'confirmed_orchard_balance:\s*(\d+)', balance_output)
            orchard_sats = int(orchard_match.group(1)) if orchard_match else 0
            
            print(f"💰 Transparent: {transparent_sats / 100_000_000} ZEC, Orchard: {orchard_sats / 100_000_000} ZEC")
            
            required_sats = amount_sats + 20000  # Amount + fee buffer
            
            # If Orchard has insufficient funds but transparent does, suggest shielding
            if orchard_sats < required_sats:
                if transparent_sats >= required_sats:
                    child.sendline('quit')
                    child.close()
                    raise Exception(f"Insufficient Orchard balance ({orchard_sats / 100_000_000} ZEC). "
                                  f"Transparent has {transparent_sats / 100_000_000} ZEC - please shield funds first with 'shield' command")
                else:
                    child.sendline('quit')
                    child.close()
                    raise Exception(f"Insufficient balance: need {required_sats / 100_000_000} ZEC, "
                                  f"have {orchard_sats / 100_000_000} ZEC Orchard + {transparent_sats / 100_000_000} ZEC transparent")
            
            # Now check spendable Orchard balance
            print("💰 Checking spendable Orchard balance...")
            child.sendline('spendable_balance')
            
            try:
                child.expect(r'"spendable_balance":\s*(\d+)', timeout=15)
                spendable_sats = int(child.match.group(1))
            except:
                spendable_sats = 0
                
            print(f"💰 Spendable Orchard: {spendable_sats / 100_000_000} ZEC")
            
            # Check if sufficient
            if spendable_sats < required_sats:
                child.sendline('quit')
                child.close()
                raise Exception(f"Insufficient spendable Orchard balance: need {required_sats / 100_000_000} ZEC, have {spendable_sats / 100_000_000} ZEC. Transparent: {transparent_sats / 100_000_000} ZEC")
            
            print(f"✅ Sufficient funds")
            
            # Wait for prompt
            child.expect(r'\(test\) Block:\d+', timeout=15)
            
            # Send transaction
            print(f"💸 Sending transaction...")
            
            if memo and not (to_address.startswith('tm') or to_address.startswith('t1') or to_address.startswith('t3')):
                child.sendline(f'send {to_address} {amount_sats} "{memo}"')
            else:
                child.sendline(f'send {to_address} {amount_sats}')
            
            # Wait for send response
            child.expect(r'\(test\) Block:\d+', timeout=20)
            
            # Confirm transaction
            print("✅ Confirming transaction...")
            child.sendline('confirm')
            
            # Wait for TXID
            index = child.expect([
                r'"txids":\s*\[\s*"([0-9a-f]{64})"',
                r'error',
                pexpect.TIMEOUT
            ], timeout=45)
            
            if index == 0:
                txid = child.match.group(1)
                print(f"✅ Success! TXID: {txid}")
                
                # Quit cleanly
                child.sendline('quit')
                child.close()
                
                # Record transaction
                timestamp = datetime.utcnow().isoformat() + "Z"
                self._record_transaction(to_address, amount, txid, memo)
                
                return {
                    "success": True,
                    "txid": txid,
                    "timestamp": timestamp
                }
            elif index == 1:
                error_output = child.before + child.after
                child.sendline('quit')
                child.close()
                raise Exception(f"Transaction error: {error_output[:500]}")
            else:
                child.sendline('quit')
                child.close()
                raise Exception("Transaction timeout - no TXID received")
                
        except pexpect.TIMEOUT as e:
            print(f"❌ Timeout: {e}")
            if 'child' in locals():
                child.close()
            return {"success": False, "error": f"CLI timeout: {str(e)}"}
        except pexpect.EOF as e:
            print(f"❌ CLI closed unexpectedly: {e}")
            return {"success": False, "error": "CLI closed unexpectedly"}
        except Exception as e:
            print(f"❌ Send failed: {e}")
            if 'child' in locals():
                try:
                    child.sendline('quit')
                    child.close()
                except:
                    pass
            return {"success": False, "error": str(e)}
                    
    def _record_transaction(self, to_address, amount, txid, memo=""):
        """Record transaction to history"""
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
            print(f"⚠️ Failed to record transaction: {e}")
    
    def get_transaction_history(self, limit=100):
        """Get transaction history"""
        try:
            if not self.history_file.exists():
                return []
            
            history = json.loads(self.history_file.read_text())
            return history[-limit:]
        except Exception as e:
            print(f"❌ Error reading history: {e}")
            return []
    
    def get_stats(self):
        """Get wallet statistics with pool breakdown"""
        try:
            # Get all balances in one call to avoid timing issues
            transparent_balance = 0.0
            orchard_balance = 0.0
            sapling_balance = 0.0
            
            result = self._run_zingo_cmd("balance", nosync=False)
            
            if isinstance(result, dict) and 'output' in result:
                output = result['output']
                
                # Parse transparent
                match = re.search(r'confirmed_transparent_balance:\s*([\d_]+)', output)
                if match:
                    transparent_balance = int(match.group(1).replace('_', '')) / 100_000_000
                
                # Parse orchard
                match = re.search(r'confirmed_orchard_balance:\s*([\d_]+)', output)
                if match:
                    orchard_balance = int(match.group(1).replace('_', '')) / 100_000_000
                    
                # Parse sapling
                match = re.search(r'confirmed_sapling_balance:\s*([\d_]+)', output)
                if match:
                    sapling_balance = int(match.group(1).replace('_', '')) / 100_000_000
            
            balance_total = transparent_balance + orchard_balance + sapling_balance
            address = self.get_address()
            history = self.get_transaction_history(limit=10)
            
            return {
                "balance": balance_total,
                "orchard_balance": orchard_balance,
                "transparent_balance": transparent_balance,
                "sapling_balance": sapling_balance,
                "address": address,
                "transactions_count": len(history),
                "recent_transactions": history[-5:] if history else []
            }
        except Exception as e:
            print(f"❌ Error getting stats: {e}")
            return {
                "balance": 0.0,
                "orchard_balance": 0.0,
                "transparent_balance": 0.0,
                "sapling_balance": 0.0,
                "address": None,
                "transactions_count": 0,
                "recent_transactions": []
            }

# Singleton
_wallet = None

def get_wallet():
    global _wallet
    if _wallet is None:
        _wallet = ZingoWallet()
    return _wallet