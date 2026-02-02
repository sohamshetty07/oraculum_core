// src/focus_group.rs
// SOCIETY ENGINE V4.0: "Free-MAD" Blackboard Architecture
// Fixes: Metadata Loss (Undefined IDs) & Consensus Collapse (Echo Chambers)

use std::sync::Arc;
use tokio::sync::Mutex; 
use rayon::prelude::*;
use crate::brain::AgentBrain;
use crate::agent_swarm::{Agent, SimulationResult};
use chrono::Local;

pub struct FocusGroupSession;

impl FocusGroupSession {
    
    // --- THE BLACKBOARD ARCHITECTURE ---
    // We maintain a shared "Room History" string that evolves.
    // We do NOT ask the LLM to generate the speaker names/IDs. We force the identity from Rust.
    pub async fn run_debate(
        brain: &Arc<AgentBrain>,
        agents: &Vec<Agent>,
        product_context: &str,
        rounds: usize
    ) -> Vec<SimulationResult> {
        let mut results = Vec::new();
        // The Blackboard: Shared memory of the conversation
        let room_history = Arc::new(Mutex::new(String::from("--- DEBATE START ---\n")));

        println!("🗣️ MODE: Starting Multi-Agent Focus Group ({} Rounds)...", rounds);

        for round in 1..=rounds {
            println!("   ⟳ Running Round {}...", round);
            
            // --- "Free-MAD" CONFLICT INJECTION ---
            // Research suggests escalating conflict in middle rounds to prevent "polite consensus".
            let (stage_instruction, temp) = match round {
                1 => (
                    "PHASE 1: INITIAL REACTIONS. \
                    Give your raw, unfiltered first impression. Be honest but brief.", 
                    0.6 // Moderate creativity
                ),
                _ if round == rounds => (
                    "PHASE 3: FINAL VERDICT. \
                    Did the discussion change your mind? Give a final Yes/No decision.", 
                    0.5 // Stable
                ),
                _ => (
                    "PHASE 2: THE DEBATE (CONFLICT MODE). \
                    Review the ROOM HISTORY. \
                    If you disagree with a previous point, ATTACK it. \
                    If you are Skeptical, find flaws in the Optimists' logic. \
                    Do NOT be polite. We need critical analysis.", 
                    0.8 // High Entropy for conflict
                ), 
            };

            // 1. Snapshot the Blackboard (Read-Only access for this batch)
            let history_snapshot = room_history.lock().await.clone();

            // 2. Parallel Inference (Rayon)
            // We map existing agents -> results. 
            let round_results: Vec<SimulationResult> = agents.par_iter().map(|agent| {
                
                // Construct Prompt with Blackboard Context
                let prompt = format!(
                    "<|user|>You are participating in a focus group.\n\
                    --- YOUR IDENTITY ---\n\
                    Name: {}\n\
                    Role: {}\n\
                    Traits: {}\n\
                    \n\
                    --- ROOM HISTORY (What others have said) ---\n\
                    {}\n\
                    \n\
                    --- YOUR TURN ---\n\
                    Topic: {}\n\
                    Current Round: {}\n\
                    \n\
                    INSTRUCTION: {}\n\
                    Based on your personality, speak to the group. \n\
                    Reference specific points from the history if they exist.\n\
                    \n\
                    MANDATORY FORMAT:\n\
                    [Thinking]\n\
                    (Internal Monologue: specific reaction to the history)\n\
                    [Verdict]\n\
                    (Spoken Response: 1-2 sentences)\n\
                    <|end|>\n<|assistant|>",
                    agent.name, agent.role, agent.speaking_style,
                    history_snapshot, // <--- Injection of shared state
                    product_context,
                    round,
                    stage_instruction
                );

                // Inference
                let raw = brain.generate(&prompt, 400, None, None, temp);
                
                // Parse (Using robust parser logic)
                let (response, thought) = Self::parse_response(&raw);

                // Return Result linked to ORIGINAL AGENT ID
                SimulationResult {
                    agent_id: agent.id, 
                    agent_name: Some(agent.name.clone()), 
                    agent_role: agent.name.clone(), 
                    agent_demographic: format!("{} ({})", agent.role, agent.demographic),
                    scenario: "focus_group".to_string(),
                    timestamp: Local::now().to_rfc3339(),
                    prompt: "Context Injection".to_string(),
                    response: response,
                    thought_process: thought,
                    
                    // --- FIXED: Initialize sources as None ---
                    // Focus groups use shared context (product_context), not individual skills per turn.
                    sources: None, 
                    // ----------------------------------------
                    
                    sentiment: "neutral".to_string(),
                    category: Some(format!("Round {}", round)),
                }
            }).collect();

            // 3. Update Blackboard (Write access)
            // We append the new responses to the history so the next round sees them.
            let mut history_guard = room_history.lock().await;
            for res in &round_results {
                // Use agent_name for the transcript history so agents know who said what
                let speaker = res.agent_name.clone().unwrap_or("Participant".to_string());
                history_guard.push_str(&format!("{}: \"{}\"\n", speaker, res.response));
                results.push(res.clone());
            }
        }

        results
    }

    // Helper: Parse [Thinking] and [Verdict] tags
    fn parse_response(raw: &str) -> (String, Option<String>) {
        let clean = raw.replace("---", "").replace("ROOM CONTEXT", "").trim().to_string();
        
        if let Some(v_idx) = clean.find("[Verdict]") {
            let verdict = clean[v_idx+9..].trim().to_string();
            
            let thought = if let Some(t_idx) = clean.find("[Thinking]") {
                 // Extract text between [Thinking] and [Verdict]
                 Some(clean[t_idx+10..v_idx].trim().to_string())
            } else { 
                None 
            };
            
            (verdict, thought)
        } else {
            // Fallback: If tags missing, treat whole text as verdict
            (clean, None)
        }
    }
}