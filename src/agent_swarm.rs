// src/agent_swarm.rs
// Agent Swarm Engine - Headless Marketing Intelligence

use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::memory::MemoryStream;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Agent {
    pub id: u32,
    pub name: String,
    pub role: String,
    pub demographic: String,
    pub beliefs: Vec<String>,
    pub spending_profile: String,
    pub product_affinity: Vec<String>,
    pub messaging_resonance: Vec<String>,
    
    // --- Voice & Personality Engine ---
    pub speaking_style: String,   // e.g. "Casual", "Analytical", "Rant"
    pub skepticism_level: String, // e.g. "High", "Medium", "Low"
    
    // --- NEW: Cognitive Skills ---
    // List of Skill IDs this agent can access (e.g., ["deep_research", "fact_check"])
    pub skills: Vec<String>, 
    // -----------------------------

    pub simulated_responses: u32,
    pub avg_sentiment: f32,

    // The Memory Stream (The "Mind")
    #[serde(skip)] 
    pub memory: Arc<Mutex<MemoryStream>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SimulationResult {
    pub agent_id: u32,
    // Explicit Name field for Blackboard Architecture
    pub agent_name: Option<String>,
    pub agent_role: String,
    pub agent_demographic: String,
    pub scenario: String,
    pub timestamp: String,
    pub prompt: String,
    pub response: String,
    
    // Stores the hidden [Thinking] block
    pub thought_process: Option<String>, 
    
    // --- NEW: Source Attribution ---
    // Stores the "Acquired Knowledge" (e.g., "Found 5 Reddit posts...")
    // This enables the "Glass Box" UI where users see the evidence.
    pub sources: Option<String>,
    // -------------------------------
    
    pub sentiment: String,
    // Changed to Option to support flexible categories
    pub category: Option<String>,
}

impl Agent {
    // This static constructor is a FALLBACK only. 
    // In the active simulation, agents are created dynamically by 'PersonaGenerator'
    // using LLM tokens and live data, completely bypassing these hardcoded values.
    pub fn new(id: u32, role: &str) -> Self {
        let names = match role {
            "Trader" => vec!["Priya", "Amit", "Neha", "Rajesh", "Deepika"],
            "Guard" => vec!["Vikram", "Anjali", "Arjun", "Pooja", "Sanjay"],
            "Miner" => vec!["Rohan", "Kavya", "Akash", "Sneha", "Nitin"],
            "Hacker" => vec!["Dev", "Zara", "Aditya", "Shruti", "Harsh"],
            "Drone" => vec!["Alex", "Maya", "Karan", "Isha", "Ravi"],
            _ => vec!["Agent"],
        };

        let name = names[(id as usize) % names.len()].to_string();

        let demographics = match role {
            "Trader" => "28F, Mumbai, middle class aspirant, bulk buys on Blinkit",
            "Guard" => "35M, Delhi, quality-conscious, premium brands",
            "Miner" => "24M, Bangalore, budget-aware, tech-savvy",
            "Hacker" => "26F, Pune, innovation-focused, early adopter",
            "Drone" => "31M, Hyderabad, convenience-driven, quick-commerce user",
            _ => "Unknown demographic",
        };

        let beliefs = match role {
            "Trader" => vec![
                "I buy for my joint family (6 people)".to_string(),
                "Health certifications matter (FSSAI, organic)".to_string(),
                "I need 10-minute delivery from Blinkit".to_string(),
                "I prefer Indian brands over MNCs".to_string(),
            ],
            "Guard" => vec![
                "Quality is non-negotiable, even at ₹500+ price".to_string(),
                "I trust heritage brands: Britannia, Amul, ITC".to_string(),
                "I check ingredients for artificial colors/flavors".to_string(),
                "Premium packaging signals quality".to_string(),
            ],
            "Miner" => vec![
                "I compare Blinkit vs Zepto prices before ordering".to_string(),
                "Bulk packs save money for monthly stock".to_string(),
                "₹10-20 difference matters for repeat purchases".to_string(),
                "I use cashback apps (Paytm, PhonePe)".to_string(),
            ],
            "Hacker" => vec![
                "I seek 'Made in India' and sustainable packaging".to_string(),
                "I influence my friend circle (200+ Instagram followers)".to_string(),
                "New launches from startups excite me".to_string(),
                "I avoid palm oil and high-sugar products".to_string(),
            ],
            "Drone" => vec![
                "10-minute Zepto delivery is my default".to_string(),
                "I order snacks 5x/week (₹2000/month budget)".to_string(),
                "Subscription benefits matter (Prime, Zepto Pass)".to_string(),
                "I value consistent quality over experiments".to_string(),
            ],
            _ => vec!["I am an agent".to_string()],
        };

        let product_affinity = match role {
            "Trader" => vec!["healthy snacks", "premium brands", "convenience"],
            "Guard" => vec!["quality verified", "established brands", "sustainability"],
            "Miner" => vec!["budget options", "bulk packs", "discounts"],
            "Hacker" => vec!["innovative products", "new trends", "tech integration"],
            "Drone" => vec!["instant delivery", "ready-to-eat", "subscriptions"],
            _ => vec!["general products"],
        }
        .iter()
        .map(|s| s.to_string())
        .collect();

        let messaging_resonance = match role {
            "Trader" => vec!["affordability with quality", "time-saving", "family value"],
            "Guard" => vec!["premium positioning", "heritage/trust", "certifications"],
            "Miner" => vec!["best price guarantee", "bulk savings", "loyalty rewards"],
            "Hacker" => vec!["first-to-market", "sustainability", "innovation story"],
            "Drone" => vec!["10-min delivery", "subscription benefits", "tracking"],
            _ => vec!["general messaging"],
        }
        .iter()
        .map(|s| s.to_string())
        .collect();

        Self {
            id,
            name,
            role: role.to_string(),
            demographic: demographics.to_string(),
            beliefs,
            spending_profile: format!("{}-typical", role.to_lowercase()),
            product_affinity,
            messaging_resonance,
            
            // --- Default Initialization for Fallback Agents ---
            speaking_style: "Neutral".to_string(),
            skepticism_level: "Medium".to_string(),
            
            // --- NEW: Initialize Default Skills ---
            skills: vec!["deep_research".to_string(), "fact_check".to_string()],
            // -------------------------------------

            simulated_responses: 0,
            avg_sentiment: 0.5,
            memory: Arc::new(Mutex::new(MemoryStream::new())), 
        }
    }

    pub fn update_sentiment(&mut self, sentiment_score: f32, response_count: u32) {
        self.simulated_responses = response_count;
        self.avg_sentiment = sentiment_score;
    }

    // Helper to check if agent has a skill
    pub fn has_skill(&self, skill_id: &str) -> bool {
        self.skills.contains(&skill_id.to_string())
    }
}

pub struct AgentSwarm {
    pub agents: Arc<Mutex<Vec<Agent>>>,
    pub results: Arc<Mutex<Vec<SimulationResult>>>,
}

impl AgentSwarm {
    pub fn new(agent_count: usize) -> Self {
        let roles = vec!["Trader", "Guard", "Miner", "Hacker", "Drone"];
        let mut agents = Vec::with_capacity(agent_count);

        for i in 0..agent_count {
            let role = roles[i % roles.len()];
            agents.push(Agent::new(i as u32, role));
        }

        Self {
            agents: Arc::new(Mutex::new(agents)),
            results: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn add_result(&self, result: SimulationResult) {
        if let Ok(mut results) = self.results.lock() {
            results.push(result);
        }
    }

    pub fn get_agents(&self) -> Vec<Agent> {
        self.agents
            .lock()
            .map(|agents| agents.clone())
            .unwrap_or_default()
    }

    pub fn get_results(&self) -> Vec<SimulationResult> {
        self.results
            .lock()
            .map(|results| results.clone())
            .unwrap_or_default()
    }

    pub fn get_timestamp() -> String {
        let now: DateTime<Utc> = Utc::now();
        now.format("%Y-%m-%dT%H:%M:%SZ").to_string()
    }

    pub fn sentiment_from_response(response: &str) -> String {
        let response_lower = response.to_lowercase();
        if response_lower.contains("love") || response_lower.contains("great")
            || response_lower.contains("amazing") || response_lower.contains("perfect")
            || response_lower.contains("excellent") || response_lower.contains("definitely")
        {
            "positive".to_string()
        } else if response_lower.contains("bad") || response_lower.contains("terrible")
            || response_lower.contains("hate") || response_lower.contains("awful")
            || response_lower.contains("dislike")
        {
            "negative".to_string()
        } else if response_lower.contains("maybe") || response_lower.contains("could")
            || response_lower.contains("depends") || response_lower.contains("interesting")
        {
            "neutral".to_string()
        } else {
            "mixed".to_string()
        }
    }

    pub fn extract_category(response: &str, scenario: &str) -> Option<String> {
        let category = match scenario {
            "product_launch" => {
                if response.contains("buy") || response.contains("purchase") {
                    "intent_to_buy"
                } else if response.contains("healthy") || response.contains("quality") {
                    "quality_focused"
                } else if response.contains("price") || response.contains("cost") {
                    "price_sensitive"
                } else {
                    "intrigued"
                }
            }
            "creative_test" => {
                if response.to_lowercase().contains("second") {
                    "option_b_preference"
                } else if response.to_lowercase().contains("first") {
                    "option_a_preference"
                } else {
                    "unclear_preference"
                }
            }
            "cx_flow" => {
                if response.contains("buy") || response.contains("cart") {
                    "converted"
                } else if response.contains("consider") || response.contains("check") {
                    "considering"
                } else {
                    "aware"
                }
            }
            "ab_messaging" => {
                if response.contains("affordable") || response.contains("value") {
                    "value_resonance"
                } else if response.contains("premium") || response.contains("indulgent") {
                    "premium_resonance"
                } else {
                    "neutral_resonance"
                }
            }
            "persona_generation" => "persona_data",
            _ => "general",
        };
        
        Some(category.to_string())
    }
}