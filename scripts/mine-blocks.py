#!/usr/bin/env python3
"""
Mine blocks on Zcash regtest using Zebra's generate RPC method.
This is the correct method per Zebra RPC documentation.
"""
import requests
import sys
import time


def get_block_count():
    """Get current block count"""
    try:
        response = requests.post(
            "http://127.0.0.1:8232",
            json={
                "jsonrpc": "2.0",
                "id": "getcount",
                "method": "getblockcount",
                "params": []
            },
            auth=("zcashrpc", "notsecure"),
            timeout=5
        )
        
        if response.status_code == 200:
            result = response.json()
            return result.get("result", 0)
        return 0
    except Exception as e:
        print(f"❌ Error getting block count: {e}")
        return 0


def mine_blocks(count=101):
    """
    Mine blocks using Zebra's generate RPC method.
    
    Args:
        count: Number of blocks to mine (default: 101)
    
    Returns:
        bool: True if successful, False otherwise
    """
    print(f"🔨 Mining {count} blocks on regtest...")
    
    # Get starting height
    start_height = get_block_count()
    print(f"📊 Starting at block height: {start_height}")
    
    try:
        # Use Zebra's generate method (not getblocktemplate!)
        response = requests.post(
            "http://127.0.0.1:8232",
            json={
                "jsonrpc": "2.0",
                "id": "mine",
                "method": "generate",
                "params": [count]
            },
            auth=("zcashrpc", "notsecure"),
            timeout=30
        )
        
        if response.status_code != 200:
            print(f"❌ HTTP Error: {response.status_code}")
            print(f"Response: {response.text}")
            return False
        
        result = response.json()
        
        # Check for RPC errors
        if "error" in result and result["error"] is not None:
            print(f"❌ RPC Error: {result['error']}")
            return False
        
        # Get final height to verify
        final_height = get_block_count()
        blocks_mined = final_height - start_height
        
        print(f"✅ Successfully mined {blocks_mined} blocks")
        print(f"📊 New block height: {final_height}")
        
        if blocks_mined != count:
            print(f"⚠️  Warning: Requested {count} blocks but mined {blocks_mined}")
        
        return True
        
    except requests.exceptions.Timeout:
        print("❌ Request timeout - Zebra node not responding")
        return False
    except Exception as e:
        print(f"❌ Mining failed: {e}")
        return False


if __name__ == "__main__":
    # Parse command line argument or use default
    if len(sys.argv) > 1:
        try:
            count = int(sys.argv[1])
        except ValueError:
            print("❌ Error: Argument must be an integer")
            sys.exit(1)
    else:
        count = 101  # Default for coinbase maturity
    
    # Mine blocks
    success = mine_blocks(count)
    sys.exit(0 if success else 1)