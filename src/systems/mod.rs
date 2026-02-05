// src/systems/mod.rs
pub mod sensory; // <--- The 'pub' is crucial. Without it, 'skills.rs' cannot see 'sensory.rs'.