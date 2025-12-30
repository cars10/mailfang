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
    pub headers: Option<String>,
    pub from: String,
    pub size: i32,
    pub raw_data: String,
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
#[diesel(table_name = email_attachments)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct EmailAttachment {
    pub id: String,
    pub email_id: String,
    pub filename: Option<String>,
    pub mime_type: String,
    pub data: Vec<u8>,
    pub size: i32,
    pub content_id: Option<String>,
    pub headers: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Queryable, Selectable, Insertable, Identifiable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = recipients)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Recipient {
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
#[diesel(belongs_to(Recipient))]
#[diesel(table_name = email_recipients)]
#[diesel(primary_key(email_id, recipient_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct EmailRecipient {
    pub email_id: String,
    pub recipient_id: String,
}
