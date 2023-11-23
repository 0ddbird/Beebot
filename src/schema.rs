// @generated automatically by Diesel CLI.

diesel::table! {
    activity_logs (id) {
        id -> Nullable<Integer>,
        payments -> Integer,
        vouchers -> Integer,
        pdf_count -> Integer,
        email_count -> Integer,
        website_ok -> Bool,
        slack_sent -> Bool,
        email_sent -> Bool,
        datetime -> Nullable<Text>,
    }
}
