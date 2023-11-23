-- Your SQL goes here
CREATE TABLE activity_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    payments INTEGER NOT NULL,
    vouchers INTEGER NOT NULL,
    pdf_count INTEGER NOT NULL,
    email_count INTEGER NOT NULL,
    website_ok BOOLEAN NOT NULL,
    slack_sent BOOLEAN NOT NULL,
    email_sent BOOLEAN NOT NULL,
    datetime TEXT DEFAULT CURRENT_TIMESTAMP
);