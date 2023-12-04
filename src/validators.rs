use std::fmt::{Display, Formatter, Result};

use chrono::prelude::*;
use chrono_tz::Europe::Paris;

use crate::parser::{EmailStatuses, VoucherStatuses};
use crate::parser::{PageResults, PaymentStatuses};

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

    let payments_result = validate_payment_status(
        "Validated payments",
        threshold,
        pages.validated_payments_count,
    );
    let paid_vouchers_result = validate_voucher_status(
        "Paid vouchers",
        threshold,
        pages.paid_vouchers_count,
        pages.not_imported_count,
    );
    let pdf_count_result = validate_pdf_count(
        "PDF count",
        threshold,
        pages.pdf_count,
        pages.not_imported_count,
    );
    let emails_result = validate_email_status(
        "Email count",
        threshold,
        pages.email_check_count,
        pages.not_imported_count,
    );
    let purchase_website_result =
        validate_purchase_website_status("Purchase website", pages.is_website_online);

    let celery_statuses = validate_celery_statuses("Celery", pages.is_celery_online);

    vec![
        (payments_result, pages.url_validated_payments.clone()),
        (paid_vouchers_result, pages.url_vouchers_count.clone()),
        (pdf_count_result, pages.url_pdf_count.clone()),
        (emails_result, pages.url_email_check_count.clone()),
        (purchase_website_result, pages.url_website.clone()),
        (celery_statuses, pages.url_celery.clone()),
    ]
}

fn validate_pdf_count(
    name: &str,
    threshold: usize,
    pdf_count: usize,
    max_possible_count: usize,
) -> UnitValidationResult {
    let mut result = UnitValidationResult {
        name: name.to_string(),
        status: Status::Alert,
        message: "".to_string(),
        value: Value::Count(0),
    };

    // Arbitrary value to not scare the team with a warning icon
    let fixed_threshold_for_ok = 85 * max_possible_count / 100;

    // The threshold must depend on the maximum possible value
    let relative_threshold_for_warning = threshold * max_possible_count / 100;

    if pdf_count >= fixed_threshold_for_ok {
        result.status = Status::Ok;
    } else if pdf_count >= relative_threshold_for_warning {
        result.status = Status::Warning;
    } else {
        result.status = Status::Alert;
    }
    result.value = Value::Count(pdf_count);
    result.message = format!("`{}/{}`", pdf_count, max_possible_count);

    result
}

fn validate_payment_status(
    name: &str,
    threshold: usize,
    statuses: PaymentStatuses,
) -> UnitValidationResult {
    let mut result = UnitValidationResult {
        name: name.to_string(),
        status: Status::Alert,
        message: "".to_string(),
        value: Value::Count(0),
    };

    let validated_count = statuses.validated;
    let minimum_paid_expected = 100 - statuses.group;

    if validated_count >= 85 * minimum_paid_expected / 100 {
        result.status = Status::Ok;
    } else if validated_count > threshold * minimum_paid_expected / 100 {
        result.status = Status::Warning;
    } else {
        result.status = Status::Alert;
    }
    result.message = format!(
        "`{}/{} VALIDATED` `{} TO VALIDATE` `{} ERROR` `{} 3D SECURE` `{} CANCELLED` `{} GROUP`",
        statuses.validated,
        minimum_paid_expected,
        statuses.to_validate,
        statuses.error,
        statuses.threed_secure,
        statuses.cancelled,
        statuses.group
    );
    result.value = Value::Count(validated_count);

    result
}

fn validate_voucher_status(
    name: &str,
    threshold: usize,
    vouchers: VoucherStatuses,
    max_possible_value: usize,
) -> UnitValidationResult {
    let mut result = UnitValidationResult {
        name: name.to_string(),
        status: Status::Alert,
        message: "".to_string(),
        value: Value::Count(0),
    };

    let total_vouchers = max_possible_value;
    let paid_percentage = if total_vouchers > 0 {
        (vouchers.paid as f64 / total_vouchers as f64) * 100.0
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

    result.value = Value::Count(vouchers.paid);

    result.message = format!(
        "`{}/{} PAID`, `{} ERROR`, `{} OTHER`",
        vouchers.paid, max_possible_value, vouchers.error, vouchers.other
    );

    result
}

fn validate_email_status(
    name: &str,
    threshold: usize,
    statuses: EmailStatuses,
    max_possible_value: usize,
) -> UnitValidationResult {
    let mut result = UnitValidationResult {
        name: name.to_string(),
        status: Status::Alert,
        message: "".to_string(),
        value: Value::Count(0),
    };

    let total_emails = max_possible_value;
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
        "`{}/{} SENT`, `{} NOT SENT`, `{} BULK`",
        statuses.sent, max_possible_value, statuses.not_sent, statuses.bulk
    );

    result
}

fn validate_purchase_website_status(name: &str, is_ok: bool) -> UnitValidationResult {
    let mut result = UnitValidationResult {
        name: name.to_string(),
        status: Status::Alert,
        message: "passed".to_string(),
        value: Value::Bool(false),
    };

    match is_ok {
        true => {
            result.message = "`ONLINE`".to_string();
            result.status = Status::Ok;
            result.value = Value::Bool(true);
        }
        false => {
            result.message = "`DOWN`".to_string();
            result.status = Status::Alert;
        }
    }

    result
}

fn validate_celery_statuses(name: &str, is_online: bool) -> UnitValidationResult {
    let mut result = UnitValidationResult {
        name: name.to_string(),
        status: Status::Alert,
        message: "Celery status: `OFFLINE`".to_string(),
        value: Value::Bool(false),
    };

    if is_online {
        result.message = "`ONLINE`".to_string();
        result.status = Status::Ok;
        result.value = Value::Bool(true)
    }

    result
}
