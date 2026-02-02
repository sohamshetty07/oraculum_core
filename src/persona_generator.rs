// src/persona_generator.rs
// DYNAMIC VERSION: Context-Sharded Doppelgänger Engine + SKILLS INTEGRATION
// FEATURES: 
// 1. Shards 'Wild Voices' so agents have unique backstories.
// 2. Uses 'Diversity Seeds' to force LLM variance (preventing name collision).
// 3. Assigns Skills dynamically based on Persona Role.
// 4. Truncates context to prevent Pipe Deadlocks.

use crate::brain::AgentBrain;
use crate::agent_swarm::Agent;
use crate::memory::MemoryStream; 
use std::sync::{Arc, Mutex}; 
use serde_json::Value;
use std::collections::HashSet;

pub struct PersonaGenerator;

impl PersonaGenerator {
    pub fn generate_from_voices(
        count: usize, 
        audience_criteria: &str, 
        real_voices: Vec<String>, 
        brain: &Arc<AgentBrain>
    ) -> Vec<Agent> {
        
        let mut agents = Vec::new();
        let mut global_id_counter = 1; 
        let mut used_names = HashSet::new();

        println!("✨ GENERATOR: Synthesizing {} unique personas for target: '{}'...", count, audience_criteria);

        // CONFIGURATION
        let batch_size = 5; // We generate 5 agents at a time to keep LLM context stable
        let batches = (count as f32 / batch_size as f32).ceil() as usize;

        for batch_idx in 0..batches {
            if agents.len() >= count { break; }

            // --- STRATEGY 1: VOICE SHARDING ---
            // Don't give every agent every comment. Slice the data.
            // Batch 0 gets comments 0-4, Batch 1 gets 5-9, etc.
            let start_voice = (batch_idx * 3) % real_voices.len().max(1);
            let end_voice = ((batch_idx + 1) * 3).min(real_voices.len());
            
            let voice_subset = if !real_voices.is_empty() && start_voice < real_voices.len() {
                real_voices[start_voice..end_voice].to_vec()
            } else {
                Vec::new()
            };

            // --- TRUNCATION LOGIC (Prevent Pipe Deadlock) ---
            let mut voice_context = if !voice_subset.is_empty() {
                format!("SOURCE MATERIAL (Base personalities on these SPECIFIC real comments):\n{}\n\n", voice_subset.join("\n---\n"))
            } else {
                String::new()
            };
            
            // Limit context size to ~2000 chars to ensure it fits in the standard pipe buffer
            if voice_context.len() > 2000 {
                voice_context.truncate(2000);
                voice_context.push_str("\n...(truncated)...");
            }

            // --- STRATEGY 2: DIVERSITY SEEDS ---
            // If we don't guide the LLM, it defaults to the most probable average person.
            // We force it to look at different corners of the room for each batch.
            let archetype_instruction = match batch_idx % 4 {
                0 => "FOCUS: Early Adopters & Optimists. Use modern, urban names.",
                1 => "FOCUS: Skeptics & Budget-Conscious. Use traditional names.",
                2 => "FOCUS: Quality-Conscious & Brand Loyalists. Use specific regional names (e.g. South Indian, Bengali).",
                _ => "FOCUS: Critics & Detractors. Use diverse names."
            };

            // --- THE PROMPT ---
            // UPDATED: Replaced negative constraints with positive regional instructions
            let prompt = format!(
                "<|user|>Task: Generate a JSON array of {} unique Indian consumer personas matching: '{}'.\n\n\
                {}\
                \n\
                DIVERSITY INSTRUCTION: {}\n\
                CRITICAL RULES:\n\
                1. USE DIVERSE REGIONAL NAMES: Pick names from South India, Bengal, Punjab, Maharashtra, etc. Avoid generic names like Aryan or Rohan.\n\
                2. VARY THE SKEPTICISM: Not everyone agrees.\n\
                3. REALISM: Use the Source Material to define their 'speaking_style'.\n\
                \n\
                Format: [{{ \"name\": \"...\", \"age\": 20, \"city\": \"...\", \"occupation\": \"...\", \"spending_behavior\": \"...\", \"cultural_values\": \"...\", \"speaking_style\": \"...\", \"skepticism_level\": \"...\" }}] \
                \n\
                Return ONLY JSON. No text.<|end|>\n<|assistant|>",
                batch_size, 
                audience_criteria, 
                voice_context, 
                archetype_instruction
            );

            // Call Python Brain with HIGH TEMPERATURE (0.8)
            // This forces diversity in names
            let response_text = brain.generate(&prompt, 1500, None, None, 0.8); 
            let clean_json = clean_json_text(&response_text);
            
            // Parse & Build
            if let Ok(parsed) = serde_json::from_str::<Value>(&clean_json) {
                if let Some(array) = parsed.as_array() {
                    for item in array {
                        if agents.len() >= count { break; }

                        let mut raw_name = item["name"].as_str().unwrap_or("Agent").to_string();
                        
                        // Fallback duplicate handler (just in case LLM ignores us)
                        if used_names.contains(&raw_name) {
                            raw_name = format!("{} {}", raw_name, global_id_counter);
                        }
                        used_names.insert(raw_name.clone());

                        let id = global_id_counter;
                        global_id_counter += 1;
                        
                        // Extract fields with defaults
                        let role = item["occupation"].as_str().unwrap_or("Consumer").to_string();
                        let city = item["city"].as_str().unwrap_or("Metro").to_string();
                        let age = item["age"].as_u64().unwrap_or(25);
                        let spending = item["spending_behavior"].as_str().unwrap_or("Moderate").to_string();
                        let culture = item["cultural_values"].as_str().unwrap_or("Traditional").to_string();
                        let style = item["speaking_style"].as_str().unwrap_or("Neutral").to_string();
                        let skepticism = item["skepticism_level"].as_str().unwrap_or("Medium").to_string();

                        // Rich demographic string for context injection later
                        let full_demographic = format!("{}, {}y/o, {}, {}", city, age, role, spending);
                        
                        let agent = Agent {
                            id,
                            name: raw_name.clone(),
                            role: role.clone(),
                            demographic: full_demographic,
                            beliefs: vec![culture, spending.clone()],
                            spending_profile: spending,
                            product_affinity: vec!["General".to_string()],
                            messaging_resonance: vec![],
                            speaking_style: style.clone(),
                            skepticism_level: skepticism.clone(),
                            
                            // --- NEW: Dynamic Skill Assignment based on Role ---
                            // If they are an Analyst, Engineer, or Journalist, they check facts.
                            // Everyone else just does deep research (social sentiment).
                            skills: if role.to_lowercase().contains("analyst") 
                                    || role.to_lowercase().contains("engineer")
                                    || role.to_lowercase().contains("journalist") {
                                vec!["deep_research".to_string(), "fact_check".to_string()]
                            } else {
                                vec!["deep_research".to_string()]
                            },
                            // --------------------------------------------------

                            simulated_responses: 0,
                            avg_sentiment: 0.5,
                            memory: Arc::new(Mutex::new(MemoryStream::new())), 
                        };
                        
                        println!("   └── Created: {} ({}) [Style: {}]", raw_name, role, style);
                        agents.push(agent);
                    }
                }
            } else {
                // Fallback if JSON breaks
                println!("   ⚠️ JSON Parse Error. Generating Fallback Agent.");
                let fallback = get_fallback_agents(global_id_counter, 1, audience_criteria);
                global_id_counter += 1;
                agents.extend(fallback);
            }
        }
        
        agents
    }

    pub fn generate_batch(count: usize, criteria: &str, brain: &Arc<AgentBrain>) -> Vec<Agent> {
        Self::generate_from_voices(count, criteria, Vec::new(), brain)
    }
}

// --- UTILS ---

fn clean_json_text(text: &str) -> String {
    let start = text.find('[').unwrap_or(0);
    let end = text.rfind(']').map(|i| i + 1).unwrap_or(text.len());
    if start < end {
        text[start..end].replace("```json", "").replace("```", "").trim().to_string()
    } else {
        text.to_string()
    }
}

fn get_fallback_agents(start_id: u32, needed: usize, criteria: &str) -> Vec<Agent> {
    let mut fallbacks = Vec::new();
    for i in 0..needed {
        let id = start_id + i as u32;
        fallbacks.push(Agent {
            id,
            name: format!("Participant {}", id),
            role: "General Consumer".to_string(),
            demographic: format!("Target: {}", criteria),
            beliefs: vec![], spending_profile: "Moderate".to_string(), 
            product_affinity: vec![], messaging_resonance: vec![], 
            speaking_style: "Neutral".to_string(), skepticism_level: "Medium".to_string(),
            // Default skills for fallback agents
            skills: vec!["deep_research".to_string()],
            simulated_responses: 0, avg_sentiment: 0.5,
            memory: Arc::new(Mutex::new(MemoryStream::new())),
        });
    }
    fallbacks
}