// src/skills.rs
// COGNITIVE ARCHITECTURE LAYER 1: THE SKILL REGISTRY
// Defines the capabilities an agent can "equip".
// UPDATED: Added WebScout (Sensory Cortex Integration).

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::brain::AgentBrain;
use crate::systems::sensory::SensoryCortex;

// 1. The Standard Input/Output for any Skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInput {
    pub query: String,
    pub context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillOutput {
    pub success: bool,
    pub data: String,
    pub metadata: String, // JSON string for citations/sources
}

// 2. The Skill Trait
pub trait AgentSkill: Send + Sync {
    fn name(&self) -> String;
    fn description(&self) -> String;
    fn execute(&self, brain: &Arc<AgentBrain>, input: SkillInput) -> SkillOutput;
}

// 3. The Registry (Singleton)
pub struct SkillRegistry {
    skills: HashMap<String, Box<dyn AgentSkill>>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            skills: HashMap::new(),
        };
        
        // Register Core Skills
        registry.register(Box::new(DeepResearchSkill));
        registry.register(Box::new(FactCheckSkill));
        
        // NEW: Register the Autonomous Web Agent
        registry.register(Box::new(WebScout)); 
        
        registry
    }

    pub fn register(&mut self, skill: Box<dyn AgentSkill>) {
        self.skills.insert(skill.name(), skill);
    }

    pub fn get(&self, name: &str) -> Option<&Box<dyn AgentSkill>> {
        self.skills.get(name)
    }
    
    pub fn list_available(&self) -> Vec<String> {
        self.skills.keys().cloned().collect()
    }
}

// --- CORE SKILL IMPLEMENTATIONS ---

// Skill 1: Deep Research (Hybrid Memory System)
struct DeepResearchSkill;
impl AgentSkill for DeepResearchSkill {
    fn name(&self) -> String { "deep_research".to_string() }
    fn description(&self) -> String { "Queries Cognitive Memory (Reddit/Graph) and fallback Web Search".to_string() }
    
    fn execute(&self, brain: &Arc<AgentBrain>, input: SkillInput) -> SkillOutput {
        let results = brain.query_memory(&input.query);
        
        if results.is_empty() {
            SkillOutput { success: false, data: "No data found.".to_string(), metadata: "{}".to_string() }
        } else {
            let combined_data = results.join("\n\n");
            SkillOutput { 
                success: true, 
                data: combined_data, 
                metadata: format!("{{\"source\": \"HybridMemory\", \"hits\": {}}}", results.len()) 
            }
        }
    }
}

// Skill 2: Fact Check (Product Specs)
struct FactCheckSkill;
impl AgentSkill for FactCheckSkill {
    fn name(&self) -> String { "fact_check".to_string() }
    fn description(&self) -> String { "Verifies product specs via OpenFoodFacts".to_string() }
    
    fn execute(&self, brain: &Arc<AgentBrain>, input: SkillInput) -> SkillOutput {
        let facts = brain.get_facts(&input.query);
        
        if facts.is_empty() || facts.contains("No structured data") {
             SkillOutput { 
                success: false, 
                data: "Facts unavailable".to_string(), 
                metadata: "{}".to_string() 
            }
        } else {
            SkillOutput { 
                success: true, 
                data: facts, 
                metadata: "{\"source\": \"OpenFoodFacts\"}".to_string() 
            }
        }
    }
}

// Skill 3: Web Scout (Sensory Cortex / Crawl4AI)
// Connects to: crate::systems::sensory -> Python API (Port 8000)
struct WebScout;
impl AgentSkill for WebScout {
    fn name(&self) -> String { "web_scout".to_string() }
    fn description(&self) -> String { "Autonomous Web Agent (Crawl4AI + Qwen) that browses live sites".to_string() }

    fn execute(&self, _brain: &Arc<AgentBrain>, input: SkillInput) -> SkillOutput {
        // Identify Target
        // For this scenario, we default to the shop, but you could parse input.query for a URL
        let target_url = "https://scrapeme.live/shop";

        println!("[SKILL] WebScout engaged. Target: {}", target_url);

        // Call the Sensory Cortex (Python)
        if let Some(knowledge) = SensoryCortex::perceive(target_url, &input.query) {
             SkillOutput {
                success: true,
                data: knowledge,
                metadata: "{\"source\": \"SensoryCortex/Crawl4AI\"}".to_string()
            }
        } else {
            SkillOutput {
                success: false,
                data: "Sensory Cortex failed to retrieve data.".to_string(),
                metadata: "{}".to_string()
            }
        }
    }
}