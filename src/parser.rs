use crate::PageResults;
use scraper::{Html, Selector};
use std::collections::HashMap;

#[derive(Default, Copy, Clone)]
pub struct EmailStatus {
    pub(crate) sent: usize,
    pub(crate) not_sent: usize,
    pub(crate) bulk: usize,
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

pub fn parse_pages(html_contents: &HashMap<String, String>) -> PageResults {
    let mut results = PageResults {
        validated_payments_count: None,
        pdf_count: None,
        email_check_count: None,
        paid_vouchers_count: None,
        is_purchase_website_ok: None,
    };

    if let Some(html) = html_contents.get("payments") {
        results.validated_payments_count = Some(count_validated_payments(html));
    }

    if let Some(html) = html_contents.get("vouchers") {
        results.pdf_count = Some(count_pdf(html));
        results.email_check_count = Some(count_email_statuses(html));
    }

    if let Some(html) = html_contents.get("paid_vouchers") {
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
