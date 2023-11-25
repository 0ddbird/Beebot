use std::fmt::{Display, Formatter, Result};

use chrono::prelude::*;
use chrono_tz::Europe::Paris;

use crate::parser::PageResults;
use crate::parser::{EmailStatuses, VoucherStatuses};

#[derive(PartialEq)]
pub enum Status {
    Ok,
    Warning,
    Alert,
}

pub enum Value {
    Count(usize),
    Bool(bool),
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Value::Count(count) => write!(f, "{}", count),
            Value::Bool(b) => write!(f, "{}", b),
        }
    }
}

pub struct UnitValidationResult {
    pub(crate) name: String,
    pub(crate) status: Status,
    pub(crate) message: String,
    pub(crate) value: Value,
}

fn get_threshold(threshold_day: usize, threshold_night: usize) -> usize {
    let paris_time = Local::now().with_timezone(&Paris);
    let hour = paris_time.hour();
    if hour >= 8 && hour < 23 {
        threshold_day
    } else {
        threshold_night
    }
}

pub fn validate(pages: &PageResults) -> Vec<(UnitValidationResult, String)> {
    let threshold = get_threshold(75, 50);

    let payments_result = validate_count(
        "Validated payments",
        threshold,
        pages.validated_payments_count,
    );
    let paid_vouchers_result =
        validate_voucher_status("Paid vouchers", threshold, pages.paid_vouchers_count);
    let pdf_count_result = validate_count("PDF count", threshold, pages.pdf_count);
    let emails_result = validate_email_status("Email count", threshold, pages.email_check_count);
    let purchase_website_result =
        validate_purchase_website_status("Purchase website", pages.is_purchase_website_ok);

    vec![
        (payments_result, pages.url_validated_payments.clone()),
        (paid_vouchers_result, pages.url_vouchers_count.clone()),
        (pdf_count_result, pages.url_pdf_count.clone()),
        (emails_result, pages.url_email_check_count.clone()),
        (purchase_website_result, pages.url_purchase_website.clone()),
    ]
}

fn validate_count(name: &str, threshold: usize, count: Option<usize>) -> UnitValidationResult {
    let mut result = UnitValidationResult {
        name: name.to_string(),
        status: Status::Alert,
        message: "".to_string(),
        value: Value::Count(0),
    };

    match count {
        None => {
            result.message = "NOT AVAILABLE".to_string();
        }
        Some(c) => {
            result.value = Value::Count(c);

            if c == 100 {
                result.status = Status::Ok;
                result.message = format!("`{}`", c);
            } else if c >= 85 && ["Validated payments", "Paid vouchers"].contains(&name) {
                result.status = Status::Ok;
                result.message = format!("`{} (>{})`", c, threshold);
            } else if c > threshold {
                result.status = Status::Warning;
                result.message = format!("`{} (>{})`", c, threshold);
            } else {
                result.status = Status::Alert;
                result.message = format!("`{} (<{})`", c, threshold);
            }
        }
    }

    result
}

fn validate_voucher_status(
    name: &str,
    threshold: usize,
    vouchers: Option<VoucherStatuses>,
) -> UnitValidationResult {
    let mut result = UnitValidationResult {
        name: name.to_string(),
        status: Status::Alert,
        message: "".to_string(),
        value: Value::Count(0),
    };

    if let Some(voucher) = vouchers {
        let total_vouchers = voucher.paid + voucher.error;
        let paid_percentage = if total_vouchers > 0 {
            (voucher.paid as f64 / total_vouchers as f64) * 100.0
        } else {
            0.0
        };

        result.status = if paid_percentage == 100.0 {
            Status::Ok
        } else if paid_percentage > threshold as f64 {
            Status::Warning
        } else {
            Status::Alert
        };

        result.value = Value::Count(voucher.paid);

        result.message = format!(
            "`{} PAID`, `{} ERROR`, `{} OTHER`",
            voucher.paid, voucher.error, voucher.other
        );
    }

    result
}

fn validate_email_status(
    name: &str,
    threshold: usize,
    statuses: Option<EmailStatuses>,
) -> UnitValidationResult {
    let mut result = UnitValidationResult {
        name: name.to_string(),
        status: Status::Alert,
        message: "".to_string(),
        value: Value::Count(0),
    };

    if let Some(statuses) = statuses {
        let total_emails = statuses.sent + statuses.not_sent;
        let sent_percentage = if total_emails > 0 {
            (statuses.sent as f64 / total_emails as f64) * 100.0
        } else {
            0.0
        };

        result.status = if sent_percentage == 100.0 {
            Status::Ok
        } else if sent_percentage > threshold as f64 {
            Status::Warning
        } else {
            Status::Alert
        };

        result.value = Value::Count(statuses.sent);

        result.message = format!(
            "`{} SENT`, `{} NOT SENT`, `{} BULK`",
            statuses.sent, statuses.not_sent, statuses.bulk
        );
    }

    result
}

fn validate_purchase_website_status(name: &str, is_ok: Option<bool>) -> UnitValidationResult {
    let mut result = UnitValidationResult {
        name: name.to_string(),
        status: Status::Alert,
        message: "passed".to_string(),
        value: Value::Bool(false),
    };

    match is_ok {
        Some(true) => {
            result.message = "`ONLINE`".to_string();
            result.status = Status::Ok;
            result.value = Value::Bool(true);
        }
        None | Some(false) => {
            result.message = "`DOWN`".to_string();
            result.status = Status::Alert;
        }
    }

    result
}
