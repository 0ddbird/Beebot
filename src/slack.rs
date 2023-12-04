use reqwest;
use serde_json::{json, Value};

use crate::db::LogEntry;
use crate::validators::{Status, UnitValidationResult, ValidationValue};

fn get_corresponding_value(name: &str, log_entry: &LogEntry) -> ValidationValue {
    match name {
        "Validated payments" => ValidationValue::Count(log_entry.payments as usize),
        "Paid vouchers" => ValidationValue::Count(log_entry.vouchers as usize),
        "PDF count" => ValidationValue::Count(log_entry.pdf_count as usize),
        "Email count" => ValidationValue::Count(log_entry.email_count as usize),
        "Purchase website" => ValidationValue::Bool(log_entry.website_ok),
        _ => ValidationValue::Count(0),
    }
}

fn get_trend_icon(
    last_log_value: Option<&ValidationValue>,
    current_value: &ValidationValue,
) -> String {
    match (last_log_value, current_value) {
        (Some(ValidationValue::Count(last_count)), ValidationValue::Count(current_count)) => {
            if current_count > last_count {
                ":trend_up:".to_string()
            } else if current_count < last_count {
                ":trend_down:".to_string()
            } else {
                ":blank:".to_string()
            }
        }
        _ => ":blank:".to_string(),
    }
}

fn get_status_icon(status: &Status) -> &'static str {
    match status {
        Status::Ok => ":square_check:",
        Status::Warning => ":square_neutral:",
        Status::Alert => ":square_x:",
    }
}

pub fn create_message(
    validation_results: &Vec<(UnitValidationResult, String)>,
    last_log: Option<LogEntry>,
    is_test_mode: bool,
) -> Value {
    let mut blocks = Vec::new();

    if is_test_mode {
        blocks.push(json!({
            "type": "section",
            "text": {
                "type": "mrkdwn",
                "text": "*THIS IS A TEST*"
            }
        }));
    }

    for (result, url) in validation_results {
        let last_log_value = last_log
            .as_ref()
            .map(|entry| get_corresponding_value(&result.name, entry));
        let trend_icon = get_trend_icon(last_log_value.as_ref(), &result.value);
        let status_symbol = get_status_icon(&result.status);

        let header_text = format!("*{} {}*", result.name, trend_icon);

        let detail_text = format!("{} {}", status_symbol, result.message);

        blocks.push(json!({
            "type": "section",
            "text": {
                "type": "mrkdwn",
                "text": header_text
            },
        }));

        blocks.push(json!({
            "type": "section",
            "text": {
                "type": "mrkdwn",
                "text": detail_text
            },
            "accessory": {
                "type": "button",
                "text": {
                    "type": "plain_text",
                    "text": "View",
                    "emoji": true
                },
                "value": "view",
                "url": url
            }
        }));
        blocks.push(json!({
            "type": "divider"
        }
        ));
    }

    json!({ "blocks": blocks })
}

pub async fn post_message(
    token: &str,
    channel: &str,
    blocks: &Value,
) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    let mut payload = blocks.as_object().unwrap().clone();
    payload.insert("channel".to_string(), json!(channel));
    payload.insert("unfurl_links".to_string(), json!(false));
    let res = client
        .post("https://slack.com/api/chat.postMessage")
        .bearer_auth(token)
        .json(&payload)
        .send()
        .await?;

    if res.status().is_success() {
        Ok(())
    } else {
        Err(res.error_for_status().unwrap_err())
    }
}
