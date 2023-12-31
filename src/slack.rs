use reqwest;
use serde_json::json;

use crate::db::LogEntry;
use crate::validators::{Status, UnitValidationResult, Value};

fn get_corresponding_value(name: &str, log_entry: &LogEntry) -> Value {
    match name {
        "Validated payments" => Value::Count(log_entry.payments as usize),
        "Paid vouchers" => Value::Count(log_entry.vouchers as usize),
        "PDF count" => Value::Count(log_entry.pdf_count as usize),
        "Email count" => Value::Count(log_entry.email_count as usize),
        "Purchase website" => Value::Bool(log_entry.website_ok),
        _ => Value::Count(0),
    }
}

pub fn create_message(
    validation_results: &Vec<(UnitValidationResult, String)>,
    last_log: Option<LogEntry>,
    is_test_mode: bool,
) -> String {
    let mut should_alert_channel = false;
    let mut message = "".to_string();

    if is_test_mode {
        message.push_str("*THIS IS A TEST*\n");
    }

    for (result, url) in validation_results {
        if result.status == Status::Alert {
            should_alert_channel = true;
        }

        let mut trend_icon = String::new();

        if let Some(ref entry) = last_log {
            let last_log_value = get_corresponding_value(&result.name, entry);

            trend_icon = match (last_log_value, &result.value) {
                (Value::Count(last_count), Value::Count(current_count)) => {
                    if current_count > &last_count {
                        ":trend_up:".to_string()
                    } else if current_count < &last_count {
                        ":trend_down:".to_string()
                    } else {
                        ":blank:".to_string()
                    }
                }
                _ => ":blank:".to_string(),
            };
        }

        let status_symbol = match result.status {
            Status::Ok => ":square_check:",
            Status::Warning => ":square_neutral:",
            Status::Alert => ":square_x:",
        };

        let link = format!(" <{}| View >\n", url);

        message.push_str(&format!(
            "{}{} {}: {} {}",
            status_symbol, trend_icon, result.name, result.message, link
        ));
    }

    if should_alert_channel {
        message.push_str(&format!("<!channel>"));
    }

    message
}

pub async fn post_message(token: &str, channel: &str, message: &str) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    let res = client
        .post("https://slack.com/api/chat.postMessage")
        .bearer_auth(token)
        .json(&json!({
            "channel": channel,
            "text": message,
            "unfurl_links": false,
        }))
        .send()
        .await?;

    if res.status().is_success() {
        Ok(())
    } else {
        Err(res.error_for_status().unwrap_err())
    }
}
