use anyhow::Result;
use serde::Deserialize;

use crate::models::post::{Category, Post, Tag};

mod api;
mod conversions;
mod models;

const API_URL: &str = "https://app.buymeacoffee.com/api/v1/posts/creator/drumscribe?per_page=20&page=:page_number&filter_by=new";
const EXHAUSTIVE: bool = false;

fn get_request_url(page: usize) -> String {
    API_URL.replace(":page_number", &page.to_string())
}

#[derive(Debug, Deserialize)]
struct PageMeta {
    current_page: usize,
    last_page: usize,
    from: usize,
    to: usize,
    total: usize,
}

#[derive(Debug, Deserialize)]
struct PageResponse<T> {
    data: Vec<T>,
    meta: PageMeta,
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut transcriptions: Vec<_> = vec![];

    let mut page_number = 1;
    loop {
        print!("Fetching page {page_number}...");
        let response = reqwest::get(get_request_url(page_number)).await?;
        let page: PageResponse<api::post::Post> = response.json().await?;

        let posts: Vec<_> = page
            .data
            .iter()
            .map(Post::from)
            .filter(|post| {
                post.tags
                    .iter()
                    .any(|tag| matches!(tag, Tag::Category(Category::Transcription)))
            })
            .collect();

        transcriptions.extend(posts);

        println!(" done!");

        if !EXHAUSTIVE || page.meta.current_page == page.meta.last_page {
            break;
        }

        page_number += 1;
    }

    println!("{transcriptions:#?}");

    Ok(())
}
