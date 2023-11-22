use std::env;

use chrono::Local;
use dotenv::dotenv;
use tokio;

mod parser;
mod requests;
mod slack_integration;
mod validators;

use crate::slack_integration::send_slack_message;
use crate::validators::{Status, UnitValidationResult};

pub struct PageChecks {
    validated_payments_count: Option<usize>,
    pdf_count: Option<usize>,
    email_check_count: Option<usize>,
    paid_vouchers_count: Option<usize>,
    is_purchase_website_ok: Option<bool>,
}

fn generate_message(validation_results: &Vec<UnitValidationResult>, watcher_id: &str) -> String {
    let current_time = Local::now().format("%H:%M").to_string();
    let mut should_alert_watcher = false;
    let mut message = format!("Report Time: {}\n\n", current_time);

    for result in validation_results {
        if result.status == Status::Alert {
            should_alert_watcher = true;
        }

        let status_symbol = match result.status {
            Status::Ok => ":white_check_mark:",
            Status::Warning => ":warning:",
            Status::Alert => ":fire:",
        };
        message.push_str(&format!(
            "{} {}: {}\n",
            status_symbol, result.name, result.message
        ));
    }

    if should_alert_watcher {
        message.push_str(&format!("<@{}>", watcher_id));
    }

    message
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let slack_token = env::var("SLACK_TOKEN").unwrap();
    let slack_channel = env::var("SLACK_CHANNEL").unwrap();
    let api_token = env::var("API_TOKEN").unwrap();
    let watcher_id = env::var("SLACK_USER_ID").unwrap();

    let urls = vec![
        ("payments", env::var("URL_PAYMENTS").unwrap()),
        ("vouchers", env::var("URL_VOUCHERS").unwrap()),
        ("paid_vouchers", env::var("URL_PAID_VOUCHERS").unwrap()),
        (
            "purchase_website",
            env::var("URL_PURCHASE_WEBSITE").unwrap(),
        ),
    ];

    let pages = requests::fetch_html_pages(&api_token, &urls).await;
    let page_checks = parser::parse_pages(&pages);
    let validation_result = validators::validate_checks(&page_checks);
    let message = generate_message(&validation_result, &watcher_id);
    if let Err(e) = send_slack_message(&slack_token, &slack_channel, &message).await {
        eprintln!("Failed to send message to Slack: {}", e);
    }
}
