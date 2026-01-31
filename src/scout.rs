// src/scout.rs
use std::sync::Arc;
use crate::brain::AgentBrain;

pub struct MarketScout;

impl MarketScout {
    /// DELEGATES the search to the Python Research Agent.
    /// This replaces the old "Wild Search" that used fragile scraping.
    /// Now, the Python Worker uses 'duckduckgo-search' and 'trafilatura' to read deep content.
    pub fn fetch_customer_voices(
        product: &str, 
        context: &str, 
        brain: &Arc<AgentBrain>
    ) -> Vec<String> {
        println!("🕵️ SCOUT: Commissioning Python Agent for Deep Research on '{}'...", product);
        
        // We call the 'research' method on the brain, which sends the {"task": "research"} JSON command.
        // The Python bridge handles the Searching, Reading, and Synthesizing.
        let voices = brain.research(product, context);
        
        // Handle Empty/Alert states
        if voices.is_empty() {
             println!("⚠️ SCOUT: The Research Agent returned empty-handed.");
             return vec!["System Alert: No specific internet voices found. Proceeding with synthetic data.".to_string()];
        } 
        
        if voices.len() == 1 && voices[0].contains("SYSTEM_ALERT") {
             println!("⚠️ SCOUT: {}", voices[0]);
             // We return empty so the PersonaGenerator knows to trigger its own fallback
             return Vec::new();
        }

        println!("✅ SCOUT: Agent returned {} verified wild voices.", voices.len());
        voices
    }

    /// Fetches live context by aggregating the Deep Research findings.
    /// We no longer do a separate "Price Check" scrape in Rust to avoid being blocked.
    /// The Python Research Agent captures pricing/value discussions naturally.
    pub fn fetch_live_context(product: &str, context: &str, brain: &Arc<AgentBrain>) -> String {
        // 1. Run the Deep Research Task
        let voices = Self::fetch_customer_voices(product, context, brain);
        
        // 2. Format the Report
        let mut report = String::from("\n\n--- 🌍 MARKET INTELLIGENCE (Deep Research) ---\n");
        
        if !voices.is_empty() {
            report.push_str("\n[REAL CONSUMER VOICES & ANALYSIS]:\n");
            for (i, v) in voices.iter().enumerate() {
                report.push_str(&format!("{}. \"{}\"\n", i+1, v));
            }
        } else {
             report.push_str("\n(No live data available. Using internal knowledge base.)\n");
        }

        report
    }
}