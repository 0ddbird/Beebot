use scraper::{Html, Selector};
use std::collections::HashMap;

#[derive(Default, Copy, Clone)]
pub struct EmailStatus {
    pub(crate) sent: usize,
    pub(crate) not_sent: usize,
    pub(crate) bulk: usize,
}

pub struct PageResults {
    pub(crate) validated_payments_count: Option<usize>,
    pub(crate) pdf_count: Option<usize>,
    pub(crate) email_check_count: Option<EmailStatus>,
    pub(crate) paid_vouchers_count: Option<usize>,
    pub(crate) is_purchase_website_ok: Option<bool>,
}

fn count_paid_vouchers(html: &str) -> usize {
    let document = Html::parse_document(html);
    let selector = Selector::parse("td.field-state").unwrap();

    document
        .select(&selector)
        .filter(|element| element.inner_html().trim() == "Paid")
        .count()
}

fn count_pdf(html: &str) -> usize {
    let document = Html::parse_document(html);
    let selector = Selector::parse("td.field-has_pdf").unwrap();

    document
        .select(&selector)
        .filter(|element| element.inner_html().trim() == "Yes")
        .count()
}

fn count_email_statuses(html: &str) -> EmailStatus {
    let document = Html::parse_document(html);
    let selector = Selector::parse("td.field-_has_been_sent").unwrap();

    let mut email_status = EmailStatus {
        sent: 0,
        not_sent: 0,
        bulk: 0,
    };

    for element in document.select(&selector) {
        match element.inner_html().trim() {
            "Yes" => email_status.sent += 1,
            "No" => email_status.not_sent += 1,
            "Bulk" => email_status.bulk += 1,
            _ => {}
        }
    }

    email_status
}

fn count_validated_payments(html: &str) -> usize {
    let document = Html::parse_document(html);
    let selector = Selector::parse("td.field-state").unwrap();

    document
        .select(&selector)
        .filter(|element| element.inner_html().trim() == "Validated")
        .count()
}

fn has_correct_content(html: &str) -> bool {
    let document = Html::parse_document(html);
    let selector = Selector::parse("h1").unwrap();

    document
        .select(&selector)
        .any(|element| element.inner_html().trim() == "Nos bons cadeaux - Le Quatrième Mur")
}

pub fn extract_metrics(html_contents: &HashMap<String, String>, is_test_mode: bool) -> PageResults {
    let mut results = PageResults {
        validated_payments_count: None,
        pdf_count: None,
        email_check_count: None,
        paid_vouchers_count: None,
        is_purchase_website_ok: None,
    };

    if is_test_mode {
        results = PageResults {
            validated_payments_count: Some(76),
            pdf_count: Some(74),
            email_check_count: Some(EmailStatus {
                sent: 30,
                not_sent: 50,
                bulk: 20,
            }),
            paid_vouchers_count: Some(100),
            is_purchase_website_ok: Some(false),
        }
    }

    if let Some(html) = html_contents.get("payments") {
        results.validated_payments_count = Some(count_validated_payments(html));
    }

    if let Some(html) = html_contents.get("paid_vouchers") {
        results.pdf_count = Some(count_pdf(html));
        results.email_check_count = Some(count_email_statuses(html));
    }

    if let Some(html) = html_contents.get("vouchers") {
        results.paid_vouchers_count = Some(count_paid_vouchers(html));
    }

    match html_contents.get("purchase_website") {
        Some(html) if has_correct_content(html) => {
            results.is_purchase_website_ok = Some(true);
        }
        _ => {
            results.is_purchase_website_ok = Some(false);
        }
    }

    results
}
