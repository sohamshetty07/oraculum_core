// src/batch_inference.rs
// UPDATED: Compatible with Python Bridge & MAGMA Architecture

use crate::agent_swarm::{Agent, SimulationResult, AgentSwarm};
use crate::brain::AgentBrain;
use crate::scenarios::Scenario;
use std::sync::Arc;
use rayon::prelude::*;

#[derive(Clone, Debug)]
pub struct InferenceRequest {
    pub agent_id: u32,
    pub agent_name: String,
    pub agent_role: String,
    pub agent_demographic: String, // <--- Added to track demographic info
    pub scenario_key: String,      // <--- Added for accurate categorization
    pub prompt: String,
}

#[derive(Clone, Debug)]
pub struct InferenceResult {
    pub agent_id: u32,
    pub response: String,
    pub thought_process: Option<String>, // <--- Added to capture hidden thoughts
    pub sentiment: String,
    pub category: String,
}

pub struct BatchInferenceEngine {
    pub brain: Arc<AgentBrain>,
    pub batch_size: usize,
}

impl BatchInferenceEngine {
    pub fn new(brain: Arc<AgentBrain>, batch_size: usize) -> Self {
        Self {
            brain,
            batch_size,
        }
    }

    /// Process requests: parallel batch processing using the Python Brain
    pub fn process_requests(&self, requests: Vec<InferenceRequest>) -> Vec<InferenceResult> {
        let total = requests.len();
        let counter = Arc::new(std::sync::Mutex::new(0usize));

        requests
            .par_chunks(self.batch_size)
            .flat_map(|chunk| {
                chunk
                    .iter()
                    .map(|req| {
                        // Progress counter
                        let mut count = counter.lock().unwrap();
                        *count += 1;
                        let current = *count;
                        drop(count);

                        print!(
                            "\r[{}/{}] {} ({})",
                            current, total, req.agent_name, req.agent_role
                        );
                        std::io::Write::flush(&mut std::io::stdout()).ok();

                        // 1. Generate via Python Bridge
                        // Using a reasonable token limit for batch responses (e.g., 300)
                        let raw_response = self.brain.generate(&req.prompt, 300, None, None);

                        // 2. Parse Thought vs Verdict (MAGMA Architecture)
                        // This ensures we capture the "Hidden Thought" even in batch mode
                        let (thought, content) = if let Some(verdict_start) = raw_response.find("[Verdict]") {
                            let verdict = raw_response[verdict_start + 9..].trim().to_string();
                            let thought_text = if let Some(think_start) = raw_response.find("[Thinking]") {
                                Some(raw_response[think_start + 10..verdict_start].trim().to_string())
                            } else {
                                None
                            };
                            (thought_text, verdict)
                        } else {
                            // Fallback cleanup if tags are missing
                            let clean = raw_response
                                .replace("[Thinking]", "")
                                .replace("[Verdict]", "")
                                .trim()
                                .to_string();
                            (None, clean)
                        };

                        let sentiment = AgentSwarm::sentiment_from_response(&content);
                        let category = AgentSwarm::extract_category(&content, &req.scenario_key);

                        InferenceResult {
                            agent_id: req.agent_id,
                            response: content,
                            thought_process: thought,
                            sentiment,
                            category,
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    /// Demo mode: instant simulated responses with realistic variation
    pub fn process_requests_demo(&self, requests: Vec<InferenceRequest>) -> Vec<InferenceResult> {
        let responses = vec![
            "The product is good quality affordable.",
            "Like this very much premium value.",
            "Not worth expensive price point.",
            "Good product affordable quick delivery.",
            "Really great quality very good.",
            "Bad quality not worth buying.",
            "Like the product affordable price.",
            "Product is decent very good.",
            "Good quality worth the price.",
            "Premium product excellent very worth.",
        ];

        requests
            .iter()
            .enumerate()
            .map(|(idx, req)| {
                let template = responses[idx % responses.len()];
                InferenceResult {
                    agent_id: req.agent_id,
                    response: template.to_string(),
                    thought_process: None,
                    sentiment: AgentSwarm::sentiment_from_response(template),
                    category: "product_quality".to_string(),
                }
            })
            .collect()
    }
}

/// Convert agents to inference requests
pub fn prepare_inference_requests(
    agents: &[Agent],
    scenario: &Box<dyn Scenario>,
) -> Vec<InferenceRequest> {
    agents
        .iter()
        .map(|agent| InferenceRequest {
            agent_id: agent.id,
            agent_name: agent.name.clone(),
            agent_role: agent.role.clone(),
            agent_demographic: agent.demographic.clone(), // Capture demographic
            scenario_key: scenario.scenario_key().to_string(), // Capture scenario key
            prompt: scenario.generate_prompt(agent, None),
        })
        .collect()
}

/// Convert inference results back to simulation results
pub fn convert_to_simulation_results(
    requests: Vec<InferenceRequest>,
    results: Vec<InferenceResult>,
    scenario_key: &str,
) -> Vec<SimulationResult> {
    results
        .into_iter()
        .zip(requests.into_iter())
        .map(|(result, req)| SimulationResult {
            agent_id: result.agent_id,
            agent_role: req.agent_role,
            agent_demographic: req.agent_demographic, // Pass through
            scenario: scenario_key.to_string(),
            timestamp: AgentSwarm::get_timestamp(),
            prompt: req.prompt,
            response: result.response,
            thought_process: result.thought_process, // Pass through
            sentiment: result.sentiment,
            category: result.category,
        })
        .collect()
}