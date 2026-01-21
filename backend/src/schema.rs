// @generated automatically by Diesel CLI.

diesel::table! {
    attachments (id) {
        id -> Text,
        email_id -> Text,
        filename -> Nullable<Text>,
        content_type -> Nullable<Text>,
        compressed_data -> Binary,
        size -> Integer,
        content_id -> Nullable<Text>,
        disposition -> Nullable<Text>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    email_envelope_recipients (email_id, envelope_recipient_id) {
        email_id -> Text,
        envelope_recipient_id -> Text,
    }
}

diesel::table! {
    emails (id) {
        id -> Text,
        message_id -> Nullable<Text>,
        subject -> Nullable<Text>,
        date -> Nullable<Timestamp>,
        envelope_from -> Text,
        size -> Integer,
        compressed_data -> Binary,
        body_text -> Nullable<Text>,
        body_html -> Nullable<Text>,
        rendered_body_html -> Nullable<Text>,
        read -> Bool,
        has_attachments -> Bool,
        created_at -> Timestamp,
    }
}

diesel::table! {
    envelope_recipients (id) {
        id -> Text,
        email -> Text,
    }
}

diesel::table! {
    headers (id) {
        id -> Text,
        email_id -> Text,
        name -> Text,
        value -> Text,
        created_at -> Timestamp,
    }
}

diesel::joinable!(attachments -> emails (email_id));
diesel::joinable!(email_envelope_recipients -> emails (email_id));
diesel::joinable!(email_envelope_recipients -> envelope_recipients (envelope_recipient_id));
diesel::joinable!(headers -> emails (email_id));

diesel::allow_tables_to_appear_in_same_query!(
    attachments,
    email_envelope_recipients,
    emails,
    envelope_recipients,
    headers,
);
