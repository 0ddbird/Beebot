use std::string::String;

use serde_json::json;

use crate::validators::{Status, UnitValidationResult};

pub fn compose_mail_body(
    validation_results: &Vec<(UnitValidationResult, String)>,
    is_test_mode: bool,
) -> String {
    let mut message = "".to_string();

    if is_test_mode {
        message.push_str("THIS IS A TEST\n\n");
    }

    for (result, _) in validation_results {
        let status_text = match result.status {
            Status::Ok => "✅",
            Status::Warning => "⚠️",
            Status::Alert => "❌",
        };
        let clean_message = result.message.replace('`', "");
        message.push_str(&format!(
            "{} {}: {}\n",
            status_text, result.name, clean_message,
        ));
    }

    message
}

pub async fn send_mail(
    token: &str,
    sender: &str,
    recipients: Vec<&String>,
    body: &str,
    is_test_mode: bool,
) -> Result<(), reqwest::Error> {
    let test_subject = if is_test_mode {
        "THIS IS A TEST - "
    } else {
        ""
    }
    .to_string();
    let subject = format!("🚨 {} EMERGENCY | Issue with app", test_subject);
    let client = reqwest::Client::new();

    let mut json_recipients = Vec::new();
    for recipient in recipients {
        json_recipients.push(json!({"email": recipient}));
    }

    let res = client
        .post("https://api.sendgrid.com/v3/mail/send")
        .bearer_auth(token)
        .json(&json!({
            "personalizations": [{
                "to": json_recipients,
                "subject": subject
            }],
            "from": {"email": sender},
            "content": [{
                "type": "text/plain",
                "value": body
            }]
        }))
        .send()
        .await?;

    if res.status().is_success() {
        Ok(())
    } else {
        Err(res.error_for_status().unwrap_err())
    }
}
