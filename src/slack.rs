use crate::validators::{Status, UnitValidationResult};
use reqwest;
use serde_json::json;

pub fn compose_slack_message(
    validation_results: &Vec<UnitValidationResult>,
    current_time: &String,
) -> String {
    let mut should_alert_channel = false;
    let mut message = format!("*Report Time: {}*\n\n", current_time);

    for result in validation_results {
        if result.status == Status::Alert {
            should_alert_channel = true;
        }

        let status_symbol = match result.status {
            Status::Ok => ":white_check_mark:",
            Status::Warning => ":warning:",
            Status::Alert => ":fire:",
        };
        // Escaping "<" character for Slack
        let formatted_message = result.message.replace("<", "&lt;");
        message.push_str(&format!(
            "{} {}: {}\n",
            status_symbol, result.name, formatted_message
        ));
    }

    if should_alert_channel {
        message.push_str(&format!("<!channel>"));
    }

    message
}

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
