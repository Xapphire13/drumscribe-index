use std::{str::FromStr, sync::OnceLock};

use anyhow::{Error, anyhow};
use regex::Regex;

use crate::{
    api::post::{Post as ApiPost, Tag},
    corrections::correct_artist,
    models::song::{Difficulty, Song},
};

static SEQUENCE_NUMBER_REGEX: OnceLock<Regex> = OnceLock::new();
const TRANSCRIPTION_CATEGORY_ID: usize = 73_044;

impl From<&Vec<Tag>> for Difficulty {
    fn from(value: &Vec<Tag>) -> Self {
        value
            .iter()
            .flat_map(|tag| match tag {
                Tag::Category { category_id } => match category_id {
                    174_260 => Ok(Difficulty::Beginner),
                    174_255 => Ok(Difficulty::Intermediate),
                    174_257 => Ok(Difficulty::Advanced),
                    174_258 => Ok(Difficulty::Expert),
                    174_259 => Ok(Difficulty::Master),
                    _ => Err(anyhow!("Not a difficulty")),
                },
                Tag::Other => Err(anyhow!("Not a category")),
            })
            .next()
            .unwrap_or(Difficulty::Unrated)
    }
}

impl TryFrom<&ApiPost> for Song {
    type Error = Error;

    fn try_from(value: &ApiPost) -> Result<Self, Self::Error> {
        if !value.tags.iter().any(|tag| {
            if let Tag::Category { category_id } = tag
                && *category_id == TRANSCRIPTION_CATEGORY_ID
            {
                true
            } else {
                false
            }
        }) {
            return Err(anyhow!("Only transcriptions can be converted into songs"));
        }

        let song_details: SongDetails = value.project_update_heading.parse()?;

        Ok(Song {
            id: value.id,
            artist: song_details.artist,
            title: song_details.title,
            sequence_number: song_details.sequence_number,
            link: value.share_urls.copy_url.clone(),
            difficulty: (&value.tags).into(),
        })
    }
}

struct SongDetails {
    artist: String,
    title: String,
    sequence_number: String,
}

impl FromStr for SongDetails {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split(" - ");
        let title = split.next().unwrap_or("Unknown").trim();
        split = split.next().unwrap_or("").split(" | ");
        let artist = split.next().unwrap_or("Unknown").trim();
        let re = SEQUENCE_NUMBER_REGEX.get_or_init(|| Regex::new(r"#(\d+)").unwrap());
        let sequence_number = if let Some(captures) = re.captures(split.next().unwrap_or("")) {
            captures.get(1).unwrap().as_str()
        } else {
            ""
        };

        Ok(Self {
            artist: correct_artist(artist).to_owned(),
            title: title.to_owned(),
            sequence_number: sequence_number.to_owned(),
        })
    }
}
