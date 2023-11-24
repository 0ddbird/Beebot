use std::collections::HashMap;

use futures::future;
use log::error;
use reqwest;

pub struct Page {
    pub(crate) url: String,
    pub(crate) html: String,
}

async fn fetch_html(url: &str, api_token: &str) -> Result<(String, String), reqwest::Error> {
    let client = reqwest::Client::new();
    let res = client
        .get(url)
        .header("Authorization", api_token)
        .send()
        .await?;

    if res.status().is_success() {
        Ok((url.to_string(), res.text().await?))
    } else {
        Err(res.error_for_status().unwrap_err())
    }
}

pub async fn request_pages(
    api_token: &str,
    urls: &Vec<(&str, String)>,
    is_test_mode: bool,
) -> HashMap<String, Page> {
    if is_test_mode {
        return HashMap::new();
    }

    let futures = urls
        .iter()
        .map(|(key, url)| {
            let token = api_token.to_string();
            let url_clone = url.clone();
            async move { (key.to_string(), fetch_html(&url_clone, &token).await) }
        })
        .collect::<Vec<_>>();

    let results = future::join_all(futures).await;

    let mut html_contents = HashMap::new();

    for (key, result) in results {
        match result {
            Ok((url, html)) => {
                let page = Page { url, html };
                html_contents.insert(key, page);
            }
            Err(e) => {
                error!("Error while fetching pages: {}", e)
            }
        }
    }
    html_contents
}
