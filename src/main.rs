use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Result, anyhow};
use clap::Parser;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::{
    api::{coffee_api::CoffeeApi, post::Post},
    models::song::{Song, SongGroup},
    output::{Formatter, html::HtmlFormatter, json::JsonFormatter, markdown::MarkdownFormatter},
};

mod api;
mod conversions;
mod corrections;
mod models;
mod output;

#[derive(Parser)]
#[command(name = "drumscribe-index")]
#[command(about = "DrumScribe song index generator")]
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
}

const INDEX_CACHE_FILENAME: &str = "index.bin";

#[derive(Debug, Deserialize)]
struct PageMeta {
    current_page: usize,
    last_page: usize,
}

#[derive(Debug, Deserialize)]
struct PageResponse<T> {
    data: Vec<T>,
    meta: PageMeta,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct IndexCache {
    #[serde(skip)]
    path: PathBuf,
    songs: Vec<Song>,
}

impl IndexCache {
    fn load(data_dir: &Path) -> Result<Self> {
        let index_cache_path = data_dir.join(INDEX_CACHE_FILENAME);

        let index_cache = if let Ok(bytes) = fs::read(&index_cache_path) {
            postcard::from_bytes::<IndexCache>(&bytes)?
        } else {
            IndexCache::default()
        };

        Ok(IndexCache {
            path: index_cache_path,
            ..index_cache
        })
    }

    fn save(&self) -> Result<()> {
        let bytes = postcard::to_allocvec(self)?;
        fs::write(&self.path, &bytes)?;

        Ok(())
    }

    fn is_empty(&self) -> bool {
        self.songs.is_empty()
    }
}

fn create_data_dir() -> Result<PathBuf> {
    let project_dirs = ProjectDirs::from("com", "xapphire13", env!("CARGO_PKG_NAME"))
        .ok_or(anyhow!("Can't load project dirs"))?;

    let data_dir = project_dirs.data_dir();
    fs::create_dir_all(data_dir)?;

    Ok(data_dir.to_path_buf())
}

fn group_songs(songs: Vec<Song>) -> Vec<SongGroup> {
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
            .push(song);
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

    if index_cache.is_empty() {
        let mut page_number = 1;
        loop {
            print!("Fetching page {page_number}...");
            let response: PageResponse<Post> = coffee_api.get_posts(page_number).await?;
            let page: Vec<_> = response.data.iter().flat_map(Song::try_from).collect();

            index_cache.songs.extend(page);

            println!(" done!");

            if response.meta.current_page == response.meta.last_page {
                break;
            }

            page_number += 1;
        }

        index_cache.save()?;
    }

    let formatter: Box<dyn Formatter> = if cli.markdown {
        Box::new(MarkdownFormatter)
    } else if cli.html {
        Box::new(HtmlFormatter)
    } else {
        // Default to JSON
        Box::new(JsonFormatter)
    };

    print!("{}", formatter.format(&index_cache.songs)?);

    Ok(())
}
