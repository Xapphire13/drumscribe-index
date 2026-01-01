use anyhow::Result;

use crate::{group_songs, models::song::Song, output::Formatter};
use std::fmt::Write;

pub struct MarkdownFormatter;

impl Formatter for MarkdownFormatter {
    fn format(&self, songs: &[Song]) -> Result<String> {
        let groups = group_songs(songs.to_vec());

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
