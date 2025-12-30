use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Tag {
    #[serde()]
    Category {
        category_id: usize,
    },
    Other,
}

#[derive(Debug, Deserialize)]
pub struct Post {
    pub id: usize,
    pub project_update_heading: String,
    pub tags: Vec<Tag>,
}
