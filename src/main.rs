// src/main.rs
// ORACULUM CORE - API SERVER
// Serves the React Frontend via REST API (Actix-Web)

use actix_web::{web, App, HttpServer};
use actix_cors::Cors;
use std::sync::Arc;
use dashmap::DashMap;

// --- MODULE REGISTRATION ---
// Preserving all your existing logic modules
mod brain;
mod agent_swarm;
mod scenarios;
mod reporter;
mod persona_generator;
mod api;
mod focus_group;
mod analyst;
mod scout;
mod memory;
mod wiki;
mod skills;   // Manages the Agents' capabilities (WebScout, etc.)
mod systems;  // NEW: Manages External Connections (Sensory Cortex -> Python)

use brain::AgentBrain;
use agent_swarm::{AgentSwarm, SimulationResult};
use scenarios::Scenario;
use skills::{SkillRegistry, SkillInput};

// Shared State for the Server
pub struct AppState {
    pub brain: Arc<AgentBrain>,
    pub jobs: Arc<DashMap<String, api::JobStatus>>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🚀 Oraculum API Server Starting...");
    
    // 1. Initialize Neural Engine (ONCE at startup)
    let brain = Arc::new(AgentBrain::new());
    
    // 2. Initialize Job Store
    let jobs = Arc::new(DashMap::new());
    
    // 3. Create Shared State
    let app_state = web::Data::new(AppState {
        brain: brain.clone(),
        jobs: jobs.clone(),
    });

    println!("🌍 Server running at http://127.0.0.1:8080");

    // 4. Start HTTP Server
    HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .wrap(cors)
            .app_data(app_state.clone())
            .route("/api/simulate", web::post().to(api::start_simulation))
            .route("/api/status/{id}", web::get().to(api::get_job_status))
            .route("/api/analyze", web::post().to(api::analyze_job))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

// SHARED HELPER FUNCTION
// This runs on a separate thread (via Rayon) when the API is called
pub fn run_simulation_parallel(
    brain: &Arc<AgentBrain>,
    swarm: &Arc<AgentSwarm>,
    scenario: &Box<dyn Scenario>,
    image_data: Option<String>,
    pdf_data: Option<String>,
    product_context: String, // Context passed from API (e.g. "Price of Bulbasaur")
) {
    use rayon::prelude::*;
    
    let agents = swarm.get_agents();
    
    // Parallel Agent Execution
    let results: Vec<SimulationResult> = agents
        .par_iter()
        .map(|agent| {
            // 1. Generate Base Prompt (Who am I?)
            let mut prompt = scenario.generate_prompt(agent, None);

            // 2. --- SKILL EXECUTION (WEB / RAG) ---
            let mut acquired_knowledge = String::new();
            
            // If the agent has skills (e.g., ["web_scout"]), execute them
            if !agent.skills.is_empty() {
                let registry = SkillRegistry::new();
                
                for skill_id in &agent.skills {
                    if let Some(skill) = registry.get(skill_id) {
                        
                        // Pass the Product Context to the skill
                        let input = SkillInput {
                            query: product_context.clone(),
                            context: agent.demographic.clone()
                        };
                        
                        println!("[AGENT] {} is executing skill: {}", agent.name, skill_id);
                        let output = skill.execute(brain, input);
                        
                        if output.success {
                            acquired_knowledge.push_str(&format!(
                                "\n### SENSORY OBSERVATION (Source: {})\n{}\n", 
                                skill_id.to_uppercase(), 
                                output.data
                            ));
                        } else {
                             println!("[WARN] Skill {} failed for agent {}", skill_id, agent.name);
                        }
                    }
                }
            }

            // 3. Inject Knowledge into Prompt
            if !acquired_knowledge.is_empty() {
                // We inject this BEFORE the final instruction so the agent "knows" it before "speaking"
                let knowledge_block = format!("\n\n=== REAL-WORLD CONTEXT ACQUIRED ===\n{}\n===================================\nUse the facts above to answer accurately.\n", acquired_knowledge);
                
                prompt.push_str(&knowledge_block);
            }
            
            // 4. Inference (Using the gathered knowledge)
            let raw_response = brain.generate(&prompt, 800, image_data.clone(), pdf_data.clone(), 0.7);
            
            // 5. Process & Return
            let (response_text, thought_process) = scenario.process_response(&raw_response);
            let sentiment = AgentSwarm::sentiment_from_response(&response_text);
            let category = AgentSwarm::extract_category(&response_text, scenario.scenario_key());

            SimulationResult {
                agent_id: agent.id,
                agent_name: Some(agent.name.clone()),
                agent_role: agent.name.clone(),
                agent_demographic: format!("{} ({})", agent.role, agent.demographic),
                scenario: scenario.scenario_key().to_string(),
                timestamp: AgentSwarm::get_timestamp(),
                prompt,
                response: response_text,
                thought_process,
                
                // IMPORTANT: Populate sources so the UI shows where the data came from
                sources: if !acquired_knowledge.is_empty() { Some(acquired_knowledge) } else { None },
                
                sentiment,
                category,
            }
        })
        .collect();

    for res in results {
        swarm.add_result(res);
    }
}