use crate::PageChecks;

#[derive(PartialEq)]
pub enum Status {
    Ok,
    Warning,
    Alert,
}

pub struct UnitValidationResult {
    pub(crate) name: String,
    pub(crate) status: Status,
    pub(crate) message: String,
}

pub fn validate_checks(page_checks: &PageChecks) -> Vec<UnitValidationResult> {
    let payments_result = validate_count(
        "Validated payments",
        75,
        page_checks.validated_payments_count,
    );
    let paid_vouchers_result = validate_count("Paid vouchers", 75, page_checks.paid_vouchers_count);
    let pdf_count_result = validate_count("PDF count", 100, page_checks.pdf_count);
    let email_check_result =
        validate_count("Email check count", 100, page_checks.email_check_count);
    let purchase_website_result =
        validate_purchase_website_status("Purchase website", page_checks.is_purchase_website_ok);

    vec![
        payments_result,
        paid_vouchers_result,
        pdf_count_result,
        email_check_result,
        purchase_website_result,
    ]
}

fn validate_count(name: &str, threshold: usize, count: Option<usize>) -> UnitValidationResult {
    let mut result = UnitValidationResult {
        name: name.to_string(),
        status: Status::Alert,
        message: "".to_string(),
    };

    match count {
        None => {
            result.message = "ARE NOT AVAILABLE".to_string();
            result.status = Status::Alert;
        }
        Some(c) if c < threshold => {
            result.message = format!("{} &lt; {}", c, threshold);
            result.status = Status::Warning;
        }
        Some(c) => {
            result.message = format!("{}", c);
            result.status = Status::Ok;
        }
    }

    result
}

fn validate_purchase_website_status(name: &str, is_ok: Option<bool>) -> UnitValidationResult {
    let mut result = UnitValidationResult {
        name: name.to_string(),
        status: Status::Alert,
        message: "passed".to_string(),
    };

    match is_ok {
        Some(true) => {
            result.message = "is OK".to_string();
            result.status = Status::Ok;
        }
        None | Some(false) => {
            result.message = "is DOWN".to_string();
            result.status = Status::Alert;
        }
    }

    result
}
