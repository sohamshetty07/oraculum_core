import requests
import json
import urllib.parse
import time

# --- CONFIGURATION ---
USER_AGENT = 'OraculumMarketBot/1.0 (Student Project)'

def fetch_reddit_voices(query):
    """
    Channel 1: The 'Voice of the People'
    Fetches raw discussions from Reddit. Zero HTML parsing.
    """
    print(f"\n📢 CHANNEL 1: Reddit Voices (Query: '{query}')...")
    url = f"https://www.reddit.com/search.json?q={urllib.parse.quote(query)}&sort=relevance&limit=5"
    headers = {'User-Agent': USER_AGENT}
    
    try:
        resp = requests.get(url, headers=headers, timeout=5)
        if resp.status_code == 200:
            data = resp.json()
            posts = data.get('data', {}).get('children', [])
            
            if posts:
                print(f"   ✅ SUCCESS: Found {len(posts)} threads.")
                voices = []
                for p in posts[:3]:
                    title = p['data']['title']
                    score = p['data']['score']
                    print(f"      - \"{title}\" (Score: {score})")
                    voices.append(title)
                return voices
            else:
                print("   ⚠️  Reddit silent on this exact topic.")
                return []
        else:
            print(f"   ❌ Reddit API Status: {resp.status_code}")
            return []
    except Exception as e:
        print(f"   ❌ Connection Error: {e}")
        return []

def fetch_wikipedia_context(query):
    """
    Channel 2: The 'Brand Authority'
    Fetches the corporate identity and history.
    """
    # Extract the Brand Name (heuristic: first word or two)
    brand_guess = query.split()[0]
    print(f"\n🏛️ CHANNEL 2: Wikipedia Authority (Query: '{brand_guess}')...")
    
    url = "https://en.wikipedia.org/w/api.php"
    params = {
        "action": "query",
        "format": "json",
        "prop": "extracts",
        "exintro": True,
        "explaintext": True,
        "titles": brand_guess
    }
    
    try:
        resp = requests.get(url, params=params, headers={'User-Agent': USER_AGENT}, timeout=5)
        data = resp.json()
        pages = data.get("query", {}).get("pages", {})
        
        for page_id, page_data in pages.items():
            if page_id != "-1":
                extract = page_data.get("extract", "")
                snippet = extract[:200].replace('\n', ' ')
                print(f"   ✅ SUCCESS: Verified Entity.")
                print(f"      - {snippet}...")
                return extract
            
        print("   ⚠️  Entity not found on Wikipedia.")
        return ""
    except Exception as e:
        print(f"   ❌ Wikipedia Error: {e}")
        return ""

def fetch_product_specs(query):
    """
    Channel 3: The 'Hard Facts' (Open Database)
    Tries OpenFoodFacts first. Best for FMCG/Beverages.
    """
    print(f"\n📦 CHANNEL 3: Product Specifications (Query: '{query}')...")
    
    # 1. Try OpenFoodFacts (World's largest open food DB)
    off_url = f"https://world.openfoodfacts.org/cgi/search.pl?search_terms={urllib.parse.quote(query)}&search_simple=1&action=process&json=1"
    
    try:
        resp = requests.get(off_url, headers={'User-Agent': USER_AGENT}, timeout=8)
        data = resp.json()
        products = data.get('products', [])
        
        if products:
            p = products[0] # Top match
            print(f"   ✅ SUCCESS: Found Data in OpenFoodFacts!")
            
            # Extract key marketing data
            specs = {
                "Product Name": p.get('product_name', 'Unknown'),
                "Brand": p.get('brands', 'Unknown'),
                "NutriScore": p.get('nutriscore_grade', 'N/A').upper(),
                "Quantity": p.get('quantity', 'Unknown'),
                "Ingredients Count": len(p.get('ingredients_tags', [])),
                "Labels": p.get('labels', 'None')
            }
            
            for k, v in specs.items():
                print(f"      - {k}: {v}")
            
            return json.dumps(specs)
            
        else:
            print("   ⚠️  Not found in Food DB. (If this was Tech, we'd query a different DB).")
            return ""
            
    except Exception as e:
        print(f"   ❌ Specs Fetch Error: {e}")
        return ""

if __name__ == "__main__":
    # Test with your specific product
    target = "Amul Protein Lassi"
    
    print(f"--- STARTING FEDERATED SCOUT FOR: {target} ---")
    
    voices = fetch_reddit_voices(target)
    context = fetch_wikipedia_context(target)
    facts = fetch_product_specs(target)
    
    print("\n--- FEDERATION REPORT ---")
    if voices: print(f"✅ Voices: Acquired ({len(voices)} threads)")
    if context: print(f"✅ Context: Acquired ({len(context)} chars)")
    if facts: print(f"✅ Facts: Acquired (Structured JSON)")
    if not (voices or context or facts): print("❌ SYSTEM FAILURE: All channels silent.")