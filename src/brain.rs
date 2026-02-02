// src/brain.rs
// ORACULUM CORE - NEURAL ENGINE BRIDGE
// Connects Rust logic to Python's GPU-accelerated inference process
// UPDATED: Robust Pipe Communication (No Deadlocks)

use std::process::{Command, Stdio, Child};
use std::io::{Write, BufReader, BufRead};
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};

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
    message: Option<String>,
}

pub struct AgentBrain {
    python_process: Arc<Mutex<Child>>,
}

impl AgentBrain {
    pub fn new() -> Self {
        println!("🧠 BRAIN: Waking up Neural Engine (Vision & Research Enabled)...");
        
        let mut child = Command::new("python3")
            .arg("python_bridge/inference_worker.py")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit()) 
            .spawn()
            .expect("Failed to spawn Python worker. Is the virtual env active? Run 'source .venv/bin/activate'");

        {
            let stdout = child.stdout.as_mut().expect("Failed to capture stdout from Python");
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();
            
            println!("   └── Waiting for Neural Models to load...");
            
            loop {
                line.clear();
                match reader.read_line(&mut line) {
                    Ok(0) => panic!("❌ Python process closed unexpectedly during startup."),
                    Ok(_) => {
                        let trimmed = line.trim();
                        if trimmed.is_empty() { continue; }
                        
                        if let Ok(status) = serde_json::from_str::<serde_json::Value>(trimmed) {
                             if let Some(s) = status.get("status").and_then(|v| v.as_str()) {
                                 if s == "ready" {
                                     if let Some(mem) = status.get("memory").and_then(|v| v.as_str()) {
                                         println!("✅ BRAIN: Neural Engine Online (M4 GPU Active) | Memory: {}", mem);
                                     } else {
                                         println!("✅ BRAIN: Neural Engine Online (M4 GPU Active)");
                                     }
                                     break;
                                 } else if s.contains("loading") {
                                     continue;
                                 } else if s == "error" {
                                     let msg = status.get("message").and_then(|v| v.as_str()).unwrap_or("Unknown error");
                                     panic!("❌ Python reported error: {}", msg);
                                 }
                             }
                        }
                    }
                    Err(e) => panic!("❌ Error reading from Python: {}", e),
                }
            }
        }

        Self {
            python_process: Arc::new(Mutex::new(child)),
        }
    }

    // --- CRITICAL HELPER: Safe Send & Receive ---
    fn send_command<T: Serialize, R: for<'de> Deserialize<'de>>(&self, request: &T) -> Result<R, String> {
        let mut child = self.python_process.lock().unwrap();
        
        // 1. Serialize
        // We replace any raw newlines in the JSON string itself to prevent the pipe reader 
        // in Python from thinking the message ended early.
        let mut json_req = serde_json::to_string(request).map_err(|e| e.to_string())?;
        
        // SANITIZATION: Remove internal newlines if any exist (though serde usually handles this)
        // This is a safety net.
        if json_req.contains('\n') {
             json_req = json_req.replace('\n', " "); 
        }

        if let Some(stdin) = child.stdin.as_mut() {
            // Write payload
            if let Err(e) = stdin.write_all(json_req.as_bytes()) {
                return Err(format!("Failed to write to stdin: {}", e));
            }
            // Write Delimiter
            if let Err(e) = stdin.write_all(b"\n") {
                return Err(format!("Failed to write newline: {}", e));
            }
            // CRITICAL: Force flush to ensure Python sees it immediately
            if let Err(e) = stdin.flush() {
                return Err(format!("Failed to flush stdin: {}", e));
            }
        } else {
            return Err("Failed to open stdin".to_string());
        }

        // 2. Read Response
        if let Some(stdout) = child.stdout.as_mut() {
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();
            
            if let Err(e) = reader.read_line(&mut line) {
                return Err(format!("Failed to read from stdout: {}", e));
            }
            
            let trimmed = line.trim();
            if trimmed.is_empty() {
                return Err("Empty response from Python (Process might have crashed)".to_string());
            }

            serde_json::from_str::<R>(trimmed).map_err(|e| format!("JSON Parse Error: {} | Input: {}", e, trimmed))
        } else {
            Err("Failed to open stdout".to_string())
        }
    }

    // --- API METHODS ---

    pub fn generate(&self, prompt: &str, max_tokens: usize, image_b64: Option<String>, pdf_b64: Option<String>, temp: f32) -> String {
        let request = InferenceRequest {
            prompt: prompt.to_string(),
            max_tokens,
            image: image_b64, 
            pdf: pdf_b64,
            temperature: temp, 
        };

        match self.send_command::<_, InferenceResponse>(&request) {
            Ok(res) => {
                if res.status == "success" {
                    res.text.unwrap_or_default()
                } else {
                    format!("Error: {}", res.message.unwrap_or("Unknown".to_string()))
                }
            },
            Err(e) => format!("System Error: {}", e)
        }
    }

    pub fn query_memory(&self, query: &str) -> Vec<String> {
        let request = serde_json::json!({
            "task": "query_memory",
            "query": query
        });

        #[derive(Deserialize)]
        struct MemoryResponse {
            status: String,
            data: Option<Vec<String>>,
            #[allow(dead_code)] message: Option<String>,
        }

        match self.send_command::<_, MemoryResponse>(&request) {
            Ok(res) => {
                if res.status == "success" {
                    res.data.unwrap_or_default()
                } else {
                    Vec::new()
                }
            },
            Err(e) => {
                eprintln!("🧠 MEMORY ERROR: {}", e);
                Vec::new()
            }
        }
    }

    pub fn research(&self, product: &str, context: &str) -> Vec<String> {
        let request = serde_json::json!({
            "task": "research",
            "product": product,
            "context": context
        });

        #[derive(Deserialize)]
        struct ResearchResponse {
            status: String,
            research_data: Option<Vec<String>>,
            #[allow(dead_code)] message: Option<String>,
        }

        match self.send_command::<_, ResearchResponse>(&request) {
            Ok(res) => {
                 if res.status == "success" {
                    res.research_data.unwrap_or_default()
                 } else {
                    Vec::new()
                 }
            },
            Err(e) => {
                eprintln!("🧠 RESEARCH ERROR: {}", e);
                Vec::new()
            }
        }
    }

    pub fn get_facts(&self, query: &str) -> String {
        let request = serde_json::json!({
            "task": "get_facts",
            "query": query
        });

        #[derive(Deserialize)]
        struct FactResponse {
            status: String,
            fact_sheet: Option<String>,
            #[allow(dead_code)] message: Option<String>,
        }

        match self.send_command::<_, FactResponse>(&request) {
            Ok(res) => {
                if res.status == "success" {
                    res.fact_sheet.unwrap_or_default()
                } else {
                    String::new()
                }
            },
            Err(e) => {
                eprintln!("🧠 FACT ERROR: {}", e);
                String::new()
            }
        }
    }
}

impl Drop for AgentBrain {
    fn drop(&mut self) {
        if let Ok(mut child) = self.python_process.lock() {
            println!("🧠 BRAIN: Shutting down Neural Engine...");
            let _ = child.kill();
        }
    }
}