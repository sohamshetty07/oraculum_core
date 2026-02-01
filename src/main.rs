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
mod analyst; // <--- NEW: Register the Analyst Engine module
mod scout;
mod memory;
mod wiki; // <--- NEW: Register the Wiki module

use brain::AgentBrain;
use agent_swarm::{AgentSwarm, SimulationResult};
use scenarios::Scenario;

// Shared State for the Server
// This structure is passed to every API route handler
pub struct AppState {
    pub brain: Arc<AgentBrain>, // The Python Process (Neural Engine)
    pub jobs: Arc<DashMap<String, api::JobStatus>>, // In-memory Job Store
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🚀 Oraculum API Server Starting...");
    
    // 1. Initialize Neural Engine (ONCE at startup)
    // This spawns the Python process. It stays alive for the life of the server.
    let brain = Arc::new(AgentBrain::new());
    
    // 2. Initialize Job Store
    // Stores simulation status so the frontend can poll for updates
    let jobs = Arc::new(DashMap::new());
    
    // 3. Create Shared State
    let app_state = web::Data::new(AppState {
        brain: brain.clone(),
        jobs: jobs.clone(),
    });

    println!("🌍 Server running at http://127.0.0.1:8080");

    // 4. Start HTTP Server
    HttpServer::new(move || {
        // Configure CORS to allow the Frontend (port 3000 usually) to talk to us
        let cors = Cors::permissive(); 

        App::new()
            .wrap(cors)
            .app_data(app_state.clone()) // Inject state
            .route("/api/simulate", web::post().to(api::start_simulation)) // Start job
            .route("/api/status/{id}", web::get().to(api::get_job_status)) // Check job
            .route("/api/analyze", web::post().to(api::analyze_job)) // <--- NEW: Register the Analysis Endpoint
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

// SHARED HELPER FUNCTION
// This is called by the API thread to run the actual heavy lifting
// It mimics the old main() loop but is now callable on demand
pub fn run_simulation_parallel(
    brain: &Arc<AgentBrain>,
    swarm: &Arc<AgentSwarm>,
    scenario: &Box<dyn Scenario>,
    image_data: Option<String>, // <--- NEW ARGUMENT: Carries the Base64 image
    pdf_data: Option<String>, // <--- NEW
) {
    use rayon::prelude::*;
    
    let agents = swarm.get_agents();
    
    // Simple parallel loop using Rayon
    // Sends prompts to Python bridge concurrently
    let results: Vec<SimulationResult> = agents
        .par_iter()
        .map(|agent| {
            let prompt = scenario.generate_prompt(agent, None);
            
            // Ask the Brain (Python)
            // PASS IMAGE TO BRAIN: We clone the image data for each thread
            // Updated Temp to 0.7 for creativity as discussed
            let raw_response = brain.generate(&prompt, 800, image_data.clone(), pdf_data.clone(), 0.7);
            
            // --- UPDATED PARSING ---
            // Use Scenario's robust parser to handle [Thinking] and [Verdict] tags
            // plus strip artifacts like "---"
            let (response_text, thought_process) = scenario.process_response(&raw_response);

            // Analyze
            let sentiment = AgentSwarm::sentiment_from_response(&response_text);
            let category = AgentSwarm::extract_category(&response_text, scenario.scenario_key());

            // Return Result
            SimulationResult {
                agent_id: agent.id,
                // ADDED: Missing field required by new struct definition
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
                sentiment,
                category,
            }
        })
        .collect();

    // Store all results in the swarm
    for res in results {
        swarm.add_result(res);
    }
}