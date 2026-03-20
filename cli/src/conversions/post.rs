use std::str::FromStr;

use anyhow::{Error, anyhow};

use crate::{
    api::post::{Post as ApiPost, Tag},
    corrections::correct_artist,
    models::song::{Difficulty, Song},
};

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
        let (title, remainder) = s.split_once(" - ").unwrap_or((s, ""));
        let (artist, sequence_part) = remainder.split_once(" | ").unwrap_or((remainder, ""));

        let sequence_number = sequence_part
            .trim()
            .strip_prefix('#')
            .and_then(|s| s.split_whitespace().next())
            .unwrap_or("");

        Ok(Self {
            artist: correct_artist(artist.trim()).to_owned(),
            title: title.trim().to_owned(),
            sequence_number: sequence_number.to_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_song_details_full_format() {
        // Format: "Title - Artist | #123"
        let details: SongDetails = "Everlong - Foo Fighters | #42 DRUMSCRIBE".parse().unwrap();
        assert_eq!(details.title, "Everlong");
        assert_eq!(details.artist, "Foo Fighters");
        assert_eq!(details.sequence_number, "42");
    }

    #[test]
    fn test_song_details_with_whitespace() {
        let details: SongDetails = "  Everlong  -  Foo Fighters  |  #42  ".parse().unwrap();
        assert_eq!(details.title, "Everlong");
        assert_eq!(details.artist, "Foo Fighters");
        assert_eq!(details.sequence_number, "42");
    }

    #[test]
    fn test_song_details_no_sequence_number() {
        let details: SongDetails = "Everlong - Foo Fighters | ".parse().unwrap();
        assert_eq!(details.title, "Everlong");
        assert_eq!(details.artist, "Foo Fighters");
        assert_eq!(details.sequence_number, "");
    }

    #[test]
    fn test_song_details_no_artist_or_sequence() {
        let details: SongDetails = "Everlong - ".parse().unwrap();
        assert_eq!(details.title, "Everlong");
        assert_eq!(details.artist, "");
        assert_eq!(details.sequence_number, "");
    }

    #[test]
    fn test_song_details_only_title() {
        let details: SongDetails = "Everlong".parse().unwrap();
        assert_eq!(details.title, "Everlong");
        assert_eq!(details.artist, "");
        assert_eq!(details.sequence_number, "");
    }

    #[test]
    fn test_song_details_sequence_with_trailing_bar() {
        // Sequence number should stop at first whitespace/non-digit
        let details: SongDetails = "Everlong - Foo Fighters | #42 | DRUMSCRIBE"
            .parse()
            .unwrap();
        assert_eq!(details.title, "Everlong");
        assert_eq!(details.artist, "Foo Fighters");
        assert_eq!(details.sequence_number, "42");
    }

    #[test]
    fn test_song_details_no_hash_prefix() {
        let details: SongDetails = "Everlong - Foo Fighters | 42".parse().unwrap();
        assert_eq!(details.title, "Everlong");
        assert_eq!(details.artist, "Foo Fighters");
        assert_eq!(details.sequence_number, "");
    }

    #[test]
    fn test_song_details_with_hyphenated_artist_name() {
        // Sequence number should stop at first whitespace/non-digit
        let details: SongDetails = "This Christmas Day - Trans-Siberian Orchestra | #622"
            .parse()
            .unwrap();
        assert_eq!(details.title, "This Christmas Day");
        assert_eq!(details.artist, "Trans-Siberian Orchestra");
        assert_eq!(details.sequence_number, "622");
    }
}
