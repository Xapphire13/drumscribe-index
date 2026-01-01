use anyhow::{Error, Result};
use reqwest::Client;

use crate::{PageResponse, api::post::Post};

const API_URL: &str = "https://app.buymeacoffee.com/api/v1/posts/creator/drumscribe?per_page=20&page=:page_number&filter_by=new";

pub struct CoffeeApi {
    client: Client,
}

impl CoffeeApi {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn get_posts(&self, page_number: usize) -> Result<PageResponse<Post>> {
        self.client
            .get(CoffeeApi::get_request_url(page_number))
            .send()
            .await?
            .json::<PageResponse<Post>>()
            .await
            .map_err(Error::msg)
    }

    fn get_request_url(page: usize) -> String {
        API_URL.replace(":page_number", &page.to_string())
    }
}
