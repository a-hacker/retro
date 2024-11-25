// src/models.rs

use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

// Represents a Card in a Retro
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub id: i32,
    pub text: String,
}

// Represents a Retro
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Retro {
    pub id: i32,
    pub retro_name: String,
    pub creator_name: String,
    pub created_at: String, // ISO 8601 format
    pub users: Vec<String>,
    pub cards: Cards,
}

// Categorized Cards within a Retro
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cards {
    pub good: Vec<Card>,
    pub bad: Vec<Card>,
    pub needs_improvement: Vec<Card>,
}

// Shared State: In-memory storage using Arc and RwLock for thread safety
pub type SharedRetros = Arc<RwLock<Vec<Retro>>>;
