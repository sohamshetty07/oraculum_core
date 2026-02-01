// src/reporter.rs
// Enhanced CSV & JSON Export with proper escaping
// UPDATE: Now captures 'thought_process' (Hidden Thoughts)

use crate::agent_swarm::{Agent, SimulationResult};
use std::fs::File;
use std::error::Error;
use csv::Writer;

pub struct Reporter;

impl Reporter {
    pub fn export_csv(
        filename: &str,
        results: &[SimulationResult],
    ) -> Result<(), Box<dyn Error>> {
        let mut wtr = Writer::from_path(filename)?;

        // Write CSV header
        wtr.write_record(&[
            "agent_id",
            "agent_role",
            "agent_demographic", 
            "scenario",
            "timestamp",
            "prompt",
            "response",
            "thought_process", // <--- NEW COLUMN
            "sentiment",
            "category",
        ])?;

        // Write each result
        for result in results {
            wtr.write_record(&[
                &result.agent_id.to_string(),
                &result.agent_role,
                &result.agent_demographic, 
                &result.scenario,
                &result.timestamp,
                &result.prompt,
                &result.response,
                // Handle optional thought process safely
                result.thought_process.as_deref().unwrap_or(""),
                &result.sentiment,
                result.category.as_deref().unwrap_or(""),
            ])?;
        }

        wtr.flush()?;
        println!("✅ CSV exported to: {}", filename);
        Ok(())
    }

    pub fn export_json(
        filename: &str,
        agents: &[Agent],
        results: &[SimulationResult],
    ) -> Result<(), Box<dyn Error>> {
        let mut personas = Vec::new();

        for agent in agents {
            let agent_results: Vec<_> =
                results.iter().filter(|r| r.agent_id == agent.id).collect();

            let sentiment_sum: f32 = agent_results
                .iter()
                .map(|r| match r.sentiment.as_str() {
                    "positive" => 1.0,
                    "negative" => -1.0,
                    "neutral" => 0.0,
                    _ => 0.5,
                })
                .sum();

            let avg_sentiment = if !agent_results.is_empty() {
                sentiment_sum / agent_results.len() as f32
            } else {
                0.5
            };

            let persona = serde_json::json!({
                "id": agent.id,
                "name": agent.name,
                "role": agent.role,
                "demographic": agent.demographic,
                "beliefs": agent.beliefs,
                "spending_profile": agent.spending_profile,
                "product_affinity": agent.product_affinity,
                "messaging_resonance": agent.messaging_resonance,
                "simulated_responses": agent_results.len(),
                "avg_sentiment": avg_sentiment,
                "recent_responses": agent_results
                    .iter()
                    .take(5) // Increased history depth
                    .map(|r| serde_json::json!({
                        "scenario": r.scenario,
                        "response": r.response,
                        "thought_process": r.thought_process, // <--- NEW FIELD
                        "sentiment": r.sentiment,
                        "category": r.category
                    }))
                    .collect::<Vec<_>>()
            });

            personas.push(persona);
        }

        let output = serde_json::json!({
            "personas": personas,
            "total_agents": agents.len(),
            "total_responses": results.len(),
            "export_timestamp": chrono::Utc::now().to_rfc3339(),
        });

        let mut file = File::create(filename)?;
        use std::io::Write;
        file.write_all(serde_json::to_string_pretty(&output)?.as_bytes())?;

        println!("✅ JSON personas exported to: {}", filename);
        Ok(())
    }

    pub fn print_summary(agents: &[Agent], results: &[SimulationResult]) {
        println!("\n📊 SIMULATION SUMMARY");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Total Agents: {}", agents.len());
        println!("Total Responses: {}", results.len());

        // Count hidden thoughts captured
        let thought_count = results.iter().filter(|r| r.thought_process.is_some()).count();
        println!("Hidden Thoughts Captured: {} (Cognitive Depth: {:.0}%)", 
            thought_count, 
            if results.is_empty() { 0.0 } else { (thought_count as f32 / results.len() as f32) * 100.0 }
        );

        // Sentiment breakdown
        let positive = results.iter().filter(|r| r.sentiment == "positive").count();
        let negative = results.iter().filter(|r| r.sentiment == "negative").count();
        let neutral = results.iter().filter(|r| r.sentiment == "neutral").count();
        let mixed = results.iter().filter(|r| r.sentiment == "mixed").count();

        println!("\n📈 Sentiment Distribution:");
        println!("  Positive: {:.1}%", if results.len() > 0 { (positive as f32 / results.len() as f32) * 100.0 } else { 0.0 });
        println!("  Negative: {:.1}%", if results.len() > 0 { (negative as f32 / results.len() as f32) * 100.0 } else { 0.0 });
        println!("  Neutral: {:.1}%", if results.len() > 0 { (neutral as f32 / results.len() as f32) * 100.0 } else { 0.0 });
        println!("  Mixed: {:.1}%", if results.len() > 0 { (mixed as f32 / results.len() as f32) * 100.0 } else { 0.0 });

        // By scenario
        let scenarios: std::collections::HashSet<_> =
            results.iter().map(|r| r.scenario.clone()).collect();
        println!("\n🎯 Response by Scenario:");
        for scenario in scenarios {
            let count = results.iter().filter(|r| r.scenario == scenario).count();
            println!("  {}: {} responses", scenario, count);
        }

        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    }
}