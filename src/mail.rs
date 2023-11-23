use crate::validators::{Status, UnitValidationResult};
use serde_json::json;

pub fn compose_mail_body(
    validation_results: &Vec<UnitValidationResult>,
    current_time: &String,
) -> String {
    let mut message = format!("Report Time: {}\n\n", current_time);

    for result in validation_results {
        let status_text = match result.status {
            Status::Ok => "✅",
            Status::Warning => "⚠️",
            Status::Alert => "🔥",
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
) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    let res = client
        .post("https://api.sendgrid.com/v3/mail/send")
        .bearer_auth(token)
        .json(&json!({
            "personalizations": [{
                "to": [{"email": to}],
                "subject": "🆘 BEEBOT ALERT !"
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
