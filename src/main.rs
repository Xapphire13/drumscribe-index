use std::{
    collections::HashMap,
    fs,
    io::{self, Write},
    path::PathBuf,
};

use anyhow::{Result, anyhow};
use chrono::Utc;
use clap::Parser;
use directories::ProjectDirs;

use crate::{
    api::{
        coffee_api::{CoffeeApi, PageResponse},
        post::Post,
    },
    index_cache::IndexCache,
    models::song::{Song, SongGroup},
    output::{
        html::HtmlFormatter, json::JsonFormatter, markdown::MarkdownFormatter, xlsx::XlsxFormatter,
    },
};

mod api;
mod conversions;
mod corrections;
mod index_cache;
mod models;
mod output;

#[derive(Parser)]
#[command(name = "drumscribe-index")]
#[command(about = "DrumScribe song index generator")]
#[allow(clippy::struct_excessive_bools)]
struct Cli {
    /// Output in JSON format (default)
    #[arg(long, group = "format")]
    json: bool,

    /// Output in Markdown format
    #[arg(long, group = "format")]
    markdown: bool,

    /// Output in HTML format
    #[arg(long, group = "format")]
    html: bool,

    /// Output in XLSX format to the specified file path
    #[arg(long, group = "format", requires = "output")]
    xlsx: bool,

    /// Saves output to specified file path
    #[arg(long, value_name = "PATH")]
    output: Option<String>,

    /// Update list of indexed songs
    #[arg(long)]
    update: bool,
}

fn create_data_dir() -> Result<PathBuf> {
    let project_dirs = ProjectDirs::from("com", "xapphire13", env!("CARGO_PKG_NAME"))
        .ok_or(anyhow!("Can't load project dirs"))?;

    let data_dir = project_dirs.data_dir();
    fs::create_dir_all(data_dir)?;

    Ok(data_dir.to_path_buf())
}

fn group_songs(songs: &[Song]) -> Vec<SongGroup> {
    let mut groups = HashMap::new();

    for song in songs {
        if song.sequence_number.is_empty() {
            continue;
        }

        let key = song.artist.to_lowercase();
        groups
            .entry(key)
            .or_insert(SongGroup {
                artist: song.artist.clone(),
                songs: vec![],
            })
            .songs
            .push(song.clone());
    }

    let mut result: Vec<_> = groups.into_values().collect();
    result.sort_unstable_by_key(|group| group.artist.to_lowercase());

    for group in &mut result {
        group
            .songs
            .sort_unstable_by_key(|song| song.title.to_lowercase());

        group.songs.dedup_by_key(|song| song.title.to_lowercase());
    }

    result
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let data_dir = create_data_dir()?;
    let mut index_cache = IndexCache::load(&data_dir)?;
    let coffee_api = CoffeeApi::new();

    if index_cache.is_empty() || cli.update {
        let highest_sequence_number = index_cache
            .songs
            .iter()
            .flat_map(|s| s.sequence_number.parse::<usize>())
            .max();
        let mut page_number = 1;

        loop {
            print!("Fetching page {page_number}...");
            io::stdout().flush()?;
            let response: PageResponse<Post> = coffee_api.get_posts(page_number).await?;
            let page: Vec<_> = response.data.iter().flat_map(Song::try_from).collect();

            let mut reached_existing_content = false;

            // Only add songs we havent already indexed
            let new_songs = page.into_iter().filter(|s| {
                if let Some(highest_sequence_number) = highest_sequence_number
                    && let Ok(sequence_number) = s.sequence_number.parse::<usize>()
                {
                    let is_new = sequence_number > highest_sequence_number;

                    // We've caught up to our index
                    if !is_new {
                        reached_existing_content = true;
                    }

                    is_new
                } else {
                    true
                }
            });

            index_cache.songs.extend(new_songs);

            println!(" done!");

            if reached_existing_content || response.meta.current_page == response.meta.last_page {
                break;
            }

            page_number += 1;
        }

        index_cache.last_indexed = Utc::now();
        index_cache.save()?;
    }

    if cli.xlsx
        && let Some(output_path) = cli.output
    {
        // XLSX format writes to a file instead of returning text-based result
        XlsxFormatter::format_to_file(&index_cache.songs, &output_path)?;
        println!("XLSX file saved to: {output_path}");
    } else {
        let file_type;

        // Text-based formats
        let formatted = if cli.markdown {
            file_type = "Markdown";
            MarkdownFormatter::format(&index_cache.songs)?
        } else if cli.html {
            file_type = "HTML";
            HtmlFormatter::format(&index_cache.songs)
        } else {
            // Default to JSON
            file_type = "JSON";
            JsonFormatter::format(&index_cache.songs)?
        };

        if let Some(output_path) = cli.output {
            fs::write(&output_path, formatted)?;
            println!("{file_type} file saved to: {output_path}");
        } else {
            print!("{formatted}");
        }
    }

    Ok(())
}
