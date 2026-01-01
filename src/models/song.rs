use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Difficulty {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
    Master,
    Unrated,
}

impl Display for Difficulty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Difficulty::Beginner => write!(f, "Beginner"),
            Difficulty::Intermediate => write!(f, "Intermediate"),
            Difficulty::Advanced => write!(f, "Advanced"),
            Difficulty::Expert => write!(f, "Expert"),
            Difficulty::Master => write!(f, "Master"),
            Difficulty::Unrated => write!(f, "NOT RATED"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Song {
    pub id: usize,
    pub artist: String,
    pub title: String,
    pub difficulty: Difficulty,
    pub link: String,
    pub sequence_number: String,
}

#[derive(Debug, Serialize)]
pub struct SongGroup {
    pub artist: String,
    pub songs: Vec<Song>,
}
