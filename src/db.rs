use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use log::{error, info};

use crate::parser::PageResults;
use crate::schema::activity_logs;
use crate::schema::activity_logs::dsl::*;

#[derive(Queryable, Insertable)]
#[diesel(table_name = activity_logs)]
pub struct LogEntry {
    pub(crate) id: Option<i32>,
    pub(crate) payments: i32,
    pub(crate) vouchers: i32,
    pub(crate) pdf_count: i32,
    pub(crate) email_count: i32,
    pub(crate) website_ok: bool,
    pub(crate) slack_sent: bool,
    pub(crate) email_sent: bool,
    pub(crate) datetime: Option<String>,
}

pub fn load_db(db_url: &str) -> Result<SqliteConnection, ConnectionError> {
    let database_url = db_url;
    SqliteConnection::establish(&database_url)
}

pub fn create_log(
    page_results: &PageResults,
    is_slack_message_sent: bool,
    is_email_sent: bool,
) -> LogEntry {
    let log_entry = LogEntry {
        id: None,
        payments: page_results.validated_payments_count.validated as i32,
        vouchers: page_results.paid_vouchers_count.paid as i32,
        pdf_count: page_results.pdf_count as i32,
        email_count: page_results.email_check_count.sent as i32,
        website_ok: page_results.is_website_online,
        slack_sent: is_slack_message_sent,
        email_sent: is_email_sent,
        datetime: None,
    };

    log_entry
}

pub fn insert_log(conn: &mut SqliteConnection, log_entry: LogEntry) {
    match diesel::insert_into(activity_logs)
        .values(&log_entry)
        .execute(conn)
    {
        Ok(_) => info!("Results inserted into the database"),
        Err(e) => error!("Failed to insert results into the database: {:?}", e),
    }
}

pub fn get_last_log(conn: &mut Result<SqliteConnection, ConnectionError>) -> Option<LogEntry> {
    match conn {
        Ok(conn) => match activity_logs.order(id.desc()).first(conn) {
            Ok(entry) => {
                info!("Fetched previous log in DB for comparison");
                Some(entry)
            }
            Err(e) => {
                error!("Error fetching last entry: {:?}", e);
                None
            }
        },
        Err(_) => {
            error!("Database connection failed. Continuing without database operations.");
            None
        }
    }
}
