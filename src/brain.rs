// src/brain.rs
// ORACULUM CORE - NEURAL ENGINE BRIDGE
// V5.1: UREQ Implementation (Sync/Async Safe)
// Fixes "Cannot drop runtime" panic by using a pure blocking client.

use std::process::{Command, Stdio, Child};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use serde::{Deserialize, Serialize};
// Uses 'ureq' for safe blocking HTTP calls inside Async runtimes
use ureq; 

const PYTHON_API_URL: &str = "http://127.0.0.1:8001";

#[derive(Serialize)]
struct InferenceRequest {
    prompt: String,
    max_tokens: usize,
    image: Option<String>, 
    pdf: Option<String>,
    temperature: f32, 
}

#[derive(Deserialize, Debug)]
struct InferenceResponse {
    status: String,
    text: Option<String>,
    #[allow(dead_code)]
    message: Option<String>,
}

pub struct AgentBrain {
    python_process: Arc<Mutex<Child>>,
    // ureq uses an Agent to hold connection pools and config
    agent: ureq::Agent,
}

impl AgentBrain {
    pub fn new() -> Self {
        println!("🧠 BRAIN: Initializing Neural Engine (HTTP Mode - Safe)...");
        
        // 1. Spawn Python Server
        let child = Command::new("python3")
            .arg("python_bridge/inference_worker.py")
            .stdout(Stdio::inherit()) 
            .stderr(Stdio::inherit())
            .spawn()
            .expect("Failed to spawn Python worker. Is the virtual env active? Run 'source .venv/bin/activate'");

        // Create a persistent agent (keeps connections open)
        let agent = ureq::AgentBuilder::new()
            .timeout_read(Duration::from_secs(300)) // 5 minute timeout for long inferences
            .timeout_write(Duration::from_secs(10))
            .build();

        // 2. Wait for Health Check
        println!("   └── Waiting for Python API to come online...");
        let mut attempts = 0;
        loop {
            if attempts > 60 { panic!("❌ Python Brain timed out (60s). Check logs."); }
            
            // Simple GET request using ureq
            match agent.get(&format!("{}/health", PYTHON_API_URL)).call() {
                Ok(resp) => {
                    if resp.status() == 200 {
                        println!("✅ BRAIN: Connection Established on Port 8001");
                        break;
                    }
                },
                Err(_) => {
                    thread::sleep(Duration::from_secs(1));
                    attempts += 1;
                }
            }
        }

        Self {
            python_process: Arc::new(Mutex::new(child)),
            agent,
        }
    }

    // --- API METHODS ---

    pub fn generate(&self, prompt: &str, max_tokens: usize, image_b64: Option<String>, pdf_b64: Option<String>, temp: f32) -> String {
        let req_body = InferenceRequest {
            prompt: prompt.to_string(),
            max_tokens,
            image: image_b64, 
            pdf: pdf_b64,
            temperature: temp, 
        };

        match self.agent.post(&format!("{}/generate", PYTHON_API_URL)).send_json(&req_body) {
            Ok(resp) => {
                // ureq returns a Reader, we convert to JSON
                match resp.into_json::<InferenceResponse>() {
                    Ok(json) => {
                        if json.status == "success" {
                            json.text.unwrap_or_default()
                        } else {
                            format!("Error: {}", json.message.unwrap_or("Unknown error".to_string()))
                        }
                    },
                    Err(e) => format!("JSON Parse Error: {}", e)
                }
            },
            Err(e) => format!("Network Error: {}", e)
        }
    }

    pub fn query_memory(&self, query: &str) -> Vec<String> {
        #[derive(Deserialize)]
        struct QueryResp {
            status: String,
            data: Option<Vec<String>>,
            #[allow(dead_code)] message: Option<String>,
        }

        let body = serde_json::json!({ "query": query });

        match self.agent.post(&format!("{}/query_memory", PYTHON_API_URL)).send_json(body) {
            Ok(resp) => {
                if let Ok(json) = resp.into_json::<QueryResp>() {
                    json.data.unwrap_or_default()
                } else {
                    Vec::new()
                }
            },
            Err(e) => {
                eprintln!("🧠 MEMORY NETWORK ERROR: {}", e);
                Vec::new()
            }
        }
    }

    pub fn research(&self, product: &str, context: &str) -> Vec<String> {
        #[derive(Deserialize)]
        struct ResearchResp {
            status: String,
            research_data: Option<Vec<String>>,
            #[allow(dead_code)] message: Option<String>,
        }

        let body = serde_json::json!({ "product": product, "context": context });

        match self.agent.post(&format!("{}/research", PYTHON_API_URL)).send_json(body) {
            Ok(resp) => {
                if let Ok(json) = resp.into_json::<ResearchResp>() {
                    json.research_data.unwrap_or_default()
                } else {
                    Vec::new()
                }
            },
            Err(e) => {
                eprintln!("🧠 RESEARCH NETWORK ERROR: {}", e);
                Vec::new()
            }
        }
    }

    pub fn get_facts(&self, query: &str) -> String {
        #[derive(Deserialize)]
        struct FactResp {
            status: String,
            fact_sheet: Option<String>,
            #[allow(dead_code)] message: Option<String>,
        }

        let body = serde_json::json!({ "query": query });

        match self.agent.post(&format!("{}/get_facts", PYTHON_API_URL)).send_json(body) {
            Ok(resp) => {
                if let Ok(json) = resp.into_json::<FactResp>() {
                    json.fact_sheet.unwrap_or_default()
                } else {
                    String::new()
                }
            },
            Err(e) => {
                eprintln!("🧠 FACT NETWORK ERROR: {}", e);
                String::new()
            }
        }
    }
}

impl Drop for AgentBrain {
    fn drop(&mut self) {
        if let Ok(mut child) = self.python_process.lock() {
            println!("🧠 BRAIN: Shutting down Python Server...");
            let _ = child.kill();
        }
    }
}