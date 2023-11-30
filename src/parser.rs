use std::collections::HashMap;

use scraper::{Html, Selector};

use crate::requests::Page;

#[derive(Default, Copy, Clone)]
pub struct EmailStatuses {
    pub(crate) sent: usize,
    pub(crate) not_sent: usize,
    pub(crate) bulk: usize,
}

#[derive(Default, Copy, Clone)]
pub struct PaymentStatuses {
    pub(crate) validated: usize,
    pub(crate) to_validate: usize,
    pub(crate) threed_secure: usize,
    pub(crate) cancelled: usize,
    pub(crate) error: usize,
}

#[derive(Default, Copy, Clone)]
pub struct VoucherStatuses {
    pub(crate) paid: usize,
    pub(crate) error: usize,
    pub(crate) other: usize,
}

#[derive(Default, Copy, Clone)]
pub struct PaymentTypes {
    pub(crate) individual: usize,
    pub(crate) group: usize,
}

#[derive(Default)]
pub struct PageResults {
    pub(crate) validated_payments_count: PaymentStatuses,
    pub(crate) payment_types_count: PaymentTypes,
    pub(crate) paid_vouchers_count: VoucherStatuses,
    pub(crate) not_imported_count: usize,
    pub(crate) pdf_count: usize,
    pub(crate) email_check_count: EmailStatuses,
    pub(crate) is_purchase_website_ok: bool,
    pub(crate) url_validated_payments: String,
    pub(crate) url_vouchers_count: String,
    pub(crate) url_pdf_count: String,
    pub(crate) url_email_check_count: String,
    pub(crate) url_purchase_website: String,
}

fn count_vouchers_statuses(html: &str) -> VoucherStatuses {
    let document = Html::parse_document(html);
    let selector = Selector::parse("td.field-state").unwrap();

    let mut voucher_status = VoucherStatuses::default();

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

    let mut email_status = EmailStatuses::default();

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

fn count_not_imported(html: &str) -> usize {
    let document = Html::parse_document(html);
    let selector = Selector::parse("td.field-imported_from").unwrap();

    document
        .select(&selector)
        .filter(|element| element.inner_html().trim() == "-")
        .count()
}

fn count_payment_statuses(html: &str) -> PaymentStatuses {
    let document = Html::parse_document(html);
    let selector = Selector::parse("td.field-state").unwrap();

    let mut payment_statuses = PaymentStatuses::default();

    for element in document.select(&selector) {
        match element.inner_html().trim() {
            "Validated" => payment_statuses.validated += 1,
            "To validate" => payment_statuses.to_validate += 1,
            "3d secure" => payment_statuses.threed_secure += 1,
            "Cancelled" => payment_statuses.cancelled += 1,
            "Error" => payment_statuses.error += 1,
            _ => {}
        }
    }

    payment_statuses
}

fn count_payment_types(html: &str) -> PaymentTypes {
    let document = Html::parse_document(html);
    let selector = Selector::parse("td.field-payment_splitting").unwrap();

    let mut payment_types = PaymentTypes::default();

    for element in document.select(&selector) {
        match element.inner_html().trim() {
            "Individual" => payment_types.individual += 1,
            "Group" => payment_types.group += 1,
            _ => {}
        }
    }

    payment_types
}

fn has_correct_content(html: &str) -> bool {
    let document = Html::parse_document(html);
    let selector = Selector::parse("h1").unwrap();

    document
        .select(&selector)
        .any(|element| element.inner_html().trim() == "Nos bons cadeaux - Le Quatrième Mur")
}

pub fn extract_metrics(html_contents: &HashMap<String, Page>, is_test_mode: bool) -> PageResults {
    let mut results = PageResults::default();

    if is_test_mode {
        results = PageResults {
            validated_payments_count: PaymentStatuses {
                validated: 100,
                to_validate: 0,
                threed_secure: 0,
                cancelled: 0,
                error: 0,
            },
            payment_types_count: PaymentTypes {
                individual: 80,
                group: 20,
            },
            paid_vouchers_count: VoucherStatuses {
                paid: 40,
                error: 10,
                other: 50,
            },
            not_imported_count: 50,
            pdf_count: 76,
            email_check_count: EmailStatuses {
                sent: 30,
                not_sent: 50,
                bulk: 20,
            },
            is_purchase_website_ok: false,
            url_validated_payments: "https://test-domain.com".to_string(),
            url_vouchers_count: "https://test-domain.com".to_string(),
            url_pdf_count: "https://test-domain.com".to_string(),
            url_email_check_count: "https://test-domain.com".to_string(),
            url_purchase_website: "https://test-domain.com".to_string(),
        }
    }

    if let Some(page) = html_contents.get("payments") {
        results.validated_payments_count = count_payment_statuses(&page.html);
        results.payment_types_count = count_payment_types(&page.html);
        results.url_validated_payments = page.url.clone();
    }

    if let Some(page) = html_contents.get("paid_vouchers") {
        results.pdf_count = count_pdf(&page.html);
        results.email_check_count = count_email_statuses(&page.html);
        results.not_imported_count = count_not_imported(&page.html);
        let url = page.url.clone();
        results.url_pdf_count = url.clone();
        results.url_email_check_count = url;
    }

    if let Some(page) = html_contents.get("vouchers") {
        results.paid_vouchers_count = count_vouchers_statuses(&page.html);
        results.url_vouchers_count = page.url.clone();
    }

    if let Some(page) = html_contents.get("purchase_website") {
        results.url_purchase_website = page.url.clone();
        results.is_purchase_website_ok = has_correct_content(&page.html);
    }

    results
}
