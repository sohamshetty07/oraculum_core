// src/persona_generator.rs
// ROBUST VERSION: Doppelgänger Engine (Wild Mode) & Memory Enabled
// UPDATE: Added Duplicate Name Prevention

use crate::brain::AgentBrain;
use crate::agent_swarm::Agent;
use crate::memory::MemoryStream; 
use std::sync::{Arc, Mutex}; 
use serde_json::Value;
use std::collections::HashSet; // <--- NEW: For tracking names

pub struct PersonaGenerator;

impl PersonaGenerator {
    // NEW: The Core Generator that can use Wild Internet Data
    pub fn generate_from_voices(
        count: usize, 
        audience_criteria: &str, 
        real_voices: Vec<String>, // <--- The Wild Data
        brain: &Arc<AgentBrain>
    ) -> Vec<Agent> {
        
        let mut agents = Vec::new();
        let mut global_id_counter = 1; 
        
        // NEW: Track used names to prevent duplicates in the same simulation
        let mut used_names = HashSet::new();

        // Construct the "DNA" for the agents
        let voice_context = if !real_voices.is_empty() {
            format!("Here are REAL comments from the internet about this topic. \
            Use these to model the personalities, speaking styles, and skepticism levels of the agents. \
            Make the agents feel like the authors of these comments:\n\n{}", real_voices.join("\n---\n"))
        } else {
            "No internet voices found. Generate realistic personas based on the target group constraints.".to_string()
        };

        println!("✨ GENERATOR: Synthesizing {} unique personas for target: '{}' (Wild Mode: {})...", 
            count, audience_criteria, !real_voices.is_empty());

        let batch_size = 5;
        let batches = (count as f32 / batch_size as f32).ceil() as usize;

        for _ in 0..batches {
            if agents.len() >= count {
                break;
            }

            // UPDATED PROMPT: Forces Voice & Skepticism based on real data
            let prompt = format!(
                "<|user|>Task: Generate a JSON array of 5 realistic Indian consumer personas matching: '{}'. \
                \n\nSOURCE MATERIAL (Model personalities on this):\n{}\n\n\
                CONSTRAINTS:\n\
                1. DIVERSITY: If source material has angry people, make angry agents. If fans, make fans.\n\
                2. Names: Culturally accurate to Region/Gender defined in target.\n\
                3. SPEAKING STYLE: Capture the vibe (e.g., 'Rant', 'Analytical', 'Short', 'Excited').\n\
                4. SKEPTICISM: Must vary ('High', 'Medium', 'Low').\n\
                \n\
                Format: [{{ \"name\": \"...\", \"age\": 20, \"city\": \"...\", \"occupation\": \"...\", \"spending_behavior\": \"...\", \"cultural_values\": \"...\", \"speaking_style\": \"...\", \"skepticism_level\": \"...\" }}] \
                \n\
                Return ONLY JSON. No text.<|end|>\n<|assistant|>",
                audience_criteria, voice_context
            );

            // Ask Brain (Python)
            // Increased token limit slightly to handle the richer JSON
            let response_text = brain.generate(&prompt, 1200, None, None); 
            
            // AGGRESSIVE CLEANUP
            let clean_json = clean_json_text(&response_text);
            
            if let Ok(parsed) = serde_json::from_str::<Value>(&clean_json) {
                if let Some(array) = parsed.as_array() {
                    for item in array {
                        if agents.len() >= count {
                            break;
                        }

                        let raw_name = item["name"].as_str().unwrap_or("Agent").to_string();
                        
                        // --- FIX: Ensure unique names. If duplicate, append ID. ---
                        let name = if used_names.contains(&raw_name) {
                            format!("{} ({})", raw_name, global_id_counter)
                        } else {
                            used_names.insert(raw_name.clone());
                            raw_name
                        };
                        // ----------------------------------------------------------

                        let id = global_id_counter;
                        global_id_counter += 1;
                        
                        let role = item["occupation"].as_str().unwrap_or("Consumer").to_string();
                        let city = item["city"].as_str().unwrap_or("India").to_string();
                        let age = item["age"].as_u64().unwrap_or(25);
                        let spending = item["spending_behavior"].as_str().unwrap_or("Moderate").to_string();
                        let culture = item["cultural_values"].as_str().unwrap_or("Traditional").to_string();
                        
                        // NEW FIELDS
                        let style = item["speaking_style"].as_str().unwrap_or("Neutral").to_string();
                        let skepticism = item["skepticism_level"].as_str().unwrap_or("Medium").to_string();

                        // Construct the rich demographic string
                        let full_demographic = format!("{}, {}y/o, {}, {}", city, age, role, spending);
                        
                        let agent = Agent {
                            id,
                            name: name.clone(),
                            role: role.clone(),
                            demographic: full_demographic,
                            beliefs: vec![culture, spending.clone()],
                            spending_profile: spending,
                            product_affinity: vec!["Consumer Goods".to_string()],
                            messaging_resonance: vec![],
                            // Assign generated personality
                            speaking_style: style.clone(),
                            skepticism_level: skepticism.clone(),
                            simulated_responses: 0,
                            avg_sentiment: 0.5,
                            memory: Arc::new(Mutex::new(MemoryStream::new())), 
                        };
                        
                        println!("   └── Created: {} ({}) [Style: {} | Skepticism: {}]", name, role, style, skepticism);
                        agents.push(agent);
                    }
                }
            } else {
                println!("   ⚠️ LLM Error. Using Fallback.");
                println!("   🔴 RAW OUTPUT: {:?}", response_text); 
                
                // Fallback logic
                let remaining_needed = count - agents.len();
                if remaining_needed > 0 {
                    let backup = get_fallback_agents(global_id_counter, remaining_needed, audience_criteria);
                    global_id_counter += backup.len() as u32;
                    agents.extend(backup);
                }
            }
        }
        
        agents
    }

    // WRAPPER: Keeps the old function signature so other code doesn't break
    pub fn generate_batch(count: usize, criteria: &str, brain: &Arc<AgentBrain>) -> Vec<Agent> {
        // Call the new engine with empty voices (Standard Mode)
        Self::generate_from_voices(count, criteria, Vec::new(), brain)
    }
}

// Robust JSON Extractor
fn clean_json_text(text: &str) -> String {
    let start = text.find('[').unwrap_or(0);
    let end = text.rfind(']').map(|i| i + 1).unwrap_or(text.len());
    
    if start < end {
        let slice = &text[start..end];
        slice.replace("```json", "").replace("```", "").trim().to_string()
    } else {
        text.to_string()
    }
}

// Updated fallback to respect 1-based indexing and needed count
fn get_fallback_agents(start_id: u32, needed: usize, criteria: &str) -> Vec<Agent> {
    let mut fallbacks = Vec::new();
    for i in 0..needed {
        let id = start_id + i as u32;
        fallbacks.push(Agent {
            id,
            name: format!("Agent {} (Fallback)", id),
            role: "General Consumer".to_string(),
            demographic: format!("Target: {}", criteria),
            beliefs: vec![], 
            spending_profile: "Moderate".to_string(), 
            product_affinity: vec![], 
            messaging_resonance: vec![], 
            // NEW Defaults
            speaking_style: "Neutral".to_string(),
            skepticism_level: "Medium".to_string(),
            simulated_responses: 0, 
            avg_sentiment: 0.5,
            memory: Arc::new(Mutex::new(MemoryStream::new())),
        });
    }
    fallbacks
}