use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Difficulty {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
    Master,
    Unrated,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Song {
    pub id: usize,
    pub artist: String,
    pub title: String,
    pub difficulty: Difficulty,
    pub link: String,
    pub sequence_number: String,
}
