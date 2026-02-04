# python_bridge/inference_worker.py
# SYSTEM V5.1: HTTP MICROSERVICE BRAIN (FastAPI + Uvicorn)
# UPDATED: Added Global GPU Lock to prevent Metal/MPS Crashes on M-Series Chips

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
import uvicorn
import threading 
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from typing import Optional, List, Any

# MLX & Intelligence Modules
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
PORT = 8003 

# --- GLOBAL STATE ---
app = FastAPI(title="Oraculum Neural Engine")
memory_graph = nx.DiGraph()
model = None
processor = None
embed_model = None
db = None
knowledge_table = None

# --- CRITICAL: GPU LOCK ---
# Metal (MPS) cannot handle parallel command buffer commits from threads.
# We must serialize all model access.
gpu_lock = threading.Lock()

# --- REQUEST MODELS ---
class GenerateRequest(BaseModel):
    prompt: str
    max_tokens: int = 300
    temperature: float = 0.0
    image: Optional[str] = None
    pdf: Optional[str] = None

class ResearchRequest(BaseModel):
    product: str
    context: str

class QueryRequest(BaseModel):
    query: str

# --- LIFECYCLE STARTUP ---
@app.on_event("startup")
async def startup_event():
    global model, processor, embed_model, db, knowledge_table
    print("🧠 BRAIN SERVER: Loading Models...", flush=True)
    
    try:
        # Load models inside lock just to be safe, though startup is sequential
        with gpu_lock:
            model, processor = load(MODEL_PATH, trust_remote_code=True)
            embed_model = SentenceTransformer(EMBEDDING_MODEL)
        
        db = lancedb.connect(DB_PATH)
        existing_tables = db.table_names()
        if "memory_bank" in existing_tables:
            knowledge_table = db.open_table("memory_bank")
            print("✅ BRAIN: Memory Bank Loaded", flush=True)
        else:
            knowledge_table = None
            print("⚠️ BRAIN: Memory Bank Empty (Will self-heal)", flush=True)
            
        print(f"✅ BRAIN: Neural Engine Online on http://127.0.0.1:{PORT}", flush=True)
    except Exception as e:
        print(f"❌ FATAL: Model Load Failed: {e}", flush=True)
        sys.exit(1)

# --- HELPER FUNCTIONS ---
def clean_query_step(query):
    original = query
    query = re.sub(r'\b\d+(ml|g|kg|L|oz)\b', '', query, flags=re.IGNORECASE)
    query = re.sub(r'\(.*?\)|pack of \d+|official|review', '', query, flags=re.IGNORECASE)
    query = re.sub(r'\b(for|with|and|flavor|flavour|variant)\b', '', query, flags=re.IGNORECASE)
    query = " ".join(query.split()) 
    if query == original:
        tokens = query.split()
        if len(tokens) > 1: query = " ".join(tokens[:-1])
    return query

def perform_federated_research(topic, audience_context):
    voices = []
    current_q = topic
    for _ in range(3): 
        try:
            url = f"https://www.reddit.com/search.json?q={urllib.parse.quote(current_q)}&sort=relevance&limit=8"
            resp = requests.get(url, headers={'User-Agent': USER_AGENT}, timeout=5)
            if resp.status_code == 200:
                posts = resp.json().get('data', {}).get('children', [])
                if posts:
                    for p in posts:
                        title = p['data']['title']
                        if len(title) > 15: voices.append(title)
                    break 
        except: pass
        new_q = clean_query_step(current_q)
        if new_q == current_q: break
        current_q = new_q

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

def perform_fact_check(query):
    current_q = query
    for _ in range(3):
        try:
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
        new_q = clean_query_step(current_q)
        if new_q == current_q: break
        current_q = new_q

    return "SYSTEM_ALERT: No structured data found in Open Databases."

def get_sharded_context(agent_name, topic):
    if not memory_graph.has_node(topic): return ""
    all_opinions = []
    for peer in memory_graph.predecessors(topic):
        if peer == agent_name: continue
        edge = memory_graph.get_edge_data(peer, topic)
        all_opinions.append(f"- {peer}: \"{edge.get('content','...')}\"")
    
    if not all_opinions: return ""
    shard_index = sum(ord(c) for c in agent_name) % 3
    shard_size = max(1, len(all_opinions) // 2)
    start = (shard_index * shard_size) % len(all_opinions)
    selected = all_opinions[start : start + 3]
    return "\n[WHAT OTHERS ARE SAYING - YOUR FEED]:\n" + "\n".join(selected) + "\n"

def process_pdf(pdf_b64):
    try:
        pdf_data = base64.b64decode(pdf_b64)
        reader = PdfReader(io.BytesIO(pdf_data))
        text = "".join([page.extract_text() for page in reader.pages])
        words = text.split()
        return [" ".join(words[i:i+200]) for i in range(0, len(words), 150)]
    except: return []

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

# --- API ENDPOINTS ---

@app.get("/health")
def health_check():
    return {"status": "ready"}

@app.post("/generate")
def generate_text(req: GenerateRequest):
    try:
        agent = "Unknown"
        topic = "General"
        if "Name:" in req.prompt: agent = req.prompt.split("Name:")[1].split("\n")[0].strip()
        if "Topic:" in req.prompt: topic = req.prompt.split("Topic:")[1].split("\n")[0].strip()

        sharded_ctx = get_sharded_context(agent, topic)
        
        # PDF RAG (Needs GPU for embedding)
        rag_ctx = ""
        if req.pdf:
            pdf_hash = hashlib.md5(req.pdf.encode()).hexdigest()
            table_name = f"doc_{pdf_hash}"
            
            # LOCK GPU for embedding
            with gpu_lock:
                if table_name not in db.table_names():
                    chunks = process_pdf(req.pdf)
                    if chunks:
                        vecs = embed_model.encode(chunks)
                        data = [{"vector": v, "text": t} for v, t in zip(vecs, chunks)]
                        db.create_table(table_name, data=data)
                tbl = db.open_table(table_name)
                q_vec = embed_model.encode([req.prompt])[0]
            
            # DB search is CPU bound, safe outside lock
            res = tbl.search(q_vec).limit(2).to_pandas()
            for _, r in res.iterrows(): rag_ctx += f"- {r['text'][:200]}...\n"

        final_context = f"{rag_ctx}\n{sharded_ctx}"
        full_prompt = req.prompt
        if "<|user|>" in full_prompt:
            full_prompt = full_prompt.replace("<|user|>", f"<|user|>\n{final_context}\n", 1) + "<|end|>\n<|assistant|>"
        else:
            full_prompt = f"<|user|>\n{final_context}\n{full_prompt}<|end|>\n<|assistant|>"

        images = None
        if req.image:
            try:
                img = Image.open(io.BytesIO(base64.b64decode(req.image)))
                images = [img]
                if "<|image_1|>" not in full_prompt:
                     full_prompt = full_prompt.replace("<|user|>", "<|user|>\n<|image_1|>", 1)
            except: pass

        # CRITICAL: GPU INFERENCE MUST BE LOCKED
        # Only one thread can run 'generate' at a time on Metal
        with gpu_lock:
            if images:
                 res = generate(model, processor, full_prompt, images, max_tokens=req.max_tokens, temp=req.temperature, verbose=False)
            else:
                 res = generate(model, processor, full_prompt, max_tokens=req.max_tokens, temp=req.temperature, verbose=False)
            
        final_text = res.text.split("<|end|>")[0].strip()
        _update_graph_memory(agent, final_text, topic)
        
        return {"status": "success", "text": final_text}

    except Exception as e:
        print(f"Generate Error: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/query_memory")
def query_memory_endpoint(req: QueryRequest):
    results = []
    
    # 1. Try Offline DB First
    if knowledge_table:
        try:
            # LOCK GPU for embedding
            with gpu_lock:
                q_vec = embed_model.encode([req.query])[0]
            
            hits = knowledge_table.search(q_vec).limit(5).to_list()
            if hits:
                results = [h["text"] for h in hits]
        except: pass
    
    # 2. Live Fallback
    if not results:
        try:
            with DDGS() as ddgs:
                web_hits = list(ddgs.text(f"{req.query} review india", max_results=4))
                new_data = []
                for hit in web_hits:
                    text = f"[LIVE WEB] {hit['title']}: {hit['body']}"
                    results.append(text)
                    if knowledge_table:
                        # LOCK GPU for embedding
                        with gpu_lock:
                            vec = embed_model.encode([text])[0]
                        new_data.append({
                            "text": text,
                            "source": "live_ddg",
                            "category": "live_fallback",
                            "score": 10,
                            "date": datetime.datetime.now().isoformat(),
                            "vector": vec
                        })
                if new_data and knowledge_table:
                    knowledge_table.add(new_data)
        except Exception as e:
            results.append(f"Web Search Failed: {str(e)}")
            
    return {"status": "success", "data": results}

@app.post("/research")
def research_endpoint(req: ResearchRequest):
    voices = perform_federated_research(req.product, req.context)
    return {"status": "success", "research_data": voices}

@app.post("/get_facts")
def facts_endpoint(req: QueryRequest):
    fact = perform_fact_check(req.query)
    return {"status": "success", "fact_sheet": fact}

if __name__ == "__main__":
    uvicorn.run(app, host="127.0.0.1", port=PORT)