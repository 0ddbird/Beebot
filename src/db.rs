use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

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

pub fn establish_connection(db_url: &str) -> Result<SqliteConnection, ConnectionError> {
    let database_url = db_url;
    SqliteConnection::establish(&database_url)
}

pub fn insert_results(conn: &mut SqliteConnection, log_entry: LogEntry) -> QueryResult<usize> {
    diesel::insert_into(activity_logs)
        .values(&log_entry)
        .execute(conn)
}

pub fn get_last_entry(conn: &mut SqliteConnection) -> Result<LogEntry, diesel::result::Error> {
    activity_logs.order(id.desc()).first(conn)
}
