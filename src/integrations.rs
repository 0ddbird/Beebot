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

pub async fn send_mail(
    sendgrid_token: &str,
    sendgrid_sender: &str,
    sendgrid_recipient: &str,
    body: &str,
) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    let res = client
        .post("https://api.sendgrid.com/v3/mail/send")
        .bearer_auth(sendgrid_token)
        .json(&json!({
            "personalizations": [{
                "to": [{"email": sendgrid_recipient}],
                "subject": "🆘 BEEBOT ALERT !"
            }],
            "from": {"email": sendgrid_sender},
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
