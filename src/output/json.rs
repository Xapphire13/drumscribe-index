use anyhow::Result;
use serde_json;

use crate::{group_songs, models::song::Song};

pub struct JsonFormatter;

impl JsonFormatter {
    pub fn format(songs: &[Song]) -> Result<String> {
        let groups = group_songs(songs);
        let json = serde_json::to_string_pretty(&groups)?;
        Ok(json)
    }
}
