use crate::requests::Page;
use scraper::{Html, Selector};
use std::collections::HashMap;

#[derive(Default, Copy, Clone)]
pub struct EmailStatuses {
    pub(crate) sent: usize,
    pub(crate) not_sent: usize,
    pub(crate) bulk: usize,
}

#[derive(Default, Copy, Clone)]
pub struct VoucherStatuses {
    pub(crate) paid: usize,
    pub(crate) error: usize,
    pub(crate) other: usize,
}

pub struct PageResults {
    pub(crate) validated_payments_count: Option<usize>,
    pub(crate) paid_vouchers_count: Option<VoucherStatuses>,
    pub(crate) pdf_count: Option<usize>,
    pub(crate) email_check_count: Option<EmailStatuses>,
    pub(crate) is_purchase_website_ok: Option<bool>,
    pub(crate) url_validated_payments: String,
    pub(crate) url_vouchers_count: String,
    pub(crate) url_pdf_count: String,
    pub(crate) url_email_check_count: String,
    pub(crate) url_purchase_website: String,
}

fn count_vouchers_statuses(html: &str) -> VoucherStatuses {
    let document = Html::parse_document(html);
    let selector = Selector::parse("td.field-state").unwrap();

    let mut voucher_status = VoucherStatuses {
        paid: 0,
        error: 0,
        other: 0,
    };

    for element in document.select(&selector) {
        match element.inner_html().trim() {
            "Paid" => voucher_status.paid += 1,
            "Error" => voucher_status.error += 1,
            _ => voucher_status.other += 1,
        }
    }

    voucher_status
}

fn count_pdf(html: &str) -> usize {
    let document = Html::parse_document(html);
    let selector = Selector::parse("td.field-has_pdf").unwrap();

    document
        .select(&selector)
        .filter(|element| element.inner_html().trim() == "Yes")
        .count()
}

fn count_email_statuses(html: &str) -> EmailStatuses {
    let document = Html::parse_document(html);
    let selector = Selector::parse("td.field-_has_been_sent").unwrap();

    let mut email_status = EmailStatuses {
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

pub fn extract_metrics(html_contents: &HashMap<String, Page>, is_test_mode: bool) -> PageResults {
    let mut results = PageResults {
        validated_payments_count: None,
        pdf_count: None,
        email_check_count: None,
        paid_vouchers_count: None,
        is_purchase_website_ok: None,
        url_validated_payments: "".to_string(),
        url_vouchers_count: "".to_string(),
        url_pdf_count: "".to_string(),
        url_email_check_count: "".to_string(),
        url_purchase_website: "".to_string(),
    };

    if is_test_mode {
        results = PageResults {
            validated_payments_count: Some(85),
            paid_vouchers_count: Some(VoucherStatuses {
                paid: 40,
                error: 10,
                other: 50,
            }),
            pdf_count: Some(76),
            email_check_count: Some(EmailStatuses {
                sent: 30,
                not_sent: 50,
                bulk: 20,
            }),
            is_purchase_website_ok: Some(false),
            url_validated_payments: "https://test-domain.com".to_string(),
            url_vouchers_count: "https://test-domain.com".to_string(),
            url_pdf_count: "https://test-domain.com".to_string(),
            url_email_check_count: "https://test-domain.com".to_string(),
            url_purchase_website: "https://test-domain.com".to_string(),
        }
    }

    if let Some(page) = html_contents.get("payments") {
        results.validated_payments_count = Some(count_validated_payments(&page.html));
        results.url_validated_payments = page.url.clone();
    }

    if let Some(page) = html_contents.get("paid_vouchers") {
        results.pdf_count = Some(count_pdf(&page.html));
        results.email_check_count = Some(count_email_statuses(&page.html));
        let url = page.url.clone();
        results.url_pdf_count = url.clone();
        results.url_email_check_count = url;
    }

    if let Some(page) = html_contents.get("vouchers") {
        results.paid_vouchers_count = Some(count_vouchers_statuses(&page.html));
        results.url_vouchers_count = page.url.clone();
    }

    if let Some(page) = html_contents.get("purchase_website") {
        results.url_purchase_website = page.url.clone();
        results.is_purchase_website_ok = Some(has_correct_content(&page.html));
    } else {
        results.is_purchase_website_ok = Some(false);
    }

    results
}
