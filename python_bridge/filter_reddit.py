# python_bridge/filter_reddit.py
# Usage: python python_bridge/filter_reddit.py

import json
import os

# --- CONFIGURATION ---
# Path to the massive file you downloaded (Update this path if needed)
INPUT_FILE = "/Users/soham/arctic_shift/data/r_India_comments.jsonl" 
OUTPUT_FILE = "./python_bridge/filtered_india_context.jsonl"

# Quality Thresholds
MIN_SCORE = 5        # Only keep comments with 5+ upvotes
MIN_LENGTH = 50      # Only keep substantial comments (no "lol" or "this")
MAX_ITEMS = 50000    # Stop after gathering this many high-quality items

def filter_data():
    print(f"🌊 STARTING FILTER: Streaming from {INPUT_FILE}...")
    
    if not os.path.exists(INPUT_FILE):
        print(f"❌ ERROR: File not found at {INPUT_FILE}")
        print("Please ensure the download is finished and the path is correct.")
        return

    count = 0
    with open(OUTPUT_FILE, 'w', encoding='utf-8') as out_f:
        with open(INPUT_FILE, 'r', encoding='utf-8') as in_f:
            for line in in_f:
                try:
                    data = json.loads(line)
                    
                    # 1. Check Score (Upvotes)
                    score = data.get('score', 0)
                    if score < MIN_SCORE: continue

                    # 2. Check Content
                    body = data.get('body', '')
                    if body == "[deleted]" or body == "[removed]": continue
                    if len(body) < MIN_LENGTH: continue

                    # 3. Save Good Data
                    clean_obj = {
                        "text": body,
                        "score": score,
                        "source": "r/India",
                        "date": data.get('created_utc', 0)
                    }
                    out_f.write(json.dumps(clean_obj) + "\n")
                    
                    count += 1
                    if count % 1000 == 0:
                        print(f"   Collected {count} items...", end='\r')
                    
                    if count >= MAX_ITEMS:
                        break
                        
                except Exception as e:
                    continue

    print(f"\n✅ COMPLETE: Saved {count} high-quality contexts to {OUTPUT_FILE}")

if __name__ == "__main__":
    filter_data()