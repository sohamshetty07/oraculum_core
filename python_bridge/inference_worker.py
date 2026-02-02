# python_bridge/inference_worker.py
# SYSTEM V4.1: HYBRID BRAIN (Local RAG + Live Fallback + Context Sharding)
# UPDATED: Fixed Deadlock using readline() instead of iterator

import sys
import json
import base64
import io
import re
import urllib.parse
import hashlib
import networkx as nx 
import lancedb
import requests
import datetime
from pypdf import PdfReader
from sentence_transformers import SentenceTransformer
from mlx_vlm import load, generate
from PIL import Image
from duckduckgo_search import DDGS 

# --- CONFIGURATION ---
MODEL_PATH = "mlx-community/Phi-3.5-vision-instruct-4bit"
EMBEDDING_MODEL = "all-MiniLM-L6-v2" 
DB_PATH = "./knowledge_db" 
USER_AGENT = 'OraculumMarketBot/1.0 (Student Project)'

# --- GLOBAL MEMORY STATE ---
memory_graph = nx.DiGraph()

# --- INITIALIZATION ---
print(json.dumps({"status": "loading_models"}), flush=True)

try:
    model, processor = load(MODEL_PATH, trust_remote_code=True)
    embed_model = SentenceTransformer(EMBEDDING_MODEL)
    
    # Initialize DB and try to load the Indian Context table
    db = lancedb.connect(DB_PATH)
    
    # Handle deprecation warning gracefully while checking for table
    existing_tables = db.table_names()
    if "memory_bank" in existing_tables:
        knowledge_table = db.open_table("memory_bank")
        print(json.dumps({"status": "ready", "memory": "loaded"}), flush=True)
    else:
        knowledge_table = None
        print(json.dumps({"status": "ready", "memory": "empty_waiting_for_input"}), flush=True)

except Exception as e:
    print(json.dumps({"status": "error", "message": f"Startup Error: {str(e)}"}), flush=True)
    sys.exit(1)

# --- HELPER: ROBUST QUERY RELAXATION ---
def clean_query_step(query):
    """Recursively simplifies a query until APIs respond."""
    original = query
    # Remove measurements and marketing fluff
    query = re.sub(r'\b\d+(ml|g|kg|L|oz)\b', '', query, flags=re.IGNORECASE)
    query = re.sub(r'\(.*?\)|pack of \d+|official|review', '', query, flags=re.IGNORECASE)
    query = re.sub(r'\b(for|with|and|flavor|flavour|variant)\b', '', query, flags=re.IGNORECASE)
    query = " ".join(query.split()) # Collapse whitespace
    
    if query == original: # If regex fails, chop last word
        tokens = query.split()
        if len(tokens) > 1: query = " ".join(tokens[:-1])
    return query

# --- TOOL 1: FEDERATED RESEARCH (Legacy API - Voices + Context) ---
def perform_federated_research(topic, audience_context):
    """
    Queries Reddit (Voices) and Wikipedia (Context) via direct API calls.
    Kept for legacy compatibility and backup.
    """
    voices = []
    
    # 1. REDDIT (The Voice)
    current_q = topic
    for _ in range(3): # Retry loop
        try:
            url = f"https://www.reddit.com/search.json?q={urllib.parse.quote(current_q)}&sort=relevance&limit=8"
            resp = requests.get(url, headers={'User-Agent': USER_AGENT}, timeout=5)
            if resp.status_code == 200:
                posts = resp.json().get('data', {}).get('children', [])
                if posts:
                    for p in posts:
                        title = p['data']['title']
                        if len(title) > 15: voices.append(title)
                    break # Found data, stop relaxing
        except: pass
        
        # Relax query for next loop
        new_q = clean_query_step(current_q)
        if new_q == current_q: break
        current_q = new_q

    # 2. WIKIPEDIA (The Context)
    wiki_summary = ""
    brand_guess = topic.split()[0]
    try:
        url = "https://en.wikipedia.org/w/api.php"
        params = {
            "action": "query", "format": "json", "prop": "extracts",
            "exintro": True, "explaintext": True, "titles": brand_guess
        }
        resp = requests.get(url, params=params, headers={'User-Agent': USER_AGENT}, timeout=4)
        data = resp.json().get("query", {}).get("pages", {})
        for _, page in data.items():
            if "extract" in page:
                wiki_summary = page["extract"][:300]
                voices.append(f"[Context] Brand Background: {wiki_summary}...")
                break
    except: pass

    if not voices:
        return ["SYSTEM_ALERT: No digital footprint found. The product might be too new or niche."]
    
    return list(set(voices))[:15]

# --- TOOL 2: FACT CHECK (OpenFoodFacts) ---
def perform_fact_check(query):
    """
    Queries OpenFoodFacts for FMCG specs.
    """
    current_q = query
    for _ in range(3):
        try:
            # OpenFoodFacts API
            url = f"https://world.openfoodfacts.org/cgi/search.pl?search_terms={urllib.parse.quote(current_q)}&search_simple=1&action=process&json=1"
            resp = requests.get(url, headers={'User-Agent': USER_AGENT}, timeout=6)
            products = resp.json().get('products', [])
            
            if products:
                p = products[0]
                specs = (
                    f"Product: {p.get('product_name', 'Unknown')}\n"
                    f"Brand: {p.get('brands', 'Unknown')}\n"
                    f"NutriScore: {p.get('nutriscore_grade', '?').upper()}\n"
                    f"Ingredients: {', '.join(p.get('ingredients_tags', [])[:5])}..."
                )
                return specs
        except: pass
        
        # Relax
        new_q = clean_query_step(current_q)
        if new_q == current_q: break
        current_q = new_q

    return "SYSTEM_ALERT: No structured data found in Open Databases."

# --- HELPER: CONTEXT SHARDING ---
def get_sharded_context(agent_name, topic):
    """
    Prevents 'Echo Chamber' by giving different agents different memories.
    Uses hashing on the agent's name to deterministically assign a shard.
    """
    if not memory_graph.has_node(topic): return ""
    
    all_opinions = []
    for peer in memory_graph.predecessors(topic):
        if peer == agent_name: continue
        edge = memory_graph.get_edge_data(peer, topic)
        all_opinions.append(f"- {peer}: \"{edge.get('content','...')}\"")
    
    if not all_opinions: return ""

    # SHARDING LOGIC
    shard_index = sum(ord(c) for c in agent_name) % 3
    shard_size = max(1, len(all_opinions) // 2)
    start = (shard_index * shard_size) % len(all_opinions)
    selected = all_opinions[start : start + 3]
    
    return "\n[WHAT OTHERS ARE SAYING - YOUR FEED]:\n" + "\n".join(selected) + "\n"

# --- HELPER: PDF PROCESSING ---
def process_pdf(pdf_b64):
    try:
        pdf_data = base64.b64decode(pdf_b64)
        reader = PdfReader(io.BytesIO(pdf_data))
        text = "".join([page.extract_text() for page in reader.pages])
        words = text.split()
        return [" ".join(words[i:i+200]) for i in range(0, len(words), 150)]
    except: return []

# --- MAGMA MEMORY UPDATE ---
def _update_graph_memory(agent_name, response_text, topic):
    try:
        sentiment = 0.0
        lower = response_text.lower()
        if "love" in lower or "great" in lower: sentiment = 1.0
        elif "hate" in lower or "bad" in lower: sentiment = -1.0
        
        memory_graph.add_node(agent_name, type="agent")
        memory_graph.add_node(topic, type="topic")
        memory_graph.add_edge(agent_name, topic, relation="has_opinion", weight=sentiment, content=response_text[:120])
    except: pass 

# --- MAIN LOOP (UPDATED: Non-Blocking Read) ---
while True:
    try:
        # UPDATED: Use readline() instead of iterator to avoid buffering deadlocks
        line = sys.stdin.readline()
        if not line: break # EOF
        
        line = line.strip()
        if not line: continue 
        
        req = json.loads(line)
        task = req.get("task")
        
        # === COMMAND ROUTER ===

        # --- NEW TASK: QUERY MEMORY (RAG + Live Fallback) ---
        if task == "query_memory":
            query = req.get("query", "")
            results = []
            
            # 1. Try Offline DB First (The 50k Reddit Records)
            if knowledge_table:
                try:
                    q_vec = embed_model.encode([query])[0]
                    # Search for top 5 closest semantic matches
                    hits = knowledge_table.search(q_vec).limit(5).to_list()
                    if hits:
                        results = [h["text"] for h in hits]
                except Exception as e:
                    # Proceed to fallback if DB error
                    pass
            
            # 2. Live Fallback (Self-Healing Memory)
            # If local DB had no results (empty list), we search the live web
            if not results:
                try:
                    with DDGS() as ddgs:
                        # Search specifically for reviews/discussions
                        web_hits = list(ddgs.text(f"{query} review india", max_results=4))
                        
                        new_data = []
                        for hit in web_hits:
                            text = f"[LIVE WEB] {hit['title']}: {hit['body']}"
                            results.append(text)
                            
                            # 3. Save to Brain (Future agents will see this instantly)
                            if knowledge_table:
                                new_data.append({
                                    "text": text,
                                    "source": "live_ddg",
                                    "category": "live_fallback",
                                    "score": 10,
                                    "date": datetime.datetime.now().isoformat(),
                                    "vector": embed_model.encode([text])[0]
                                })
                        
                        if new_data and knowledge_table:
                            knowledge_table.add(new_data)
                except Exception as e:
                    results.append(f"Web Search Failed: {str(e)}")

            print(json.dumps({"status": "success", "data": results}), flush=True)
            continue
        
        # TASK 1: RESEARCH (Federated Legacy)
        if task == "research":
            voices = perform_federated_research(req.get("product"), req.get("context"))
            print(json.dumps({"status": "success", "research_data": voices}), flush=True)
            continue

        # TASK 2: FACT CHECK (Open DBs)
        if task == "get_facts":
            fact = perform_fact_check(req.get("query"))
            print(json.dumps({"status": "success", "fact_sheet": fact}), flush=True)
            continue

        # TASK 3: INFERENCE
        if task == "generate":
            prompt = req.get("prompt", "")
            max_tokens = req.get("max_tokens", 300)
            temp = req.get("temperature", 0.0)
            
            # Metadata
            agent = "Unknown"
            topic = "General"
            if "Name:" in prompt: agent = prompt.split("Name:")[1].split("\n")[0].strip()
            if "Topic:" in prompt: topic = prompt.split("Topic:")[1].split("\n")[0].strip()

            # Context Assembly (Sharded)
            sharded_ctx = get_sharded_context(agent, topic)
            
            # RAG (PDFs)
            rag_ctx = ""
            if req.get("pdf"):
                pdf_hash = hashlib.md5(req.get("pdf").encode()).hexdigest()
                table_name = f"doc_{pdf_hash}"
                
                # Check if table exists (Using list_tables logic implicit in lancedb)
                if table_name not in db.table_names():
                    chunks = process_pdf(req.get("pdf"))
                    if chunks:
                        vecs = embed_model.encode(chunks)
                        data = [{"vector": v, "text": t} for v, t in zip(vecs, chunks)]
                        db.create_table(table_name, data=data)
                
                tbl = db.open_table(table_name)
                q_vec = embed_model.encode([prompt])[0]
                res = tbl.search(q_vec).limit(2).to_pandas()
                for _, r in res.iterrows(): rag_ctx += f"- {r['text'][:200]}...\n"

            # Final Prompt Injection
            final_context = f"{rag_ctx}\n{sharded_ctx}"
            
            if "<|user|>" in prompt:
                full_prompt = prompt.replace("<|user|>", f"<|user|>\n{final_context}\n", 1) + "<|end|>\n<|assistant|>"
            else:
                full_prompt = f"<|user|>\n{final_context}\n{prompt}<|end|>\n<|assistant|>"

            # Generate
            images = None
            if req.get("image"):
                try:
                    img = Image.open(io.BytesIO(base64.b64decode(req.get("image"))))
                    images = [img]
                    if "<|image_1|>" not in full_prompt:
                        full_prompt = full_prompt.replace("<|user|>", "<|user|>\n<|image_1|>", 1)
                except: pass

            if images:
                res = generate(model, processor, full_prompt, images, max_tokens=max_tokens, temp=temp, verbose=False)
            else:
                res = generate(model, processor, full_prompt, max_tokens=max_tokens, temp=temp, verbose=False)
                
            final_text = res.text.split("<|end|>")[0].strip()
            _update_graph_memory(agent, final_text, topic)
            
            print(json.dumps({"status": "success", "text": final_text}), flush=True)
            # Force cleanup to keep memory sane
            continue

    except Exception as e:
        print(json.dumps({"status": "error", "message": str(e)}), flush=True)