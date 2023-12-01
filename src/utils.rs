use std::env;
use std::fs::{self, File};

use chrono::Local;
use simplelog::*;

pub struct Environment {
    pub(crate) db_url: String,
    pub(crate) api_token: String,
    pub(crate) slack_token: String,
    pub(crate) slack_channel: String,
    pub(crate) mail_token: String,
    pub(crate) mail_sender: String,
    pub(crate) mail_recipient_1: String,
    pub(crate) mail_recipient_2: String,
    pub(crate) mail_recipient_3: String,
    pub(crate) celery_username: String,
    pub(crate) celery_password: String,
    pub(crate) urls: Vec<(&'static str, String)>,
}

pub fn load_environment() -> Environment {
    let db_url = env::var("DATABASE_URL").unwrap();
    let api_token = env::var("API_TOKEN").unwrap();
    let slack_token = env::var("SLACK_API_TOKEN").unwrap();
    let slack_channel = env::var("SLACK_CHANNEL").unwrap();
    let mail_token = env::var("SENDGRID_API_TOKEN").unwrap();
    let mail_sender = env::var("SENDGRID_SENDER").unwrap();
    let mail_recipient_1 = env::var("SENDGRID_RECIPIENT_1").unwrap();
    let mail_recipient_2 = env::var("SENDGRID_RECIPIENT_2").unwrap();
    let mail_recipient_3 = env::var("SENDGRID_RECIPIENT_3").unwrap();
    let celery_username = env::var("CELERY_USERNAME").unwrap();
    let celery_password = env::var("CELERY_PASSWORD").unwrap();

    let urls = vec![
        ("payments", env::var("URL_PAYMENTS").unwrap()),
        ("vouchers", env::var("URL_VOUCHERS").unwrap()),
        ("paid_vouchers", env::var("URL_PAID_VOUCHERS").unwrap()),
        (
            "purchase_website",
            env::var("URL_PURCHASE_WEBSITE").unwrap(),
        ),
        ("celery", env::var("URL_CELERY").unwrap()),
    ];

    let environment = Environment {
        db_url,
        api_token,
        slack_token,
        slack_channel,
        mail_token,
        mail_sender,
        mail_recipient_1,
        mail_recipient_2,
        mail_recipient_3,
        celery_username,
        celery_password,
        urls,
    };

    environment
}

pub fn load_logfile() -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all("logs").expect("Failed to create logs directory");
    let log_file_name = format!("logs/{}.log", Local::now().format("%Y_%m_%d_%H-%M-%S"));
    let _ = WriteLogger::init(
        LevelFilter::Info,
        Config::default(),
        File::create(log_file_name).unwrap(),
    );

    Ok(())
}
