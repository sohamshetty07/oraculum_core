// src/api.rs
use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::sync::Arc;
use crate::AppState;
use crate::agent_swarm::{Agent, SimulationResult, AgentSwarm};
use crate::scenarios::Scenario; 
use crate::persona_generator::PersonaGenerator;
use crate::focus_group::FocusGroupSession; // Use the new Async Session
use crate::analyst::AnalystEngine;
use std::thread;

// 1. The Request Format
#[derive(Deserialize)]
pub struct SimulationRequest {
    pub scenario: String,        
    pub product_name: String,    
    pub context: String,         
    pub target_audience: String, 
    pub agent_count: usize,
    pub image_data: Option<String>, 
    pub pdf_data: Option<String>,   
}

// 2. The Response Format
#[derive(Serialize)]
pub struct JobCreatedResponse {
    pub job_id: String,
    pub status: String,
}

// 3. The Job Status
#[derive(Serialize, Clone)]
pub struct JobStatus {
    pub id: String,
    pub status: String, 
    pub progress: f32,  
    pub agents: Vec<Agent>,
    pub results: Vec<SimulationResult>,
}

// 4. Analysis Payloads
#[derive(Deserialize)]
pub struct AnalyzeRequest {
    pub job_id: String,
}

#[derive(Serialize)]
pub struct AnalysisResponse {
    pub report: String,
}

// POST /api/simulate
pub async fn start_simulation(
    data: web::Data<AppState>,
    req: web::Json<SimulationRequest>,
) -> impl Responder {
    let job_id = Uuid::new_v4().to_string();
    let brain = data.brain.clone();
    let jobs = data.jobs.clone();

    // Create initial empty job state
    let initial_status = JobStatus {
        id: job_id.clone(),
        status: "processing".to_string(),
        progress: 0.0,
        agents: Vec::new(),
        results: Vec::new(),
    };
    jobs.insert(job_id.clone(), initial_status);

    // Prepare variables for thread
    let job_id_clone = job_id.clone();
    let req_scenario = req.scenario.clone();
    let req_count = req.agent_count;
    let req_target = req.target_audience.clone();
    let req_product = req.product_name.clone();
    let req_context = req.context.clone();
    let req_image = req.image_data.clone(); 
    let req_pdf = req.pdf_data.clone(); 

    // SPAWN THREAD
    thread::spawn(move || {
        println!("🚀 API: Starting Job {} [Scenario: {}]", job_id_clone, req_scenario);

        // --- STEP 0: FEDERATED INTELLIGENCE GATHERING (The Triad) ---
        println!("🕵️ SCOUT: Initiating Federated Research (Reddit + Wiki)...");
        let research_data = brain.research(&req_product, &req_context);
        
        println!("📦 SCOUT: Fetching Product Specifications...");
        let fact_sheet = brain.get_facts(&req_product);

        let voices_text = if research_data.is_empty() {
            "No direct consumer discussions found online.".to_string()
        } else {
            research_data.join("\n---\n")
        };

        let enriched_context = format!(
            "PRODUCT: {}\nUSER CONTEXT: {}\n\n--- FACTUAL SPECS (Open Database) ---\n{}\n\n--- MARKET RESEARCH (Reddit Voices & Wiki) ---\n{}", 
            req_product, 
            req_context, 
            fact_sheet, 
            voices_text
        );

        // --- STEP 1: DOPPELGÄNGER GENERATION ---
        let agents = PersonaGenerator::generate_from_voices(req_count, &req_target, research_data, &brain);
        
        if let Some(mut job) = jobs.get_mut(&job_id_clone) {
            job.agents = agents.clone();
            job.progress = 0.25; 
        }

        let swarm = Arc::new(AgentSwarm {
            agents: Arc::new(std::sync::Mutex::new(agents)),
            results: Arc::new(std::sync::Mutex::new(Vec::new())),
        });

        // 3. EXECUTION BRANCHING
        if req_scenario == "focus_group" {
            // --- ASYNC DEBATE MODE (Blackboard Architecture) ---
            
            // Create a temporary Tokio runtime for the async debate execution
            let rt = tokio::runtime::Runtime::new().unwrap();
            
            let debate_results = rt.block_on(async {
                // Call the new async blackboard engine
                // We pass enriched_context so the agents know the full picture (Wiki + Reddit)
                crate::focus_group::FocusGroupSession::run_debate(
                    &brain, 
                    &swarm.get_agents(), 
                    &enriched_context, 
                    3 // 3 Rounds
                ).await
            });

            // Store results in the Swarm & Job Store
            for res in debate_results {
                swarm.add_result(res);
            }
            
            // Update Job Progress
            if let Some(mut job) = jobs.get_mut(&job_id_clone) {
                 job.results = swarm.get_results(); 
                 job.progress = 0.90;
            }

        } else {
            // --- STANDARD PARALLEL MODE ---
            let scenario: Box<dyn Scenario> = match req_scenario.as_str() {
                "creative_test" => Box::new(crate::scenarios::CreativeTestScenario::new(
                    &req_product, 
                    &enriched_context,
                    "Ad Campaign"
                )),
                "ab_messaging" => Box::new(crate::scenarios::ABMessagingScenario::new(
                    &req_product, 
                    &enriched_context,
                    "Brand Positioning"
                )),
                "cx_flow" => Box::new(crate::scenarios::CXFlowScenario::new(
                    "consideration", 
                    &format!("{} - {}", req_product, enriched_context)
                )),
                _ => Box::new(crate::scenarios::ProductLaunchScenario::new(
                    &req_product,
                    "Consumer Product",
                    vec![&enriched_context]
                )),
            };

            crate::run_simulation_parallel(&brain, &swarm, &scenario, req_image, req_pdf);
        }
        
        // 5. Complete Job
        if let Some(mut job) = jobs.get_mut(&job_id_clone) {
            job.results = swarm.get_results();
            job.status = "completed".to_string();
            job.progress = 1.0;
        }
        println!("✅ API: Job {} Finished", job_id_clone);
    });

    HttpResponse::Ok().json(JobCreatedResponse {
        job_id,
        status: "processing".to_string(),
    })
}

// GET /api/status/{job_id}
pub async fn get_job_status(
    data: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    let job_id = path.into_inner();
    
    if let Some(job) = data.jobs.get(&job_id) {
        HttpResponse::Ok().json(job.clone())
    } else {
        HttpResponse::NotFound().body("Job not found")
    }
}

// POST /api/analyze
pub async fn analyze_job(
    data: web::Data<AppState>,
    req: web::Json<AnalyzeRequest>,
) -> impl Responder {
    let job_id = req.job_id.clone();
    println!("📊 API: Analysis requested for Job {}", job_id);

    // 1. Retrieve Job Data safely
    let (results, scenario_key) = if let Some(job) = data.jobs.get(&job_id) {
        let results = job.results.clone();
        let scenario = results.first().map(|r| r.scenario.clone()).unwrap_or_else(|| "unknown".to_string());
        (results, scenario)
    } else {
        return HttpResponse::NotFound().body("Job not found");
    };

    if results.is_empty() {
        return HttpResponse::BadRequest().body("No results available to analyze");
    }

    // 2. Call the Analyst Engine
    let brain = data.brain.clone();
    let report_result = web::block(move || {
        AnalystEngine::generate_report(&brain, &scenario_key, &results)
    }).await;

    // 3. Return the Report
    match report_result {
        Ok(report) => HttpResponse::Ok().json(AnalysisResponse { report }),
        Err(e) => {
            println!("❌ API Error: Analysis generation failed: {}", e);
            HttpResponse::InternalServerError().body("Failed to generate report")
        }
    }
}