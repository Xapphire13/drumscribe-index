use anyhow::{Result, anyhow};
use reqwest::Client;
use serde::Deserialize;

use crate::api::post::Post;

const TRANSCRIPTION_CATEGORY_ID: usize = 73_044;
const API_URL: &str = "https://app.buymeacoffee.com/api/v1/posts/creator/drumscribe?per_page=:per_page&page=:page_number&filter_by=new&category_id=:category_id";

#[derive(Debug, Deserialize)]
pub struct PageMeta {
    pub current_page: usize,
    pub last_page: usize,
}

#[derive(Debug, Deserialize)]
pub struct PageResponse<T> {
    pub data: Vec<T>,
    pub meta: PageMeta,
}

pub struct CoffeeApi {
    client: Client,
}

impl CoffeeApi {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn get_posts(&self, page_number: usize, per_page: usize) -> Result<PageResponse<Post>> {
        self.client
            .get(CoffeeApi::get_request_url(page_number, per_page))
            .send()
            .await?
            .json::<PageResponse<Post>>()
            .await
            .map_err(|e| anyhow!("Failed to deserialize posts from page {page_number}: {e}"))
    }

    fn get_request_url(page: usize, per_page: usize) -> String {
        API_URL
            .replace(":page_number", &page.to_string())
            .replace(":per_page", &per_page.to_string())
            .replace(":category_id", &TRANSCRIPTION_CATEGORY_ID.to_string())
    }
}
