use anyhow::Result;

use crate::models::song::Song;

pub mod html;
pub mod json;
pub mod markdown;

pub trait Formatter {
    fn format(&self, songs: &[Song]) -> Result<String>;
}
