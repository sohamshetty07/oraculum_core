// src/brain.rs
// ORACULUM CORE - NEURAL ENGINE BRIDGE
// Connects Rust logic to Python's GPU-accelerated inference process

use std::process::{Command, Stdio, Child};
use std::io::{Write, BufReader, BufRead};
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct InferenceRequest {
    prompt: String,
    max_tokens: usize,
    // 1. NEW FIELD: Optional Base64 Image string
    image: Option<String>, 
    pdf: Option<String>,
    // 2. NEW FIELD: Temperature control for creativity
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
                        if let Ok(status) = serde_json::from_str::<serde_json::Value>(trimmed) {
                             if let Some(s) = status.get("status").and_then(|v| v.as_str()) {
                                 if s == "ready" {
                                     println!("✅ BRAIN: Neural Engine Online (M4 GPU Active)");
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

    // 2. UPDATED SIGNATURE: Now accepts optional image_b64 and temperature
    pub fn generate(&self, prompt: &str, max_tokens: usize, image_b64: Option<String>, pdf_b64: Option<String>, temp: f32) -> String {
        let mut child = self.python_process.lock().unwrap();
        
        // Send Request
        {
            let stdin = child.stdin.as_mut().expect("Failed to open stdin");
            let request = InferenceRequest {
                prompt: prompt.to_string(),
                max_tokens,
                image: image_b64, 
                pdf: pdf_b64,
                temperature: temp, // Pass temperature to Python
            };
            let json_req = serde_json::to_string(&request).unwrap();
            
            if let Err(e) = writeln!(stdin, "{}", json_req) {
                return format!("Error writing to Python: {}", e);
            }
        }

        // Read Response
        let stdout = child.stdout.as_mut().expect("Failed to open stdout");
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        
        if let Err(e) = reader.read_line(&mut line) {
             return format!("Error reading from Python: {}", e);
        }

        if line.is_empty() {
             return "Error: Empty response from Neural Engine".to_string();
        }

        let response: InferenceResponse = serde_json::from_str(&line).unwrap_or_else(|_| {
            InferenceResponse {
                status: "error".to_string(),
                text: Some(format!("Error parsing JSON: {}", line)),
                message: None,
            }
        });

        if response.status == "success" {
            response.text.unwrap_or_default()
        } else {
            response.message.unwrap_or_else(|| "Unknown error".to_string())
        }
    }

    // --- RESEARCH METHOD ---
    // This calls the "Deep Research Agent" inside the Python worker.
    pub fn research(&self, product: &str, context: &str) -> Vec<String> {
        let mut child = self.python_process.lock().unwrap();

        // Send JSON Command
        {
            let stdin = child.stdin.as_mut().expect("Failed to open stdin");
            
            // Construct the special "task": "research" payload
            let request = serde_json::json!({
                "task": "research",
                "product": product,
                "context": context
            });
            
            let json_req = serde_json::to_string(&request).unwrap();

            if let Err(e) = writeln!(stdin, "{}", json_req) {
                eprintln!("🧠 ERROR: Failed to send research command: {}", e);
                return Vec::new();
            }
        }

        // Read Response
        let stdout = child.stdout.as_mut().expect("Failed to open stdout");
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();

        if let Err(e) = reader.read_line(&mut line) {
             eprintln!("🧠 ERROR: Failed to read research response: {}", e);
             return Vec::new();
        }

        // Parse Response
        #[derive(Deserialize)]
        struct ResearchResponse {
            status: String,
            research_data: Option<Vec<String>>,
            message: Option<String>,
        }

        let response: ResearchResponse = serde_json::from_str(&line).unwrap_or_else(|_| {
            ResearchResponse {
                status: "error".to_string(),
                research_data: None,
                message: Some(format!("Invalid JSON from Python: {}", line)),
            }
        });

        if response.status == "success" {
            response.research_data.unwrap_or_default()
        } else {
            eprintln!("🧠 RESEARCH ERROR: {}", response.message.unwrap_or_default());
            Vec::new()
        }
    }

    // --- NEW: FACT CHECK METHOD ---
    // Calls the "Fact Agent" (Wikipedia + Search) in Python
    pub fn get_facts(&self, query: &str) -> String {
        let mut child = self.python_process.lock().unwrap();

        // Send JSON Command
        {
            let stdin = child.stdin.as_mut().expect("Failed to open stdin");
            
            let request = serde_json::json!({
                "task": "get_facts",
                "query": query
            });
            
            let json_req = serde_json::to_string(&request).unwrap();

            if let Err(e) = writeln!(stdin, "{}", json_req) {
                eprintln!("🧠 ERROR: Failed to send fact command: {}", e);
                return String::new();
            }
        }

        // Read Response
        let stdout = child.stdout.as_mut().expect("Failed to open stdout");
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();

        if let Err(e) = reader.read_line(&mut line) {
             eprintln!("🧠 ERROR: Failed to read fact response: {}", e);
             return String::new();
        }

        // Parse Response
        #[derive(Deserialize)]
        struct FactResponse {
            status: String,
            fact_sheet: Option<String>,
            message: Option<String>,
        }

        let response: FactResponse = serde_json::from_str(&line).unwrap_or_else(|_| {
            FactResponse {
                status: "error".to_string(),
                fact_sheet: None,
                message: Some(format!("Invalid JSON from Python: {}", line)),
            }
        });

        if response.status == "success" {
            response.fact_sheet.unwrap_or_default()
        } else {
            eprintln!("🧠 FACT ERROR: {}", response.message.unwrap_or_default());
            String::new()
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