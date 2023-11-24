use crate::validators::{Status, UnitValidationResult};
use serde_json::json;
use std::string::String;

pub fn compose_mail_body(
    validation_results: &Vec<UnitValidationResult>,
    is_test_mode: bool,
) -> String {
    let mut message = "".to_string();

    if is_test_mode {
        message.push_str("THIS IS A TEST\n");
    }

    for result in validation_results {
        let status_text = match result.status {
            Status::Ok => "✅",
            Status::Warning => "⚠️",
            Status::Alert => "❌",
        };
        message.push_str(&format!(
            "{} {}: {}\n",
            status_text, result.name, result.message
        ));
    }

    message
}

pub async fn send_mail(
    token: &str,
    from: &str,
    to: &str,
    body: &str,
    is_test_mode: bool,
) -> Result<(), reqwest::Error> {
    let test_subject = if is_test_mode {
        "THIS IS A TEST - "
    } else {
        ""
    }
    .to_string();
    let subject = format!("🚨 {} EMERGENCY | Issue with MBB app", test_subject);

    let client = reqwest::Client::new();
    let res = client
        .post("https://api.sendgrid.com/v3/mail/send")
        .bearer_auth(token)
        .json(&json!({
            "personalizations": [{
                "to": [{"email": to}],
                "subject": subject
            }],
            "from": {"email": from},
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
