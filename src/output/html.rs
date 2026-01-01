use anyhow::Result;
use maud::{DOCTYPE, html};

use crate::{group_songs, models::song::Song, output::Formatter};

pub struct HtmlFormatter;

impl Formatter for HtmlFormatter {
    fn format(&self, songs: &[Song]) -> Result<String> {
        let groups = group_songs(songs);

        let markup = html! {
            (DOCTYPE)
            html {
                head {
                    meta charset="UTF-8";
                    meta name="viewport" content="width=device-width, initial-scale=1.0";
                    title { "DrumScribe Index" }
                    style {
                        "@page { size: letter portrait; margin: 0.5in; }"
                        "* { box-sizing: border-box; }"
                        "body { font-family: Arial, sans-serif; margin: 0 auto; padding: 20px; background-color: white; max-width: 800px; }"
                        ".item { break-inside: avoid; -webkit-column-break-inside: avoid; page-break-inside: avoid; margin-bottom: 0.5em; }"
                        ".artist-header { color: #333; border-bottom: 2px solid #007bff; padding-bottom: 6px; margin: 0.5em 0 8px 0; font-size: 1.1em; font-weight: bold; break-after: avoid; page-break-after: avoid; }"
                        ".artist-header:first-child { margin-top: 0; }"
                        ".song-item { background: white; padding: 6px 8px; border-radius: 3px; border: 1px solid #e0e0e0; font-size: 0.85em; margin-bottom: 4px; }"
                        ".song-title { font-weight: bold; color: #007bff; }"
                        ".song-number { color: #666; }"
                        ".song-difficulty { color: #28a745; font-style: italic; }"
                        "@media print { body { margin: 0; padding: 0; max-width: none; column-count: 2; column-gap: 1em; column-fill: auto; orphans: 2; widows: 2; } * { -webkit-print-color-adjust: exact; print-color-adjust: exact; } }"
                    }
                }
                body {
                    @for group in &groups {
                        div.artist-group {
                            div.item.artist-header { (group.artist) }
                            @for song in &group.songs {
                                div.item.song-item {
                                    span.song-title { (song.title) }
                                    " | "
                                    span.song-number { "#" (song.sequence_number) }
                                    " | "
                                    span.song-difficulty { (song.difficulty) }
                                }
                            }
                        }
                    }
                }
            }
        };

        Ok(markup.into_string())
    }
}
