use std::env;

use chrono::Local;
use dotenv::dotenv;
use log::info;
use mail::send_mail;
use simplelog::*;
use std::fs::{self, File};
use tokio;

extern crate diesel;

use crate::db::{establish_connection, get_last_entry, insert_results, LogEntry};
use crate::parser::EmailStatus;
use crate::slack::send_slack_message;
use crate::validators::Status;

mod db;
mod mail;
mod parser;
mod requests;
mod schema;
mod slack;
mod validators;

pub struct PageResults {
    validated_payments_count: Option<usize>,
    pdf_count: Option<usize>,
    email_check_count: Option<EmailStatus>,
    paid_vouchers_count: Option<usize>,
    is_purchase_website_ok: Option<bool>,
}

#[tokio::main]
async fn main() {
    fs::create_dir_all("logs").expect("Failed to create logs directory");
    let log_file_name = format!("logs/{}.log", Local::now().format("%Y_%m_%d_%H-%M-%S"));
    let _ = WriteLogger::init(
        LevelFilter::Info,
        Config::default(),
        File::create(log_file_name).unwrap(),
    );
    dotenv().ok();

    info!("Beebot starting");

    let db_url = env::var("DATABASE_URL").unwrap();
    let api_token = env::var("API_TOKEN").unwrap();
    let slack_token = env::var("SLACK_API_TOKEN").unwrap();
    let slack_channel = env::var("SLACK_CHANNEL").unwrap();
    let mail_token = env::var("SENDGRID_API_TOKEN").unwrap();
    let mail_sender = env::var("SENDGRID_SENDER").unwrap();
    let mail_recipient = env::var("SENDGRID_RECIPIENT").unwrap();

    info!("Connecting to db");
    let mut conn = establish_connection(&db_url);

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
    let pages = requests::request_pages(&api_token, &urls).await;
    let page_results = parser::parse_pages(&pages);

    info!("Validating results");
    let validation_results = validators::validate(&page_results);
    let current_time = Local::now().format("%H:%M").to_string();

    let mut last_entry: Option<LogEntry> = None;

    if let Ok(ref mut conn) = conn {
        last_entry = match get_last_entry(conn) {
            Ok(entry) => {
                info!("Fetched previous log in DB for comparison");
                Some(entry)
            },
            Err(e) => {
                info!("Error fetching last entry: {:?}", e);
                None
            }
        };
    } else {
        info!("Database connection failed. Continuing without database operations.");
    }

    info!("Sending Slack message");
    let mut is_slack_message_sent = false;
    let slack_message =
        slack::compose_slack_message(&validation_results, last_entry, &current_time);

    match send_slack_message(&slack_token, &slack_channel, &slack_message).await {
        Ok(_) => {
            is_slack_message_sent = true;
            info!("Slack message sent");
        }
        Err(e) => {
            info!("Failed to send message to Slack: {}", e);
        }
    }

    let mail_body = mail::compose_mail_body(&validation_results, &current_time);

    info!("\n{}", mail_body);

    let mut is_email_sent = false;
    if validation_results
        .iter()
        .any(|result| result.status == Status::Alert)
    {
        info!("Sending alert email\nMail content:\n{}", mail_body);
        match send_mail(&mail_token, &mail_sender, &mail_recipient, &mail_body).await {
            Ok(_) => {
                is_email_sent = true;
                info!("Email sent");
            }
            Err(e) => {
                info!("Failed to send email: {}", e);
            }
        }
    }

    if let Ok(ref mut conn) = conn {
        let mut log_entry = LogEntry {
            id: None,
            payments: page_results.validated_payments_count.unwrap_or_default() as i32,
            vouchers: page_results.paid_vouchers_count.unwrap_or_default() as i32,
            pdf_count: page_results.pdf_count.unwrap_or_default() as i32,
            email_count: 0,
            website_ok: page_results.is_purchase_website_ok.unwrap_or_default(),
            slack_sent: is_slack_message_sent,
            email_sent: is_email_sent,
            datetime: None,
        };

        if let Some(email_status) = page_results.email_check_count {
            log_entry.email_count = email_status.sent as i32;
        }

        match insert_results(conn, log_entry) {
            Ok(_) => info!("Results inserted into the database"),
            Err(e) => info!("Failed to insert results into the database: {:?}", e),
        }
    } else {
        info!("Failed to establish a database connection");
    }

    info!("Beebot shutdown");
}
