use anyhow::Result;
use serde_json;

use crate::{group_songs, models::song::Song, output::Formatter};

pub struct JsonFormatter;

impl Formatter for JsonFormatter {
    fn format(&self, songs: &[Song]) -> Result<String> {
        let groups = group_songs(songs.to_vec());
        let json = serde_json::to_string_pretty(&groups)?;
        Ok(json)
    }
}
