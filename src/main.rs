use clap::Parser;
use dotenv::dotenv;
use log::{error, info};
use tokio;

extern crate diesel;

use crate::db::{get_last_record, load_db, LogEntry};
use crate::mail::send_mail;
use crate::utils::{load_environment, load_logfile};
use crate::validators::Status;

mod db;
mod mail;
mod parser;
mod requests;
mod schema;
mod slack;
mod utils;
mod validators;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    test: bool,
}

#[tokio::main]
async fn main() {
    // Get environment variables
    dotenv().ok();
    let env = load_environment();

    // Init logging
    load_logfile().expect("Failed to initialize logger");
    info!("Beebot starting");

    // Get arguments from CLI
    let args = Args::parse();
    let is_test_mode = args.test;
    if is_test_mode {
        println!("Running in TEST MODE");
        info!("Running in TEST MODE");
    }

    // Init database
    info!("Connecting to db");
    let mut conn = load_db(&env.db_url);

    // Fetch + Parse
    info!("Fetching pages content");
    let html_pages = requests::request_pages(&env.api_token, &env.urls, is_test_mode).await;
    let metrics = parser::extract_metrics(&html_pages, is_test_mode);

    // Metrics validation
    info!("Validating data from HTML content");
    let results = validators::validate(&metrics);
    let last_record: Option<LogEntry> = get_last_record(&mut conn);

    // Generate and send Slack message
    let slack_message = slack::create_message(&results, last_record, is_test_mode);
    info!("Sending Slack message:\n{}\n", slack_message);
    let is_slack_message_sent =
        match slack::post_message(&env.slack_token, &env.slack_channel, &slack_message).await {
            Ok(_) => {
                info!("Slack message sent");
                true
            }
            Err(e) => {
                info!("Failed to send message to Slack: {}", e);
                false
            }
        };

    // Conditionally generate and send email
    let mail_body = mail::compose_mail_body(&results, is_test_mode);
    info!("\n{}", mail_body);
    let mut is_email_sent = false;

    let needs_alert = results.iter().any(|result| result.status == Status::Alert);

    if needs_alert {
        info!("Sending alert email\nMail content:\n{}", mail_body);
        match send_mail(
            &env.mail_token,
            &env.mail_sender,
            &env.mail_recipient,
            &mail_body,
            is_test_mode,
        )
        .await
        {
            Ok(_) => {
                is_email_sent = true;
                info!("Email sent");
            }
            Err(e) => {
                error!("Failed to send email: {}", e);
            }
        }
    }

    // Save result in database
    if !is_test_mode {
        match conn {
            Ok(ref mut conn) => {
                let log_entry = db::create_log(&metrics, is_slack_message_sent, is_email_sent);
                db::insert_log(conn, log_entry);
            }
            Err(_) => {
                error!("Failed to establish a database connection");
            }
        }
    }

    info!("Beebot shutdown");
}
