// src/main.rs
// ORACULUM CORE - API SERVER
// Serves the React Frontend via REST API (Actix-Web)

use actix_web::{web, App, HttpServer};
use actix_cors::Cors;
use std::sync::Arc;
use dashmap::DashMap; // Thread-safe hashmap for storing jobs

// Modules
mod brain;
mod agent_swarm;
mod scenarios;
mod reporter;
mod persona_generator;
mod api; // New API module handling requests
mod focus_group; 
mod analyst; // Register the Analyst Engine module
mod scout;
mod memory;
mod wiki; // Register the Wiki module
mod skills; // Register the Skills module

use brain::AgentBrain;
use agent_swarm::{AgentSwarm, SimulationResult};
use scenarios::Scenario;
use skills::{SkillRegistry, SkillInput}; // Import Skill components

// Shared State for the Server
pub struct AppState {
    pub brain: Arc<AgentBrain>, // The Python Process (Neural Engine)
    pub jobs: Arc<DashMap<String, api::JobStatus>>, // In-memory Job Store
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
// This is called by the API thread to run the actual heavy lifting
pub fn run_simulation_parallel(
    brain: &Arc<AgentBrain>,
    swarm: &Arc<AgentSwarm>,
    scenario: &Box<dyn Scenario>,
    image_data: Option<String>, 
    pdf_data: Option<String>, 
    product_context: String, // Context passed from API to guide Skill Research
) {
    use rayon::prelude::*;
    
    let agents = swarm.get_agents();
    
    // Simple parallel loop using Rayon
    let results: Vec<SimulationResult> = agents
        .par_iter()
        .map(|agent| {
            // 1. Generate Base Prompt (Who am I?)
            let mut prompt = scenario.generate_prompt(agent, None);

            // 2. --- COGNITIVE SKILL EXECUTION ---
            // Execute skills (e.g. 'deep_research') to query the Indian Brain (LanceDB)
            let mut acquired_knowledge = String::new();
            
            if !agent.skills.is_empty() {
                let registry = SkillRegistry::new();
                
                for skill_id in &agent.skills {
                    if let Some(skill) = registry.get(skill_id) {
                        // The agent queries the 'product_context' (e.g., "Mamaearth Onion Oil")
                        let input = SkillInput {
                            query: product_context.clone(), 
                            context: agent.demographic.clone() 
                        };
                        
                        let output = skill.execute(brain, input);
                        
                        if output.success {
                            acquired_knowledge.push_str(&format!(
                                "\n[SKILL USED: {}]\nResult: {}\n", 
                                skill_id, output.data
                            ));
                        }
                    }
                }
            }

            // 3. --- PROMPT INJECTION ---
            // Inject the gathered knowledge into the prompt
            if !acquired_knowledge.is_empty() {
                let knowledge_block = format!("\n--- ACQUIRED REAL-WORLD CONTEXT (MEMORY/WEB) ---\n{}\n", acquired_knowledge);
                
                if prompt.contains("INSTRUCTION:") {
                     prompt = prompt.replace("INSTRUCTION:", &format!("{}\nINSTRUCTION:", knowledge_block));
                } else {
                     prompt.push_str(&knowledge_block);
                }
            }
            
            // 4. Ask the Brain (Inference)
            let raw_response = brain.generate(&prompt, 800, image_data.clone(), pdf_data.clone(), 0.7);
            
            // 5. Parse
            let (response_text, thought_process) = scenario.process_response(&raw_response);

            // 6. Analyze
            let sentiment = AgentSwarm::sentiment_from_response(&response_text);
            let category = AgentSwarm::extract_category(&response_text, scenario.scenario_key());

            // 7. Return Result
            SimulationResult {
                agent_id: agent.id,
                agent_name: Some(agent.name.clone()),
                // MAPPED: Name as Role for UI Header consistency (e.g., "Priya")
                agent_role: agent.name.clone(),
                // MAPPED: Role + Demo for UI Subheader (e.g., "Trader (Mumbai...)")
                agent_demographic: format!("{} ({})", agent.role, agent.demographic),
                scenario: scenario.scenario_key().to_string(),
                timestamp: AgentSwarm::get_timestamp(),
                prompt,
                response: response_text, 
                thought_process, 
                
                // --- CRITICAL UPDATE: SOURCE ATTRIBUTION ---
                // Populates the "Sources" tab in the Frontend
                sources: if !acquired_knowledge.is_empty() { 
                    Some(acquired_knowledge) 
                } else { 
                    None 
                },
                // -------------------------------------------

                sentiment,
                category,
            }
        })
        .collect();

    for res in results {
        swarm.add_result(res);
    }
}