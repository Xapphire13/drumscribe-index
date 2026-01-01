use anyhow::Result;
use maud::{DOCTYPE, PreEscaped, html};

use crate::{
    group_songs,
    models::song::{Difficulty, Song},
    output::Formatter,
};

const STYLES: &str = include_str!("styles.css");

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
                        (PreEscaped(STYLES))
                    }
                }
                body {
                    @for group in &groups {
                        div.artist-group {
                            div.artist-header { (group.artist) }
                            table.song-table {
                                @for song in &group.songs {
                                    @let stars = match song.difficulty {
                                        Difficulty::Beginner => "★",
                                        Difficulty::Intermediate => "★★",
                                        Difficulty::Advanced => "★★★",
                                        Difficulty::Expert => "★★★★",
                                        Difficulty::Master => "★★★★★",
                                        Difficulty::Unrated => "—",
                                    };
                                    tr.item.song-item {
                                        td.song-title { (song.title) }
                                        td.song-number { "#" (song.sequence_number) }
                                        td.song-difficulty { (stars) }
                                    }
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
