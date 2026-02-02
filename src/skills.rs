// src/skills.rs
// COGNITIVE ARCHITECTURE LAYER 1: THE SKILL REGISTRY
// Defines the capabilities an agent can "equip".
// UPDATED: DeepResearchSkill now uses the Hybrid Memory System (LanceDB + Live Web).

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::brain::AgentBrain;

// 1. The Standard Input/Output for any Skill
// This allows us to chain skills together (Output of A -> Input of B)
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
// Any tool (Scraper, Database, Logic) must implement this.
pub trait AgentSkill: Send + Sync {
    fn name(&self) -> String;
    fn description(&self) -> String;
    // We pass the 'Brain' so skills can use Python/LLM if needed
    fn execute(&self, brain: &Arc<AgentBrain>, input: SkillInput) -> SkillOutput;
}

// 3. The Registry (Singleton)
// Maps string IDs ("web_search") to actual Rust implementations
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
// Connects to: python_bridge 'query_memory' (LanceDB + DuckDuckGo)
struct DeepResearchSkill;
impl AgentSkill for DeepResearchSkill {
    fn name(&self) -> String { "deep_research".to_string() }
    fn description(&self) -> String { "Queries Cognitive Memory (Reddit/Graph) and fallback Web Search".to_string() }
    
    fn execute(&self, brain: &Arc<AgentBrain>, input: SkillInput) -> SkillOutput {
        // CALL THE NEW MEMORY SYSTEM (Phase 2 Upgrade)
        // This executes the 'query_memory' task in Python
        let results = brain.query_memory(&input.query);
        
        if results.is_empty() {
            SkillOutput { success: false, data: "No data found.".to_string(), metadata: "{}".to_string() }
        } else {
            // Join the memory hits into a single context block for the Agent to read
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
// Connects to: python_bridge 'get_facts' (OpenFoodFacts)
struct FactCheckSkill;
impl AgentSkill for FactCheckSkill {
    fn name(&self) -> String { "fact_check".to_string() }
    fn description(&self) -> String { "Verifies product specs via OpenFoodFacts".to_string() }
    
    fn execute(&self, brain: &Arc<AgentBrain>, input: SkillInput) -> SkillOutput {
        let facts = brain.get_facts(&input.query);
        
        // Basic validation of the result
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