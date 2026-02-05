// src/systems/sensory.rs
// THE SENSORY CORTEX CLIENT
// Connects the Rust Core to the Python Sensory Agent (Microservice)

use serde::{Deserialize, Serialize};
use std::time::Duration;

// 1. The Data We Send to Python
#[derive(Serialize)]
struct CortexRequest {
    url: String,
    query: String,
}

// 2. The Data Python Sends Back
#[derive(Deserialize)]
struct CortexResponse {
    knowledge: String,
}

// 3. The Public Interface
// This acts as a static helper class that 'skills.rs' can call directly.
pub struct SensoryCortex;

impl SensoryCortex {
    /// Calls the local Python Microservice (FastAPI) to browse the web.
    /// This is a BLOCKING call, designed to run inside the Rayon thread pool
    /// managed by 'main.rs'.
    pub fn perceive(url: &str, query: &str) -> Option<String> {
        // Log the attempt
        println!("[SENSORY] Contacting Cortex (Python 3.11) for target: {}", url);

        let client = reqwest::blocking::Client::new();
        
        // We set a 60s timeout because crawling a real webpage takes time.
        // We use '127.0.0.1:8000' because that's where uvicorn is running.
        let response = client.post("http://127.0.0.1:8000/perceive")
            .json(&CortexRequest {
                url: url.to_string(),
                query: query.to_string(),
            })
            .timeout(Duration::from_secs(60)) 
            .send();

        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    // Success: Parse the JSON knowledge
                    if let Ok(json) = resp.json::<CortexResponse>() {
                        println!("[SENSORY] Success! Received {} chars of knowledge.", json.knowledge.len());
                        return Some(json.knowledge);
                    } else {
                        println!("[ERROR] Sensory Cortex response was not valid JSON.");
                    }
                } else {
                    println!("[ERROR] Sensory Cortex returned error status: {}", resp.status());
                }
                None
            },
            Err(e) => {
                // This is the most common error (Python server not running)
                println!("\n[CRITICAL ERROR] Sensory Cortex Unreachable!");
                println!("1. Is the 'oraculum_sensory_agent' terminal open?");
                println!("2. Did you run 'python -m uvicorn oraculum_server:app --reload'?");
                println!("Details: {}\n", e);
                None
            }
        }
    }
}