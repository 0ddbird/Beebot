use chrono::Local;
use dotenv::dotenv;
use log::info;
use tokio;

extern crate diesel;

use crate::db::{establish_connection, get_last_record, LogEntry};
use crate::mail::send_mail;
use crate::slack::post_message;
use crate::utils::{get_environment, get_logfile};
use crate::validators::Status;

mod db;
mod mail;
mod parser;
mod requests;
mod schema;
mod slack;
mod utils;
mod validators;

#[tokio::main]
async fn main() {
    get_logfile().expect("Failed to initialize logger");
    info!("Beebot starting");

    dotenv().ok();
    let env = get_environment();

    info!("Connecting to db");
    let mut conn = establish_connection(&env.db_url);

    info!("Fetching pages content");
    let pages = requests::request_pages(&env.api_token, &env.urls).await;
    let pages_html = parser::parse_pages(&pages);

    info!("Validating data from HTML content");
    let results = validators::validate(&pages_html);
    let current_time = Local::now().format("%H:%M").to_string();
    let last_record: Option<LogEntry> = get_last_record(&mut conn);

    info!("Sending Slack message");
    let mut is_slack_message_sent = false;
    let slack_message = slack::create_message(&results, last_record, &current_time);
    match post_message(&env.slack_token, &env.slack_channel, &slack_message).await {
        Ok(_) => {
            is_slack_message_sent = true;
            info!("Slack message sent");
        }
        Err(e) => {
            info!("Failed to send message to Slack: {}", e);
        }
    }

    let mail_body = mail::compose_mail_body(&results, &current_time);
    info!("\n{}", mail_body);
    let mut is_email_sent = false;
    if results.iter().any(|result| result.status == Status::Alert) {
        info!("Sending alert email\nMail content:\n{}", mail_body);
        match send_mail(
            &env.mail_token,
            &env.mail_sender,
            &env.mail_recipient,
            &mail_body,
        )
        .await
        {
            Ok(_) => {
                is_email_sent = true;
                info!("Email sent");
            }
            Err(e) => {
                info!("Failed to send email: {}", e);
            }
        }
    }

    match conn {
        Ok(ref mut conn) => {
            let log_entry = db::create_log(&pages_html, is_slack_message_sent, is_email_sent);
            db::insert_log(conn, log_entry);
        }
        Err(_) => {
            info!("Failed to establish a database connection");
        }
    }

    info!("Beebot shutdown");
}
