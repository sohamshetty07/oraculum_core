// src/focus_group.rs
// SOCIETY ENGINE V3.2: Anti-Echo Enforcement & Cognitive Clarity

use std::sync::Arc;
use crate::brain::AgentBrain;
use crate::agent_swarm::Agent;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub agent_id: u32,
    pub agent_name: String,
    pub role: String, 
    pub content: String,
    pub round: usize,
    pub phase: String,
    // Capture the hidden thought (The Private Graph Query Result)
    pub thought_process: Option<String>, 
}

pub struct FocusGroupSession {
    pub topic: String,
    pub context_history: Vec<ChatMessage>,
}

impl FocusGroupSession {
    pub fn new(topic: &str) -> Self {
        Self {
            topic: topic.to_string(),
            context_history: Vec::new(),
        }
    }

    pub fn generate_executive_summary(&self, brain: &Arc<AgentBrain>) -> String {
        let mut full_transcript = String::new();
        for msg in &self.context_history {
            full_transcript.push_str(&format!("[{}] {}: \"{}\"\n", msg.phase, msg.role, msg.content));
        }

        let prompt = format!(
            "<|user|>You are the Lead Market Research Analyst for Oraculum. \
            Review the following focus group transcript regarding '{}'.\n\n\
            --- TRANSCRIPT START ---\n\
            {}\n\
            --- TRANSCRIPT END ---\n\n\
            TASK: Write a strategic Executive Summary.\n\
            REQUIREMENTS: Consensus Score (0-100%), Key Themes, Controversies, and 3 Actionable Insights.<|end|>\n<|assistant|>",
            self.topic,
            full_transcript
        );
        brain.generate(&prompt, 1024, None, None)
    }

    pub fn run_round(
        &mut self, 
        brain: &Arc<AgentBrain>, 
        agents: &mut [Agent], 
        round_num: usize
    ) -> Vec<ChatMessage> {
        let mut round_messages: Vec<ChatMessage> = Vec::new();
        
        // 1. PHASE CONFIGURATION
        let (phase_name, is_debate_round) = match round_num {
            1 => ("Initial Reactions", false),
            2 => ("The Debate (Conflict)", true), // <--- The Devil's Advocate Round
            3 => ("Final Verdict", false),
            _ => ("Discussion", false),
        };

        println!("🗣️ SOCIETY ENGINE: Starting {}...", phase_name);

        // 2. IDENTIFY THE DEVIL'S ADVOCATE (For Round 2)
        let devils_advocate_id = if is_debate_round {
            agents.iter()
                .max_by_key(|a| if a.skepticism_level == "High" { 2 } else { 1 })
                .map(|a| a.id)
        } else {
            None
        };

        // --- NEW: Track the last statement to prevent echoes ---
        let mut last_statement = String::new();

        for agent in agents.iter_mut() {
            // 3. CONSTRUCT CONTEXT (The "Room Temperature")
            let mut conversation_log = String::new();
            let history_window = self.context_history.len().saturating_sub(8); 
            
            for msg in &self.context_history[history_window..] {
                conversation_log.push_str(&format!("{}: \"{}\"\n", msg.agent_name, msg.content));
            }
            // Add current round messages (immediate context)
            for msg in &round_messages {
                conversation_log.push_str(&format!("{}: \"{}\"\n", msg.agent_name, msg.content));
            }

            // 4. DYNAMIC INSTRUCTION GENERATION
            let mut secret_objective = String::new();
            let instruction; 

            // --- ANTI-ECHO INJECTION ---
            // We explicitly ban the previous phrase to force variety
            let anti_echo_prompt = if !last_statement.is_empty() {
                // Take the first 60 chars of the last statement to prevent exact repetition
                let snippet: String = last_statement.chars().take(60).collect();
                format!("CONSTRAINT: Do NOT repeat or paraphrase \"{}...\". Add a NEW perspective.", snippet)
            } else {
                "CONSTRAINT: Start the conversation with a strong, unique opinion.".to_string()
            };

            if is_debate_round && Some(agent.id) == devils_advocate_id {
                secret_objective = "SECRET OBJECTIVE: You are the designated Devil's Advocate. \
                Even if you like the product, you MUST find a flaw. \
                Attack the consensus. Call out others for being too naive.".to_string();
                
                instruction = "Review the chat history. Pick the most popular opinion and dismantle it.".to_string();
            } else {
                match round_num {
                    1 => instruction = format!(
                        "Give your immediate reaction based on your '{}' skepticism. Be honest.", 
                        agent.skepticism_level
                    ),
                    2 => instruction = "Look at the arguments so far. Pick a side (Defend or Attack). Do not be neutral.".to_string(),
                    _ => instruction = "Give your final verdict. Yes or No? Be decisive.".to_string(),
                };
            }

            // 5. THE COGNITIVE PROMPT
            // Simplified structure for Phi-3.5 to strictly follow the format
            let prompt = format!(
                "<|user|>Roleplay as a participant in a market research focus group.\n\n\
                --- IDENTITY ---\n\
                Name: {}\n\
                Role: {} ({})\n\
                Traits: Style='{}', Skepticism='{}'\n\n\
                --- ROOM CONTEXT ---\n\
                Topic: {}\n\
                History:\n{}\n\n\
                --- YOUR ORDERS ---\n\
                Phase: {}\n\
                {}\n\
                {}\n\n\
                {}\n\n\
                MANDATORY RESPONSE FORMAT:\n\
                [Thinking]\n\
                (Write your internal monologue here. Analyze if the previous speaker is wrong.)\n\
                [Verdict]\n\
                (Write your spoken response here. Keep it natural and under 2 sentences.)\n\
                \n\
                Response:<|end|>\n<|assistant|>",
                agent.name, agent.role, agent.demographic,
                agent.speaking_style, agent.skepticism_level,
                self.topic,
                if conversation_log.is_empty() { "No conversation yet." } else { &conversation_log },
                phase_name,
                instruction,
                secret_objective, // Only visible to the specific agent
                anti_echo_prompt  // <--- The Anti-Echo Constraint
            );

            // 6. EXECUTE
            let raw_response = brain.generate(&prompt, 600, None, None);

            // 7. PARSE THOUGHT vs SPEECH
            let (thought, content) = if let Some(verdict_start) = raw_response.find("[Verdict]") {
                let verdict = raw_response[verdict_start + 9..].trim().to_string();
                let thought = if let Some(think_start) = raw_response.find("[Thinking]") {
                    Some(raw_response[think_start + 10..verdict_start].trim().to_string())
                } else {
                    None
                };
                (thought, verdict)
            } else {
                // Fallback cleanup if tags are missing
                let clean = raw_response
                    .replace("[Thinking]", "")
                    .replace("[Verdict]", "")
                    .trim()
                    .to_string();
                (None, clean)
            };

            // Update the last statement so the NEXT agent knows what not to repeat
            last_statement = content.clone();

            let msg = ChatMessage {
                agent_id: agent.id,
                agent_name: agent.name.clone(),
                role: agent.role.clone(),
                content,
                round: round_num,
                phase: phase_name.to_string(),
                thought_process: thought,
            };

            round_messages.push(msg.clone());
        }

        self.context_history.extend(round_messages.clone());
        round_messages
    }
}