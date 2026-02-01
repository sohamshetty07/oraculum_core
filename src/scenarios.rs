// src/scenarios.rs
// ADVANCED MARKETING SIMULATION ENGINE
// Implements Chain-of-Thought (CoT) & Phi-3 Chat Formats
// UPGRADE: Contextual Tasking, Dynamic Personality, Memory Graph Injection & Robust Parsing

use crate::agent_swarm::Agent;

/// The Core Trait for all simulation types.
/// Must be Sync + Send for parallel execution on M4 chips.
pub trait Scenario: Sync + Send {
    fn name(&self) -> &str;
    fn scenario_key(&self) -> &str;
    fn generate_prompt(&self, agent: &Agent, custom_context: Option<&str>) -> String;
    
    // NEW: Robust Response Parser (Default Implementation)
    // Extracts [Verdict] and [Thinking] tags to keep CSVs clean.
    fn process_response(&self, raw: &str) -> (String, Option<String>) {
        let raw_clean = raw.trim();
        
        if let Some(verdict_idx) = raw_clean.find("[Verdict]") {
            // Extract the final spoken verdict
            let verdict = raw_clean[verdict_idx + 9..].trim().to_string();
            
            // Extract the hidden thought process
            let thought = if let Some(think_idx) = raw_clean.find("[Thinking]") {
                let end_think = raw_clean.find("[Action]").unwrap_or(verdict_idx);
                Some(raw_clean[think_idx + 10..end_think].trim().to_string())
            } else {
                None
            };
            
            return (verdict, thought);
        } else if let Some(action_idx) = raw_clean.find("[Action]") {
            // For CX Flow scenarios that use [Action] instead of [Verdict]
            let action = raw_clean[action_idx + 8..].trim().to_string();
            let thought = if let Some(think_idx) = raw_clean.find("[Thinking]") {
                Some(raw_clean[think_idx + 10..action_idx].trim().to_string())
            } else {
                None
            };
            return (action, thought);
        }
        
        // Fallback: Return whole string if tags are missing (Graceful Fail)
        (raw_clean.to_string(), None)
    }
}

// =========================================================================
// 1. PRODUCT LAUNCH SCENARIO
// Goal: Determine Purchase Intent & Price Sensitivity
// =========================================================================
pub struct ProductLaunchScenario {
    pub product_name: String,
    pub category: String,
    pub context_docs: Vec<String>, // Renamed from key_benefits to align with API
}

impl ProductLaunchScenario {
    pub fn new(product_name: &str, category: &str, context_docs: Vec<&str>) -> Self {
        Self {
            product_name: product_name.to_string(),
            category: category.to_string(),
            context_docs: context_docs.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl Scenario for ProductLaunchScenario {
    fn name(&self) -> &str { "Product Launch Test" }
    fn scenario_key(&self) -> &str { "product_launch" }

    fn generate_prompt(&self, agent: &Agent, _custom_context: Option<&str>) -> String {
        let full_context = self.context_docs.join("\n");
        
        // --- CONTEXTUAL TASKING LOGIC ---
        let (task_directive, tone_instruction) = match agent.skepticism_level.as_str() {
            "High" => (
                "TASK: Look for the catch. Why might this product fail? Be cynical regarding the price and claims.",
                "INSTRUCTION: Do NOT be polite. If you think it's marketing fluff, say it. Use short, blunt sentences."
            ),
            "Low" => (
                "TASK: Imagine how this improves your life. Get excited about the benefits.",
                "INSTRUCTION: Be enthusiastic. Focus on the positive vibes and how it fits your style."
            ),
            _ => ( 
                "TASK: Evaluate this product based on your specific lifestyle and budget.",
                "INSTRUCTION: Weigh the pros and cons logically. Be balanced but decisive."
            ),
        };

        format!(
            "<|user|>You are {}, a {}. \
            \nDEMOGRAPHICS: {}\
            \nPERSONALITY: You have a '{}' speaking style. Your skepticism level is '{}'.\
            \nBELIEFS: {}\n\n\
            \
            --- ROOM CONTEXT ---\n\
            Topic: {} (Product Launch)\n\
            \
            MARKET CONTEXT: You are shopping for {} and discover a new product: '{}'.\n\
            KEY BENEFITS PROMISING: {}\n\
            \
            {}\n\
            \
            TASK: {}\n\
            INSTRUCTIONS:\n\
            1. [Thinking]: First, analyze if this fits your budget and daily routine. BE CRITICAL. {}\n\
            2. [Verdict]: Give your final answer. Would you buy it? (Yes/No/Maybe) and why. Use your defined speaking style ('{}').\n\
            \
            MANDATORY RESPONSE FORMAT (Strictly follow this):\n\
            [Thinking]\n\
            I need to verify if the protein claim is real. 80 rupees is okay, but...\n\
            [Verdict]\n\
            I might buy it once to try, but only if it's on Zepto.\n\
            \n\
            Response:<|end|>\n<|assistant|>",
            agent.name, agent.role, 
            agent.demographic, 
            agent.speaking_style, agent.skepticism_level,
            agent.beliefs.join(", "),
            self.product_name, // Memory Injection Key
            self.category, self.product_name, 
            full_context, // Injected Specs/Reddit
            full_context, // Fallback injection
            task_directive,
            tone_instruction,
            agent.speaking_style
        )
    }
}

// =========================================================================
// 2. CREATIVE A/B TEST SCENARIO
// Goal: Determine Emotional Resonance & Clarity of Ad Copy
// =========================================================================
pub struct CreativeTestScenario {
    pub tagline_a: String, // Kept for compatibility but usually parsed from product_context
    pub product_context: String,
    pub campaign_goal: String,
}

impl CreativeTestScenario {
    pub fn new(product_name: &str, context: &str, goal: &str) -> Self {
        Self {
            tagline_a: product_name.to_string(), // In this flow, we pass name as Tagline A usually
            product_context: context.to_string(),
            campaign_goal: goal.to_string(),
        }
    }
}

impl Scenario for CreativeTestScenario {
    fn name(&self) -> &str { "Creative Testing" }
    fn scenario_key(&self) -> &str { "creative_test" }

    fn generate_prompt(&self, agent: &Agent, _custom_context: Option<&str>) -> String {
        format!(
            "<|user|>You are {}, a {}. ({})\n\
            PERSONALITY: Style: '{}', Skepticism: '{}'.\n\n\
            --- ROOM CONTEXT ---\n\
            Topic: {} (Creative Test)\n\
            \
            We are A/B testing marketing copy for: {}.\n\
            \
            CONTEXT:\n{}\n\
            \
            TASK: Which headline/concept captures your attention more? Consider your personal beliefs: {}.\n\
            \
            INSTRUCTIONS:\n\
            1. [Thinking]: Compare the concept to your values. Which feels more authentic? Which feels 'salesy'?\n\
            2. [Verdict]: State clearly if this resonates (High/Medium/Low) and why. Speak in your natural voice.\n\
            \
            MANDATORY RESPONSE FORMAT:\n\
            [Thinking]\n\
            This concept sounds too corporate. It doesn't hit my pain point about time.\n\
            [Verdict]\n\
            Low resonance. It feels fake.\n\
            \n\
            Response:<|end|>\n<|assistant|>",
            agent.name, agent.role, agent.demographic,
            agent.speaking_style, agent.skepticism_level,
            self.campaign_goal, // Memory Injection
            self.campaign_goal,
            self.product_context,
            agent.beliefs.join(", ")
        )
    }
}

// =========================================================================
// 3. CUSTOMER JOURNEY (CX) FLOW SCENARIO
// Goal: Identify friction points at specific funnel stages
// =========================================================================
pub struct CXFlowScenario {
    pub stage: String, 
    pub product_info: String,
}

impl CXFlowScenario {
    pub fn new(stage: &str, product_info: &str) -> Self {
        Self {
            stage: stage.to_string(),
            product_info: product_info.to_string(),
        }
    }
}

impl Scenario for CXFlowScenario {
    fn name(&self) -> &str { "Customer Journey Flow" }
    fn scenario_key(&self) -> &str { "cx_flow" }

    fn generate_prompt(&self, agent: &Agent, _custom_context: Option<&str>) -> String {
        // Dynamic prompt based on funnel stage
        let stage_context = match self.stage.as_str() {
            "awareness" => "You are scrolling through Instagram/YouTube and see an ad. You have 3 seconds of attention span.",
            "consideration" => "You are interested and looking at the product details/ingredients list on a Quick Commerce app.",
            "purchase" => "You are at the checkout screen looking at the final price including delivery fees.",
            "advocacy" => "You have just eaten/used the product. A friend asks you if it was good.",
            _ => "You encounter this product in your daily life.",
        };

        format!(
            "<|user|>You are {}, a {}. ({})\n\
            PERSONALITY: Style: '{}', Skepticism: '{}'.\n\n\
            --- ROOM CONTEXT ---\n\
            Topic: {} (CX Flow)\n\
            \
            SCENARIO: {}\n\
            PRODUCT CONTEXT: {}\n\
            \
            TASK: Be honest about your behavior. Do you click? Do you abandon cart? Do you recommend it?\n\
            \
            INSTRUCTIONS:\n\
            1. [Thinking]: specific to your spending profile ({}), how do you react?\n\
            2. [Action]: What do you actually do next? Use your defined speaking style.\n\
            \
            MANDATORY RESPONSE FORMAT:\n\
            [Thinking]\n\
            The delivery fee is too high. I can buy this locally.\n\
            [Action]\n\
            I'm closing the app. Not worth it today.\n\
            \n\
            Response:<|end|>\n<|assistant|>",
            agent.name, agent.role, agent.demographic,
            agent.speaking_style, agent.skepticism_level,
            self.stage, // Memory Injection
            stage_context,
            self.product_info,
            agent.spending_profile
        )
    }
}

// =========================================================================
// 4. A/B MESSAGING STRATEGY SCENARIO
// Goal: Test Value Propositions (Rational vs Emotional vs Social)
// =========================================================================
pub struct ABMessagingScenario {
    pub product_name: String,
    pub context: String,
    pub strategy_name: String, 
}

impl ABMessagingScenario {
    pub fn new(product_name: &str, context: &str, strategy_name: &str) -> Self {
        Self {
            product_name: product_name.to_string(),
            context: context.to_string(),
            strategy_name: strategy_name.to_string(),
        }
    }
}

impl Scenario for ABMessagingScenario {
    fn name(&self) -> &str { "A/B Messaging Strategy" }
    fn scenario_key(&self) -> &str { "ab_messaging" }

    fn generate_prompt(&self, agent: &Agent, _custom_context: Option<&str>) -> String {
        format!(
            "<|user|>You are {}, a {}. ({})\n\
            PERSONALITY: Style: '{}', Skepticism: '{}'.\n\n\
            --- ROOM CONTEXT ---\n\
            Topic: {} (Messaging)\n\
            \
            Brand: {}\n\
            We are testing the positioning strategy: '{}'.\n\
            \
            CONTEXT:\n{}\n\
            \
            TASK: Does this messaging convince you to switch from your current brand?\n\
            \
            INSTRUCTIONS:\n\
            1. [Thinking]: Evaluate which benefit matters more to you personally (e.g. price vs health vs status).\n\
            2. [Verdict]: Rate persuasiveness (1-10) and explain why. Use your voice.\n\
            \
            MANDATORY RESPONSE FORMAT:\n\
            [Thinking]\n\
            This appeals to my need for status, but ignores price.\n\
            [Verdict]\n\
            7/10. It sounds premium but expensive.\n\
            \n\
            Response:<|end|>\n<|assistant|>",
            agent.name, agent.role, agent.demographic,
            agent.speaking_style, agent.skepticism_level,
            self.product_name, // Memory Injection
            self.product_name,
            self.strategy_name,
            self.context
        )
    }
}

// =========================================================================
// FACTORY FUNCTIONS
// Used by CLI and API to construct scenarios dynamically
// =========================================================================

pub fn get_scenario(scenario_type: &str) -> Option<Box<dyn Scenario>> {
    match scenario_type {
        "product_launch" => Some(Box::new(ProductLaunchScenario::new(
            "Mondelez Plant-Based Bites",
            "Healthy Snacking",
            vec!["High protein", "No artificial flavors", "10-min Blinkit delivery"],
        ))),
        "creative_test" => Some(Box::new(CreativeTestScenario::new(
            "Snack Smart, Live Better",
            "Target Audience: Gen Z",
            "Launch Campaign",
        ))),
        "cx_flow" => Some(Box::new(CXFlowScenario::new(
            "awareness",
            "New premium snacking line launched on Zepto",
        ))),
        "ab_messaging" => Some(Box::new(ABMessagingScenario::new(
            "Mondelez Bites",
            "Affordable vs Premium Positioning",
            "Value Strategy",
        ))),
        _ => None,
    }
}

pub fn list_available_scenarios() -> Vec<&'static str> {
    vec![
        "product_launch",
        "creative_test",
        "cx_flow",
        "ab_messaging",
    ]
}