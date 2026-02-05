import asyncio
import ollama
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from crawl4ai import AsyncWebCrawler

app = FastAPI()

# --- CONFIG ---
MODEL_NAME = "qwen2.5"

# Input Schema (What Rust sends)
class SensoryRequest(BaseModel):
    url: str
    query: str

# Output Schema (What Rust receives)
class SensoryResponse(BaseModel):
    knowledge: str

@app.post("/perceive", response_model=SensoryResponse)
async def perceive(request: SensoryRequest):
    print(f"\n[ORACULUM REQUEST] Scanning: {request.url} for '{request.query}'")
    
    try:
        # 1. THE EYE (Crawl4AI)
        async with AsyncWebCrawler(verbose=False) as crawler:
            result = await crawler.arun(url=request.url, bypass_cache=True)
            
            if not result.markdown:
                raise HTTPException(status_code=500, detail="Failed to acquire visual data")

        # 2. THE BRAIN (Ollama)
        response = ollama.chat(model=MODEL_NAME, messages=[
            {
                'role': 'system',
                'content': "You are the Oraculum Cortex. Extract the requested facts from the raw data. Be concise."
            },
            {
                'role': 'user',
                'content': f"Raw Data:\n{result.markdown[:15000]}\n\nUser Goal: {request.query}"
            }
        ])
        
        knowledge = response['message']['content']
        print(f"[ORACULUM RESPONSE] {knowledge}")
        return SensoryResponse(knowledge=knowledge)

    except Exception as e:
        print(f"[ERROR] {e}")
        raise HTTPException(status_code=500, detail=str(e))

# Run with: python -m uvicorn oraculum_server:app --reload