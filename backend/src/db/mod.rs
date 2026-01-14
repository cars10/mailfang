use crate::schema;
use crate::web::error::DieselError;
use ::r2d2::PooledConnection;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use serde::Serialize;
use std::sync::Arc;

#[derive(Debug)]
pub enum DbError {
    Diesel(DieselError),
    Io(std::io::Error),
}

impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DbError::Diesel(e) => write!(f, "Database error: {}", e),
            DbError::Io(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for DbError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DbError::Diesel(e) => Some(e),
            DbError::Io(e) => Some(e),
        }
    }
}

impl From<DieselError> for DbError {
    fn from(err: DieselError) -> Self {
        DbError::Diesel(err)
    }
}

impl From<std::io::Error> for DbError {
    fn from(err: std::io::Error) -> Self {
        DbError::Io(err)
    }
}

impl From<DbError> for crate::web::error::WebError {
    fn from(err: DbError) -> Self {
        match err {
            DbError::Diesel(e) => crate::web::error::WebError::from(e),
            DbError::Io(e) => crate::web::error::WebError::from(e),
        }
    }
}

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
    pub headers: Option<String>,
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
#[diesel(table_name = schema::attachments)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct AttachmentPartial {
    pub id: String,
    pub filename: Option<String>,
    pub content_type: Option<String>,
    pub size: i32,
    pub content_id: Option<String>,
    pub disposition: Option<String>,
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
    pub attachments: Vec<AttachmentRecord>,
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
    #[serde(serialize_with = "serialize_json_string")]
    pub headers: Option<String>,
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
            headers: record.headers,
        }
    }
}

#[derive(serde::Serialize, Clone)]
pub struct AttachmentRecord {
    pub id: String,
    pub filename: Option<String>,
    pub content_type: Option<String>,
    pub size: i32,
    pub content_id: Option<String>,
    pub disposition: Option<String>,
    pub created_at: NaiveDateTime,
}

pub struct AttachmentContent {
    pub id: String,
    pub filename: Option<String>,
    pub content_type: Option<String>,
    pub data: Vec<u8>,
}

pub fn vacuum_database(conn: &mut DbConnection) -> Result<(), DieselError> {
    diesel::sql_query("VACUUM").execute(conn)?;
    Ok(())
}
