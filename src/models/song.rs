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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Song {
    pub id: usize,
    pub artist: String,
    pub title: String,
    pub difficulty: Difficulty,
    pub link: String,
    pub sequence_number: String,
}

#[derive(Debug)]
pub struct SongGroup {
    pub artist: String,
    pub songs: Vec<Song>,
}
