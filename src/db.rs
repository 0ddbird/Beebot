use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

use crate::schema::activity_logs;

#[derive(Insertable)]
#[diesel(table_name = activity_logs)]
pub struct LogEntry {
    pub(crate) payments: i32,
    pub(crate) vouchers: i32,
    pub(crate) pdf_count: i32,
    pub(crate) email_count: i32,
    pub(crate) website_ok: bool,
    pub(crate) slack_sent: bool,
    pub(crate) email_sent: bool,
}

pub fn establish_connection(db_url: &str) -> Result<SqliteConnection, diesel::ConnectionError> {
    let database_url = db_url;
    SqliteConnection::establish(&database_url)
}

pub fn insert_results(conn: &mut SqliteConnection, log_entry: LogEntry) -> QueryResult<usize> {
    use crate::schema::activity_logs::dsl::activity_logs;
    diesel::insert_into(activity_logs).values(&log_entry).execute(conn)
}
