use std::env;

use chrono::Local;
use dotenv::dotenv;
use log::info;
use simplelog::*;
use std::fs::File;
use tokio;

mod integrations;
mod parser;
mod requests;
mod validators;

use crate::integrations::{send_mail, send_slack_message};
use crate::validators::{Status, UnitValidationResult};

pub struct PageChecks {
    validated_payments_count: Option<usize>,
    pdf_count: Option<usize>,
    email_check_count: Option<usize>,
    paid_vouchers_count: Option<usize>,
    is_purchase_website_ok: Option<bool>,
}

fn generate_slack_message(
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
        message.push_str(&format!(
            "{} {}: {}\n",
            status_symbol, result.name, result.message
        ));
    }

    if should_alert_channel {
        message.push_str(&format!("<!channel>"));
    }

    message
}

fn generate_mail_content(
    validation_results: &Vec<UnitValidationResult>,
    current_time: &String,
) -> String {
    let mut message = format!("Report Time: {}\n\n", current_time);

    for result in validation_results {
        let status_text = match result.status {
            Status::Ok => "OK",
            Status::Warning => "Warning",
            Status::Alert => "Alert",
        };
        message.push_str(&format!(
            "{} - {}: {}\n",
            status_text, result.name, result.message
        ));
    }

    message
}

#[tokio::main]
async fn main() {
    let _ = WriteLogger::init(
        LevelFilter::Info,
        Config::default(),
        File::create("my_log.log").unwrap(),
    );
    info!("Beebot starting");

    dotenv().ok();
    let current_time = Local::now().format("%H:%M:%S").to_string();
    let api_token = env::var("API_TOKEN").unwrap();

    let slack_token = env::var("SLACK_API_TOKEN").unwrap();
    let slack_channel = env::var("SLACK_CHANNEL").unwrap();

    let sendgrid_token = env::var("SENDGRID_API_TOKEN").unwrap();
    let sendgrid_recipient = env::var("SENDGRID_RECIPIENT").unwrap();
    let sendgrid_sender = env::var("SENDGRID_SENDER").unwrap();

    let urls = vec![
        ("payments", env::var("URL_PAYMENTS").unwrap()),
        ("vouchers", env::var("URL_VOUCHERS").unwrap()),
        ("paid_vouchers", env::var("URL_PAID_VOUCHERS").unwrap()),
        (
            "purchase_website",
            env::var("URL_PURCHASE_WEBSITE").unwrap(),
        ),
    ];
    info!("Fetching pages content");
    let pages = requests::fetch_html_pages(&api_token, &urls).await;
    let page_checks = parser::parse_pages(&pages);

    info!("Validating results");
    let validation_result = validators::validate_checks(&page_checks);

    info!("Sending Slack message");
    let slack_message = generate_slack_message(&validation_result, &current_time);
    if let Err(e) = send_slack_message(&slack_token, &slack_channel, &slack_message).await {
        eprintln!("Failed to send message to Slack: {}", e);
    }

    if validation_result
        .iter()
        .any(|result| result.status == Status::Alert)
    {
        info!("Sending alert email");
        let mail_body = generate_mail_content(&validation_result, &current_time);

        info!("Mail content: {}", mail_body);
        if let Err(e) = send_mail(
            &sendgrid_token,
            &sendgrid_sender,
            &sendgrid_recipient,
            &mail_body,
        )
        .await
        {
            eprintln!("Failed to send email: {}", e);
        }
    }
}
