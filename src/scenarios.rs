// src/scenarios.rs
// ADVANCED MARKETING SIMULATION ENGINE
// Implements Chain-of-Thought (CoT) & Phi-3 Chat Formats
// UPGRADE: Contextual Tasking, Dynamic Personality & Memory Graph Injection

use crate::agent_swarm::Agent;

/// The Core Trait for all simulation types.
/// Must be Sync + Send for parallel execution on M4 chips.
pub trait Scenario: Sync + Send {
    fn name(&self) -> &str;
    fn generate_prompt(&self, agent: &Agent, custom_context: Option<&str>) -> String;
    fn scenario_key(&self) -> &str;
}

// =========================================================================
// 1. PRODUCT LAUNCH SCENARIO
// Goal: Determine Purchase Intent & Price Sensitivity
// =========================================================================
pub struct ProductLaunchScenario {
    pub product_name: String,
    pub category: String,
    pub key_benefits: Vec<String>,
}

impl ProductLaunchScenario {
    pub fn new(product_name: &str, category: &str, key_benefits: Vec<&str>) -> Self {
        Self {
            product_name: product_name.to_string(),
            category: category.to_string(),
            key_benefits: key_benefits.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl Scenario for ProductLaunchScenario {
    fn name(&self) -> &str { "Product Launch Test" }
    fn scenario_key(&self) -> &str { "product_launch" }

    fn generate_prompt(&self, agent: &Agent, _custom_context: Option<&str>) -> String {
        let benefits = self.key_benefits.join(", ");
        
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
            self.product_name, // <--- MEMORY INJECTION: Triggers Graph Retrieval
            self.category, self.product_name, benefits,
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
    pub tagline_a: String,
    pub tagline_b: String,
    pub product_context: String,
}

impl CreativeTestScenario {
    pub fn new(tagline_a: &str, tagline_b: &str, product_context: &str) -> Self {
        Self {
            tagline_a: tagline_a.to_string(),
            tagline_b: tagline_b.to_string(),
            product_context: product_context.to_string(),
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
            OPTION A: \"{}\"\n\
            OPTION B: \"{}\"\n\
            \
            TASK: Which headline captures your attention more? Consider your personal beliefs: {}.\n\
            \
            INSTRUCTIONS:\n\
            1. [Thinking]: Compare both options. Which feels more authentic to you? Which feels 'salesy'?\n\
            2. [Verdict]: State clearly which option wins and the main reason. Speak in your natural voice.\n\
            \
            MANDATORY RESPONSE FORMAT:\n\
            [Thinking]\n\
            Option A sounds too corporate. Option B hits my pain point about time.\n\
            [Verdict]\n\
            Option B wins. It feels more honest.\n\
            \n\
            Response:<|end|>\n<|assistant|>",
            agent.name, agent.role, agent.demographic,
            agent.speaking_style, agent.skepticism_level,
            self.product_context, // <--- MEMORY INJECTION
            self.product_context, self.tagline_a, self.tagline_b,
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
            PRODUCT: {}\n\
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
            self.product_info, // <--- MEMORY INJECTION
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
    pub value_prop_a: String,
    pub value_prop_b: String,
    pub product_name: String,
}

impl ABMessagingScenario {
    pub fn new(value_prop_a: &str, value_prop_b: &str, product_name: &str) -> Self {
        Self {
            value_prop_a: value_prop_a.to_string(),
            value_prop_b: value_prop_b.to_string(),
            product_name: product_name.to_string(),
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
            We are trying to decide how to position this product to people like you.\n\
            \
            STRATEGY A: {}\n\
            STRATEGY B: {}\n\
            \
            TASK: Which argument convinces you to switch from your current brand?\n\
            \
            INSTRUCTIONS:\n\
            1. [Thinking]: Evaluate which benefit matters more to you personally (e.g. price vs health vs status).\n\
            2. [Verdict]: Choose Strategy A or B and rate persuasiveness (1-10). Use your voice.\n\
            \
            MANDATORY RESPONSE FORMAT:\n\
            [Thinking]\n\
            Strategy A is boring. Strategy B appeals to my need for status.\n\
            [Verdict]\n\
            Strategy B is a 9/10. It sounds premium.\n\
            \n\
            Response:<|end|>\n<|assistant|>",
            agent.name, agent.role, agent.demographic,
            agent.speaking_style, agent.skepticism_level,
            self.product_name, // <--- MEMORY INJECTION
            self.product_name,
            self.value_prop_a,
            self.value_prop_b
        )
    }
}

// =========================================================================
// 5. PERSONA GENERATION (INTERNAL UTILITY)
// Goal: Self-reflection for debugging
// =========================================================================
pub struct PersonaGenerationScenario;

impl Scenario for PersonaGenerationScenario {
    fn name(&self) -> &str { "Persona Generation" }
    fn scenario_key(&self) -> &str { "persona_generation" }

    fn generate_prompt(&self, agent: &Agent, _custom_context: Option<&str>) -> String {
        format!(
            "<|user|>You are {}. Role: {}. Location: {}.\n\
            Please introduce yourself deeply. Talk about your daily routine, your financial stress levels, and your favorite places to shop.\n\
            Format: Natural first-person speech.<|end|>\n<|assistant|>",
            agent.name, agent.role, agent.demographic
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
            "Guilt-Free Indulgence, Maximum Flavor",
            "New Mondelez Premium Line",
        ))),
        "cx_flow" => Some(Box::new(CXFlowScenario::new(
            "awareness",
            "New premium snacking line launched on Zepto",
        ))),
        "ab_messaging" => Some(Box::new(ABMessagingScenario::new(
            "Affordable nutrition without compromise",
            "Indulgent treats for the conscious consumer",
            "MondeleezPlus",
        ))),
        "persona_generation" => Some(Box::new(PersonaGenerationScenario)),
        _ => None,
    }
}

pub fn list_available_scenarios() -> Vec<&'static str> {
    vec![
        "product_launch",
        "creative_test",
        "cx_flow",
        "ab_messaging",
        "persona_generation",
    ]
}