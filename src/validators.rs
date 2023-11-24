use crate::parser::EmailStatus;
use crate::parser::PageResults;
use std::fmt::{Display, Formatter, Result};

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

pub fn validate(pages: &PageResults) -> Vec<UnitValidationResult> {
    let payments_result = validate_count("Validated payments", 75, pages.validated_payments_count);
    let paid_vouchers_result = validate_count("Paid vouchers", 75, pages.paid_vouchers_count);
    let pdf_count_result = validate_count("PDF count", 100, pages.pdf_count);
    let emails_result = validate_status("Email count", pages.email_check_count);
    let purchase_website_result =
        validate_purchase_website_status("Purchase website", pages.is_purchase_website_ok);

    vec![
        payments_result,
        paid_vouchers_result,
        pdf_count_result,
        emails_result,
        purchase_website_result,
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
            result.status = Status::Alert;
        }
        Some(c) if c < threshold / 2 => {
            result.message = format!("`{} (<{})`", c, threshold / 2);
            result.value = Value::Count(c);
            result.status = Status::Alert;
        }
        Some(c) if c < threshold => {
            result.message = format!("`{} (<{})`", c, threshold);
            result.value = Value::Count(c);
            result.status = Status::Warning;
        }
        Some(c) => {
            result.message = format!("`{}`", c);
            result.value = Value::Count(c);
            result.status = Status::Ok;
        }
    }

    result
}

fn validate_status(name: &str, statuses: Option<EmailStatus>) -> UnitValidationResult {
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

        result.status = if sent_percentage >= 75.0 {
            Status::Ok
        } else if sent_percentage >= 60.0 {
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
