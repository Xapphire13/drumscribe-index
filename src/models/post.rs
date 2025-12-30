#[derive(Debug)]
pub enum Difficulty {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
    Master,
}

#[derive(Debug)]
pub enum Category {
    Transcription,
    Difficulty(Difficulty),
    Other,
}

#[derive(Debug)]
pub enum Tag {
    Category(Category),
    Other,
}

#[derive(Debug)]
pub struct Post {
    pub id: usize,
    pub artist: String,
    pub title: String,
    pub tags: Vec<Tag>,
    pub link: String,
}
