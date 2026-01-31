# python_bridge/inference_worker.py
import sys
import json
import base64
import io
import hashlib
import re
import networkx as nx 
import lancedb
from pypdf import PdfReader
from sentence_transformers import SentenceTransformer
from mlx_vlm import load, generate
from PIL import Image

# --- RESEARCH AGENT TOOLS ---
from duckduckgo_search import DDGS
import trafilatura

# --- CONFIGURATION ---
MODEL_PATH = "mlx-community/Phi-3.5-vision-instruct-4bit"
EMBEDDING_MODEL = "all-MiniLM-L6-v2" 
DB_PATH = "./data/lancedb_store"

# --- GLOBAL MEMORY STATE (MAGMA LAYER) ---
# We use a Directed Graph to store Agent Opinions and Relationships
memory_graph = nx.DiGraph()
ddgs = DDGS() # Persistent Search Client

# --- INITIALIZATION ---
print(json.dumps({"status": "loading_models"}), flush=True)

try:
    # 1. Load GenAI Model (Vision + Text)
    model, processor = load(MODEL_PATH, trust_remote_code=True)
    
    # 2. Load Embedding Model (for RAG)
    embed_model = SentenceTransformer(EMBEDDING_MODEL)
    
    # 3. Setup Vector DB
    db = lancedb.connect(DB_PATH)
    
    print(json.dumps({"status": "ready"}), flush=True)
except Exception as e:
    print(json.dumps({"status": "error", "message": f"Startup Error: {str(e)}"}), flush=True)
    sys.exit(1)

# --- HELPER: PDF PROCESSING ---
def process_pdf(pdf_b64):
    """Decodes PDF, chunks text, returns chunks."""
    try:
        pdf_data = base64.b64decode(pdf_b64)
        reader = PdfReader(io.BytesIO(pdf_data))
        text = ""
        for page in reader.pages:
            text += page.extract_text() + "\n"
        
        words = text.split()
        chunk_size = 200
        overlap = 50
        chunks = []
        for i in range(0, len(words), chunk_size - overlap):
            chunk = " ".join(words[i:i + chunk_size])
            chunks.append(chunk)
        return chunks
    except Exception as e:
        return []

# --- TOOL 1: DEEP RESEARCH AGENT (Robust Mode) ---
def perform_deep_research(topic, audience_context):
    """
    Autonomous Research Loop.
    Strategy: 
    1. Capture search snippets IMMEDIATELY (Fast Voices).
    2. Try to scrape full pages for deeper context (Deep Voices).
    3. Aggregate and deduplicate.
    """
    collected_voices = []
    clean_topic = topic.split('(')[0].strip()
    
    # Layered Search Queries
    search_layers = [
        # Layer 1: Specific Intent
        [f"{topic} reddit", f"{topic} review", f"{topic} complaints"],
        # Layer 2: Broad Topic + Region (Try to hit forums)
        [f"{clean_topic} india discussion", f"{clean_topic} taste test", f"{clean_topic} price opinion"],
        # Layer 3: Category level
        [f"best {clean_topic} brand reviews", f"packaged {clean_topic} feedback"]
    ]
    
    seen_content = set()

    for layer in search_layers:
        if len(collected_voices) >= 12: break
        
        for q in layer:
            try:
                # Get results. We assume generic region to cast a wide net, 
                # but if product is specific, DDG handles it well.
                results = ddgs.text(q, max_results=3)
                if not results: continue

                for r in results:
                    # STRATEGY 1: SNIPPET CAPTURE (Guaranteed Data)
                    # Often the snippet itself contains the "Voice" we need.
                    snippet = r['body']
                    if len(snippet) > 60 and snippet not in seen_content:
                        collected_voices.append(f"[Snippet] {snippet}")
                        seen_content.add(snippet)

                    # STRATEGY 2: DEEP SCRAPE (Best Effort)
                    try:
                        url = r['href']
                        downloaded = trafilatura.fetch_url(url)
                        if downloaded:
                            text = trafilatura.extract(downloaded)
                            if text:
                                # Find paragraph-sized chunks that look like opinions
                                paras = [p for p in text.split('\n') if 80 < len(p) < 500]
                                for p in paras[:2]: # Take top 2 meaningful paragraphs
                                    if p not in seen_content:
                                        collected_voices.append(p)
                                        seen_content.add(p)
                    except:
                        pass # Ignore scraping failures, rely on snippets
            except:
                continue

    # 3. SYNTHESIS PHASE
    if not collected_voices:
        # STRICT: No synthetic fallback. If we fail, we fail honestly.
        return ["SYSTEM_ALERT: No wild voices found. The internet is silent on this specific topic."]
        
    # Return top results, mixing snippets and deep scrapes
    return list(collected_voices)[:15]

# --- TOOL 2: FACT CHECK AGENT (Robust Specs) ---
def perform_fact_check(query):
    """
    Searches for official specifications.
    Uses snippets immediately to build context buffer, avoiding empty returns.
    """
    context_buffer = ""
    clean_query = query.split('(')[0].strip()
    
    try:
        # Broader query to catch e-commerce listings which have specs
        results = ddgs.text(f"{clean_query} specifications price ingredients release date", max_results=4)
        
        if results:
            for r in results:
                context_buffer += f"Source: {r['title']}\nSnippet: {r['body']}\n---\n"
                
                # Try one deep read for the top result
                if len(context_buffer) < 500: # Only if we need more info
                    try:
                        downloaded = trafilatura.fetch_url(r['href'])
                        text = trafilatura.extract(downloaded)
                        if text:
                            context_buffer += f"\nDeep Info: {text[:1000]}\n"
                    except: pass
    except Exception as e:
        return f"Error searching facts: {str(e)}"

    if not context_buffer:
        return "SYSTEM_ALERT: No verifiable facts found online."

    # Synthesize
    prompt = f"<|user|>Extract factual specs for '{query}' from this data:\n{context_buffer}\nOUTPUT FORMAT:\nSpecs: ...\nPrice: ...\nOrigin: ...<|end|>\n<|assistant|>"
    
    try:
        response = generate(model, processor, prompt, max_tokens=300, verbose=False)
        return response.text.split("<|end|>")[0].strip()
    except Exception as e:
        return f"Error synthesizing facts: {str(e)}"

# --- MAGMA ENGINE: GRAPH OPERATIONS ---

def _update_graph_memory(agent_name, response_text, topic):
    try:
        sentiment = 0.0
        lower_res = response_text.lower()
        if any(w in lower_res for w in ["love", "great", "excellent", "buy", "amazing", "yes"]):
            sentiment = 1.0
        elif any(w in lower_res for w in ["hate", "bad", "terrible", "avoid", "expensive", "no"]):
            sentiment = -1.0
            
        memory_graph.add_node(agent_name, type="agent")
        memory_graph.add_node(topic, type="topic")
        
        # Store sentiment and a snippet for retrieval
        memory_graph.add_edge(agent_name, topic, relation="has_opinion", weight=sentiment, content=response_text[:120])
    except Exception as e:
        pass 

def _retrieve_graph_context(current_agent, topic, limit=5):
    if not memory_graph.has_node(topic):
        return ""

    context = []
    predecessors = list(memory_graph.predecessors(topic))
    
    for agent in predecessors:
        if agent == current_agent: continue 
        
        edge_data = memory_graph.get_edge_data(agent, topic)
        sentiment = "positive" if edge_data['weight'] > 0 else "negative" if edge_data['weight'] < 0 else "neutral"
        snippet = edge_data.get('content', '...')
        
        context.append(f"- {agent} was {sentiment}: \"{snippet}...\"")
    
    if not context:
        return ""
        
    return "\n[MEMORY] What others have said:\n" + "\n".join(context[:limit]) + "\n"

# --- MAIN LOOP ---
for line in sys.stdin:
    if not line:
        break
        
    try:
        req = json.loads(line)
        
        # === COMMAND ROUTER ===
        
        # MODE 1: DEEP RESEARCH
        if req.get("task") == "research":
            product = req.get("product")
            context = req.get("context")
            voices = perform_deep_research(product, context)
            print(json.dumps({"status": "success", "research_data": voices}), flush=True)
            continue

        # MODE 2: FACT CHECK
        if req.get("task") == "get_facts":
            query = req.get("query")
            fact_sheet = perform_fact_check(query)
            print(json.dumps({"status": "success", "fact_sheet": fact_sheet}), flush=True)
            continue

        # MODE 3: INFERENCE (Standard Generation)
        prompt = req.get("prompt", "")
        max_tokens = req.get("max_tokens", 300)
        image_b64 = req.get("image", None)
        pdf_b64 = req.get("pdf", None)
        
        # --- METADATA & CONTEXT ---
        agent_name = "Unknown"
        topic = "General"
        
        # Extract metadata from prompt (sent by Rust)
        if "Name:" in prompt: 
            agent_name = prompt.split("Name:")[1].split("\n")[0].strip()
        if "Topic:" in prompt: 
            topic = prompt.split("Topic:")[1].split("\n")[0].strip()

        # RAG / Graph Retrieval
        rag_context = ""
        
        # Vector DB (PDFs)
        if pdf_b64:
            pdf_hash = hashlib.md5(pdf_b64.encode()).hexdigest()
            table_name = f"doc_{pdf_hash}"
            if table_name not in db.list_tables():
                chunks = process_pdf(pdf_b64)
                if chunks:
                    embeddings = embed_model.encode(chunks)
                    data = [{"vector": e, "text": c} for e, c in zip(embeddings, chunks)]
                    db.create_table(table_name, data=data)
            
            if table_name in db.list_tables():
                tbl = db.open_table(table_name)
                query_vec = embed_model.encode([prompt])[0]
                results = tbl.search(query_vec).limit(2).to_pandas()
                rag_context += "\n[DOCS]:\n"
                for _, row in results.iterrows():
                    rag_context += f"- {row['text'][:200]}...\n"

        # Graph Memory
        graph_context = _retrieve_graph_context(agent_name, topic)

        # --- PROMPT ASSEMBLY (FIXED DOUBLE-WRAPPING) ---
        # The Rust backend sends prompts wrapped in <|user|>...
        # We must inject our context *inside* that tag, not wrap it again.
        
        combined_context = f"{rag_context}\n{graph_context}"
        
        if "<|user|>" in prompt:
            # Inject context after the first <|user|> tag
            # Pattern: <|user|> ... content ...
            # Result: <|user|>\n[MEMORY]...\n ... content ...
            full_prompt = prompt.replace("<|user|>", f"<|user|>\n{combined_context}\n", 1) + "<|end|>\n<|assistant|>"
        else:
            # Fallback if raw text is sent
            full_prompt = f"<|user|>\n{combined_context}\n{prompt}<|end|>\n<|assistant|>"

        # --- VISION / TEXT HANDLING ---
        processed_images = None
        
        if image_b64:
            try:
                raw = base64.b64decode(image_b64)
                img = Image.open(io.BytesIO(raw))
                processed_images = [img]
                # Ensure <|image_1|> tag exists
                if "<|image_1|>" not in full_prompt:
                     full_prompt = full_prompt.replace("<|user|>", "<|user|>\n<|image_1|>", 1)
            except:
                pass

        # --- GENERATION ---
        if processed_images:
             response = generate(model, processor, full_prompt, processed_images, max_tokens=max_tokens, verbose=False, temp=0.7)
        else:
             response = generate(model, processor, full_prompt, max_tokens=max_tokens, verbose=False, temp=0.7)
        
        res = response.text if hasattr(response, "text") else str(response)
        
        # Cleanup output
        if "<|end|>" in res:
            res = res.split("<|end|>")[0]
        
        res_clean = res.strip()

        # Update Memory
        _update_graph_memory(agent_name, res_clean, topic)

        print(json.dumps({"status": "success", "text": res_clean}), flush=True)

    except Exception as e:
        print(json.dumps({"status": "error", "message": f"Brain Error: {str(e)}"}), flush=True)