use anyhow::Result;
use std::fmt::Write;

use crate::{group_songs, models::song::Song, output::Formatter};

pub struct HtmlFormatter;

impl Formatter for HtmlFormatter {
    fn format(&self, songs: &[Song]) -> Result<String> {
        let groups = group_songs(songs.to_vec());

        let mut result = String::new();

        writeln!(result, "TODO")?;

        Ok(result)
    }
}
