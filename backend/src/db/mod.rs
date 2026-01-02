use crate::schema;
use crate::web::error::DieselError;
use ::r2d2::PooledConnection;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use serde::Serialize;
use std::sync::Arc;

pub mod attachment;
pub mod counts;
pub mod email;
pub mod emails;
pub mod save_email;

#[derive(HasQuery, Clone)]
#[diesel(table_name = schema::emails)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct EmailListPartial {
    pub id: String,
    pub subject: Option<String>,
    pub date: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub from: String,
    pub read: bool,
    pub has_attachments: bool,
}

#[derive(HasQuery, Clone)]
#[diesel(table_name = schema::emails)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct EmailPartial {
    pub id: String,
    pub message_id: Option<String>,
    pub subject: Option<String>,
    pub date: Option<NaiveDateTime>,
    pub headers: Option<String>,
    pub created_at: NaiveDateTime,
    pub from: String,
    pub size: i32,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub read: bool,
}

#[derive(HasQuery, Clone)]
#[diesel(table_name = schema::email_attachments)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct EmailAttachmentPartial {
    pub id: String,
    pub filename: Option<String>,
    pub mime_type: String,
    pub size: i32,
    pub content_id: Option<String>,
    pub headers: Option<String>,
    pub created_at: NaiveDateTime,
}

pub type DbPool = Arc<r2d2::Pool<ConnectionManager<SqliteConnection>>>;
pub type DbConnection = PooledConnection<ConnectionManager<SqliteConnection>>;

pub fn serialize_json_string<S>(s: &Option<String>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match s {
        Some(s) => {
            let v: serde_json::Value = serde_json::from_str(s).unwrap_or(serde_json::Value::Null);
            v.serialize(serializer)
        }
        None => serializer.serialize_none(),
    }
}

#[derive(Clone, Default, serde::Deserialize)]
pub struct ListParams {
    pub search: Option<String>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

pub struct ListQuery {
    pub search: Option<String>,
    pub page: u64,
    pub per_page: u64,
}

impl From<ListParams> for ListQuery {
    fn from(params: ListParams) -> Self {
        Self {
            search: params.search,
            page: params.page.unwrap_or(1),
            per_page: params.per_page.unwrap_or(20),
        }
    }
}

#[derive(serde::Serialize, Clone)]
pub struct EmailRecord {
    pub id: String,
    pub message_id: Option<String>,
    pub subject: Option<String>,
    pub date: Option<NaiveDateTime>,
    #[serde(serialize_with = "serialize_json_string")]
    pub headers: Option<String>,
    pub created_at: NaiveDateTime,
    pub from: String,
    pub size: i32,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub read: bool,
    pub recipients: Vec<String>,
    pub attachments: Vec<EmailAttachmentRecord>,
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct EmailListRecord {
    pub id: String,
    pub subject: Option<String>,
    pub date: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub from: String,
    pub read: bool,
    pub has_attachments: bool,
    pub recipients: Vec<String>,
}

impl From<EmailRecord> for EmailListRecord {
    fn from(record: EmailRecord) -> Self {
        Self {
            id: record.id,
            subject: record.subject,
            date: record.date,
            created_at: record.created_at,
            from: record.from,
            recipients: record.recipients,
            read: record.read,
            has_attachments: !record.attachments.is_empty(),
        }
    }
}

#[derive(serde::Serialize, Clone)]
pub struct EmailAttachmentRecord {
    pub id: String,
    pub filename: Option<String>,
    pub mime_type: String,
    pub size: i32,
    pub content_id: Option<String>,
    #[serde(serialize_with = "serialize_json_string")]
    pub headers: Option<String>,
    pub created_at: NaiveDateTime,
}

pub struct AttachmentContent {
    pub id: String,
    pub filename: Option<String>,
    pub mime_type: String,
    pub data: Vec<u8>,
}

pub fn vacuum_database(conn: &mut DbConnection) -> Result<(), DieselError> {
    diesel::sql_query("VACUUM").execute(conn)?;
    Ok(())
}
