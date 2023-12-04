use std::collections::HashMap;

use futures::future;
use http_auth_basic::Credentials;
use log::error;
use reqwest;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};

pub struct Page {
    pub(crate) url: String,
    pub(crate) html: String,
}

async fn fetch_html(
    key: &str,
    url: &str,
    api_token: &str,
    celery_username: &str,
    celery_password: &str,
) -> Result<(String, String), reqwest::Error> {
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();

    if key == "celery" {
        headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
        let credentials = Credentials::new(celery_username, celery_password);
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&credentials.as_http_header()).unwrap(),
        );
    } else {
        let auth_value = HeaderValue::from_str(&format!("{}", api_token)).unwrap();
        headers.insert(AUTHORIZATION, auth_value);
    }

    let res = client.get(url).headers(headers).send().await?;

    if res.status().is_success() {
        Ok((url.to_string(), res.text().await?))
    } else {
        Err(res.error_for_status().unwrap_err())
    }
}

pub async fn request_pages(
    api_token: &str,
    urls: &Vec<(&str, String)>,
    celery_username: &str,
    celery_password: &str,
    is_test_mode: bool,
) -> HashMap<String, Page> {
    if is_test_mode {
        return HashMap::new();
    }

    let futures = urls
        .iter()
        .map(|(key, url)| async move {
            (
                key.to_string(),
                fetch_html(key, &url, &api_token, celery_username, celery_password).await,
            )
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
