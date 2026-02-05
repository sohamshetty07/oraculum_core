# Oraculum Core

**Oraculum** is an AI-powered market research simulation platform. It uses synthetic agents (AI personas) to simulate consumer behaviour, providing "Glass Box" insights for product launches, marketing campaigns, and competitor wargaming.

## 🧠 System Architecture

Oraculum operates on a **Three-Tier Microservice Architecture**:

1.  **The Body (Rust Backend)** `src/`
    * **Role:** High-performance orchestration, job management, and persona generation.
    * **Tech:** Actix-Web, Rayon (Parallel Processing), DashMap.
    * **Function:** Spawns agent swarms, manages the simulation lifecycle, and aggregates results.

2.  **The Sensory Cortex (Python Microservice)** `sensory_cortex/`
    * **Role:** The "Eyes and Ears" of the system. Handles unstructured web data.
    * **Tech:** FastAPI, Crawl4AI, Ollama Bridge.
    * **Function:** Performs live web scraping, sentiment analysis, and context retrieval to ground the agents in reality.

3.  **The Interface (Frontend)** `frontend/`
    * **Role:** User interaction and visualization.
    * **Tech:** Next.js (React), Tailwind CSS, Framer Motion.
    * **Function:** Displays real-time agent debates, thought processes, and aggregated insights.

---

## ⚡️ Quick Start (Fresh Installation)

Follow these steps to deploy Oraculum on a fresh machine (Apple Silicon recommended).

### 1. Prerequisites
* **Rust:** `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
* **Python 3.11+:** [Download from python.org](https://www.python.org/downloads/)
* **Node.js 18+:** [Download from nodejs.org](https://nodejs.org/)
* **Ollama:** [Download from ollama.com](https://ollama.com/)

### 2. System Setup

#### A. Initialize the Neural Engine (Ollama)
Oraculum relies on local LLMs for privacy and zero-cost inference.

```bash
ollama pull qwen2.5   # The reasoning engine used by both Brain and Cortex

```

#### B. Setup the Sensory Cortex (Python)

Navigate to the Python service directory (ensure you are in the project root):

```bash
cd sensory_cortex
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt
playwright install  # Critical: Installs the browser engine for scraping
cd ..

```

#### C. Setup the Frontend

```bash
cd frontend
npm install
cd ..

```

---

## 🚀 Running the System

You need to run the **Brain**, the **Body**, and the **UI** simultaneously in three separate terminals.

**Terminal 1: The Sensory Cortex (Python)**
This must be running **before** the Rust backend starts so the Brain can "see".

```bash
cd sensory_cortex
source venv/bin/activate
python3 -m uvicorn main:app --reload --port 8000

```

*Wait until you see: `Uvicorn running on http://127.0.0.1:8000*`

**Terminal 2: The Core (Rust)**
This runs the simulation engine.

```bash
cargo run

```

*Wait until you see: `🚀 Oraculum API Server Starting...` and `🌍 Server running at http://127.0.0.1:8080*`

**Terminal 3: The Frontend**
This runs the user interface.

```bash
cd frontend
npm run dev

```

*Open `http://localhost:3000` in your browser.*

---

## 📂 Project Structure

| Folder | Description |
| --- | --- |
| `src/` | **Rust Core.** Contains the logic for `agent_swarm`, `scenarios`, `skills`, and `systems`. |
| `sensory_cortex/` | **Python Bridge.** Contains the `main.py` (FastAPI) and `crawl4ai` logic for web scouting. |
| `frontend/` | **React UI.** The dashboard for running simulations and viewing reports. |
| `knowledge_db/` | **LanceDB.** Local vector storage for long-term agent memories (optional). |

---

## 🛠 Troubleshooting

* **"Sensory Cortex Unreachable":** Ensure Terminal 1 is running and shows "Application startup complete".
* **"Connection Refused" in Rust:** Check if port 8000 is blocked or if Python is running on a different port.
* **"Hallucinations" (Agent speaking nonsense):** Ensure `qwen2.5` is loaded in Ollama. Lighter models (like phi-3) may struggle with complex tool use.

## 🛡️ Licence

Proprietary & Confidential.

```

```