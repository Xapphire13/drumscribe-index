use anyhow::Result;

use crate::{group_songs, models::song::Song};
use std::fmt::Write;

pub struct MarkdownFormatter;

impl MarkdownFormatter {
    pub fn format(&self, songs: &Vec<Song>) -> Result<String> {
        let groups = group_songs(songs.clone());

        let mut result = String::new();

        for group in &groups {
            writeln!(result, "# {}", group.artist)?;
            for song in &group.songs {
                writeln!(
                    result,
                    "- {} | #{} | {}",
                    song.title, song.sequence_number, song.difficulty
                )?;
            }

            writeln!(result, "")?;
        }

        Ok(result)
    }
}
