// src/analyst.rs
use std::sync::Arc;
use crate::brain::AgentBrain;
use crate::agent_swarm::SimulationResult;

pub struct AnalystEngine;

impl AnalystEngine {
    pub fn generate_report(
        brain: &Arc<AgentBrain>, 
        scenario_type: &str, 
        results: &[SimulationResult]
    ) -> String {
        // 1. Contextualize the Data
        let total = results.len();
        let positive = results.iter().filter(|r| r.sentiment == "positive").count();
        let sentiment_score = if total > 0 { (positive as f32 / total as f32) * 100.0 } else { 0.0 };

        // 2. Prepare the Transcript (Deep Read)
        // CRITICAL UPDATE: We now include the 'thought_process' (Internal Monologue)
        // This allows the Analyst to detect the gap between "What they said" vs "What they meant".
        let mut transcript = String::new();
        for r in results.iter().take(60) { 
            let thought_display = r.thought_process.as_deref().unwrap_or("No internal thought captured");
            
            transcript.push_str(&format!(
                "- Participant: {} ({})\n  HIDDEN THOUGHT: {}\n  PUBLIC VERDICT: \"{}\"\n\n", 
                r.agent_role, 
                r.agent_demographic, 
                thought_display, 
                r.response
            ));
        }

        // 3. Dynamic Prompting based on Scenario
        let (role_context, specific_questions) = match scenario_type {
            "product_launch" => (
                "Product Strategy Consultant",
                "1. **The 'Say-Do' Gap**: Did agents secretly dislike something but say they liked it? Check the Hidden Thoughts.\n2. **Price Sensitivity**: What were the internal reasonings regarding money?\n3. **Adoption Blocker**: What is the #1 psychological barrier to buying?"
            ),
            "creative_test" => (
                "Creative Director",
                "1. **Attention Hook**: Which specific words in the copy triggered an internal reaction?\n2. **Emotional Resonance**: Did they feel 'sold to' or 'understood'?\n3. **Winner**: Which option feels more authentic?"
            ),
            "ab_messaging" => (
                "Brand Strategist",
                "1. **Trust Analysis**: Which message generated less skeptical internal thoughts?\n2. **Clarity**: Was there confusion in the internal monologue?\n3. **Recommendation**: Which value prop is stronger?"
            ),
            "cx_flow" => (
                "UX Researcher",
                "1. **Friction Points**: Where did the internal monologue show frustration?\n2. **Impulse vs Logic**: Did they buy on impulse or calculation?\n3. **Fixes**: Top 3 UX improvements."
            ),
            _ => (
                "Data Analyst",
                "1. **Psychological Trends**: What are the common internal drivers?\n2. **Contradictions**: Highlight instances where thoughts contradicted words.\n3. **Verdict**: Final strategic recommendation."
            )
        };

        // 4. Construct the Final Prompt
        // We inject 'Topic: ... Analysis' to allow the Python Brain to potentially link this report 
        // to the graph nodes created during the simulation.
        let prompt = format!(
            "<|user|>You are an expert {}.\n\
            Analyze the following synthetic research data (N={} Participants, Sentiment: {:.1}% Positive).\n\n\
            --- ROOM CONTEXT ---\n\
            Topic: {} Analysis\n\n\
            --- RAW DATA (TRANSCRIPT WITH HIDDEN THOUGHTS) ---\n\
            {}\n\
            --- END DATA ---\n\n\
            TASK: Generate a Management Report in Markdown.\n\
            CRITICAL: Focus on the 'HIDDEN THOUGHTS' to find true consumer intent.\n\
            {}\n\
            \
            Output Format:\n\
            ## Executive Summary\n\
            ## The Psychological Profile (Deep Dive)\n\
            ## Key Findings\n\
            ## Strategic Recommendations\n\
            (Be concise, professional, and use bullet points)<|end|>\n<|assistant|>",
            role_context, total, sentiment_score, scenario_type, transcript, specific_questions
        );

        // 5. Generate
        println!("🧠 ANALYST: Generating report for scenario '{}'...", scenario_type);
        brain.generate(&prompt, 1500, None, None, 0.4)
    }
}