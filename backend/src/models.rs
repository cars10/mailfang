use crate::schema::*;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Insertable, Identifiable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = emails)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Email {
    pub id: String,
    pub message_id: Option<String>,
    pub subject: Option<String>,
    pub date: Option<NaiveDateTime>,
    pub envelope_from: String,
    pub size: i32,
    pub compressed_data: Vec<u8>,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub rendered_body_html: Option<String>,
    pub read: bool,
    pub has_attachments: bool,
    pub created_at: NaiveDateTime,
}

#[derive(
    Queryable,
    Selectable,
    Insertable,
    Identifiable,
    Associations,
    Debug,
    Clone,
    Serialize,
    Deserialize,
)]
#[diesel(belongs_to(Email))]
#[diesel(table_name = attachments)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Attachment {
    pub id: String,
    pub email_id: String,
    pub filename: Option<String>,
    pub content_type: Option<String>,
    pub compressed_data: Vec<u8>,
    pub size: i32,
    pub content_id: Option<String>,
    pub disposition: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Queryable, Selectable, Insertable, Identifiable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = envelope_recipients)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct EnvelopeRecipient {
    pub id: String,
    pub email: String,
}

#[derive(
    Queryable,
    Selectable,
    Insertable,
    Identifiable,
    Associations,
    Debug,
    Clone,
    Serialize,
    Deserialize,
)]
#[diesel(belongs_to(Email))]
#[diesel(belongs_to(EnvelopeRecipient))]
#[diesel(table_name = email_envelope_recipients)]
#[diesel(primary_key(email_id, envelope_recipient_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct EmailEnvelopeRecipient {
    pub email_id: String,
    pub envelope_recipient_id: String,
}

#[derive(
    Queryable,
    Selectable,
    Insertable,
    Identifiable,
    Associations,
    Debug,
    Clone,
    Serialize,
    Deserialize,
)]
#[diesel(belongs_to(Email))]
#[diesel(table_name = headers)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Header {
    pub id: String,
    pub email_id: String,
    pub name: String,
    pub value: String,
    pub created_at: NaiveDateTime,
}
