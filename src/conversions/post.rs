use anyhow::anyhow;

use crate::{
    api::post::{Post as ApiPost, Tag as ApiTag},
    models::post::{Category, Difficulty, Post, Tag},
};

impl TryFrom<&ApiTag> for Category {
    type Error = anyhow::Error;

    fn try_from(value: &ApiTag) -> Result<Self, Self::Error> {
        match value {
            ApiTag::Category { category_id } => Ok(match category_id {
                73044 => Category::Transcription,
                174260 => Category::Difficulty(Difficulty::Beginner),
                174255 => Category::Difficulty(Difficulty::Intermediate),
                174257 => Category::Difficulty(Difficulty::Advanced),
                174258 => Category::Difficulty(Difficulty::Expert),
                174259 => Category::Difficulty(Difficulty::Master),
                _ => Category::Other,
            }),
            _ => Err(anyhow!("Not a category")),
        }
    }
}

impl From<&ApiPost> for Post {
    fn from(value: &ApiPost) -> Self {
        Post {
            id: value.id,
            artist: String::from("Unknown"), // TODO
            title: value.project_update_heading.clone(),
            link: String::from("Unknown"), // TODO
            tags: value
                .tags
                .iter()
                .map(|tag| {
                    if let Ok(category) = Category::try_from(tag) {
                        Tag::Category(category)
                    } else {
                        Tag::Other
                    }
                })
                .collect(),
        }
    }
}
