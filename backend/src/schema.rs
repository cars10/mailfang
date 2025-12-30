// @generated automatically by Diesel CLI.

diesel::table! {
    emails (id) {
        id -> Text,
        message_id -> Nullable<Text>,
        subject -> Nullable<Text>,
        date -> Nullable<Timestamp>,
        headers -> Nullable<Text>,
        from -> Text,
        size -> Integer,
        raw_data -> Text,
        body_text -> Nullable<Text>,
        body_html -> Nullable<Text>,
        rendered_body_html -> Nullable<Text>,
        read -> Bool,
        has_attachments -> Bool,
        created_at -> Timestamp,
    }
}

diesel::table! {
    email_attachments (id) {
        id -> Text,
        email_id -> Text,
        filename -> Nullable<Text>,
        mime_type -> Text,
        data -> Binary,
        size -> Integer,
        content_id -> Nullable<Text>,
        headers -> Nullable<Text>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    recipients (id) {
        id -> Text,
        email -> Text,
    }
}

diesel::table! {
    email_recipients (email_id, recipient_id) {
        email_id -> Text,
        recipient_id -> Text,
    }
}

diesel::joinable!(email_attachments -> emails (email_id));
diesel::joinable!(email_recipients -> emails (email_id));
diesel::joinable!(email_recipients -> recipients (recipient_id));

diesel::allow_tables_to_appear_in_same_query!(
    emails,
    email_attachments,
    recipients,
    email_recipients,
);
