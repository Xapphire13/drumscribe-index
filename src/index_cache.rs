use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::song::Song;

const INDEX_CACHE_FILENAME: &str = "index.bin";

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct IndexCache {
    #[serde(skip)]
    path: PathBuf,
    pub songs: Vec<Song>,
    pub last_indexed: DateTime<Utc>,
}

impl IndexCache {
    pub fn load(data_dir: &Path) -> Self {
        let index_cache_path = data_dir.join(INDEX_CACHE_FILENAME);

        if let Ok(bytes) = fs::read(&index_cache_path) {
            IndexCache {
                path: index_cache_path,
                ..postcard::from_bytes(&bytes).unwrap_or(IndexCache::default())
            }
        } else {
            IndexCache {
                path: index_cache_path,
                ..IndexCache::default()
            }
        }
    }

    pub fn save(&self) -> Result<()> {
        let bytes = postcard::to_allocvec(self)?;
        fs::write(&self.path, &bytes)?;

        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.songs.is_empty()
    }
}
