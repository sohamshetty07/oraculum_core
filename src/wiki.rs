// src/wiki.rs
// THE KNOWLEDGE CODEX
// Fetches factual ground truth to prevent hallucination.
// UPGRADE: Delegates to Python Fact Agent (Wiki + Web Fallback)

use std::sync::Arc;
use crate::brain::AgentBrain;

pub struct WikiScout;

impl WikiScout {
    /// Main Entry Point: Asks the Brain to find concrete specs.
    pub fn fetch_summary(query: &str, brain: &Arc<AgentBrain>) -> Option<String> {
        println!("📚 WIKI: Consulting the Codex (Fact Agent) for '{}'...", query);
        
        // Delegate to Python Fact Agent
        // This avoids the "Entity not found" issue by allowing fallback to general web search
        let fact_sheet = brain.get_facts(query);
        
        if fact_sheet.contains("SYSTEM_ALERT") || fact_sheet.trim().is_empty() {
            println!("   ⚠️ WIKI: No concrete facts found. Using generic fallback.");
            return None;
        }

        println!("   -> Fact Sheet Acquired ({} chars)", fact_sheet.len());
        Some(fact_sheet)
    }
}