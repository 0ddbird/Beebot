use reqwest;
use serde_json::json;

pub async fn send_slack_message(
    token: &str,
    channel: &str,
    message: &str,
) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    let res = client
        .post("https://slack.com/api/chat.postMessage")
        .bearer_auth(token)
        .json(&json!({
            "channel": channel,
            "text": message
        }))
        .send()
        .await?;

    if res.status().is_success() {
        Ok(())
    } else {
        Err(res.error_for_status().unwrap_err())
    }
}
