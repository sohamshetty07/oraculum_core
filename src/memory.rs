// src/memory.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::cmp::Ordering;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MemoryType {
    Observation, 
    Reflection,  
    Plan,        
    Fact,        
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: String,
    pub content: String,
    pub memory_type: MemoryType,
    pub creation_timestamp: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub importance: f32, 
    pub related_ids: Vec<String>, 
}

impl Memory {
    pub fn new(content: String, memory_type: MemoryType, importance: f32) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            content,
            memory_type,
            creation_timestamp: Utc::now(),
            last_accessed: Utc::now(),
            importance,
            related_ids: Vec::new(),
        }
    }
}

// The "Brain" Container
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryStream {
    pub memories: Vec<Memory>,
}

impl MemoryStream {
    pub fn new() -> Self {
        Self { memories: Vec::new() }
    }

    pub fn add_memory(&mut self, content: String, kind: MemoryType, importance: f32) {
        let mem = Memory::new(content, kind, importance);
        self.memories.push(mem);
    }

    pub fn retrieve(&mut self, query: &str, limit: usize) -> Vec<Memory> {
        let now = Utc::now();
        
        let mut scored: Vec<(usize, f32)> = self.memories.iter().enumerate().map(|(i, mem)| {
            let hours_since = (now - mem.creation_timestamp).num_hours() as f32;
            let recency = 0.99f32.powf(hours_since);
            let importance = mem.importance;
            let relevance = if query.split_whitespace().any(|word| mem.content.to_lowercase().contains(&word.to_lowercase())) {
                1.0
            } else {
                0.0
            };
            let score = (recency * 0.5) + (importance * 0.3) + (relevance * 2.0);
            (i, score)
        }).collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));

        let mut result = Vec::new();
        for (idx, _) in scored.into_iter().take(limit) {
            let mem = &mut self.memories[idx];
            mem.last_accessed = now;
            result.push(mem.clone());
        }
        result
    }
}