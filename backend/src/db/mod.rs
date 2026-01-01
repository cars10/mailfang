use crate::models::*;
use crate::schema;
use crate::web::error::DieselError;
use ::r2d2::PooledConnection;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use serde::Serialize;
use std::sync::Arc;

pub mod counts;
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

pub fn get_email_by_id(
    conn: &mut DbConnection,
    email_id: &str,
) -> Result<Option<EmailRecord>, DieselError> {
    let email = EmailPartial::query()
        .filter(schema::emails::id.eq(email_id))
        .first::<EmailPartial>(conn)
        .optional()?;

    if let Some(email) = email {
        let attachments = EmailAttachmentPartial::query()
            .filter(schema::email_attachments::email_id.eq(&email.id))
            .load::<EmailAttachmentPartial>(conn)?;

        let recipients: Vec<String> = schema::email_recipients::table
            .inner_join(schema::recipients::table)
            .filter(schema::email_recipients::email_id.eq(&email.id))
            .select(schema::recipients::email)
            .load(conn)?;

        let attachment_records = attachments
            .into_iter()
            .map(|att| EmailAttachmentRecord {
                id: att.id,
                filename: att.filename,
                mime_type: att.mime_type,
                size: att.size,
                content_id: att.content_id,
                headers: att.headers,
                created_at: att.created_at,
            })
            .collect();

        Ok(Some(EmailRecord {
            id: email.id,
            message_id: email.message_id,
            subject: email.subject,
            date: email.date,
            headers: email.headers,
            created_at: email.created_at,
            from: email.from,
            size: email.size,
            body_text: email.body_text,
            body_html: email.body_html,
            read: email.read,
            recipients,
            attachments: attachment_records,
        }))
    } else {
        Ok(None)
    }
}

pub fn delete_email_by_id(conn: &mut DbConnection, email_id: &str) -> Result<usize, DieselError> {
    let affected = diesel::delete(schema::emails::table.filter(schema::emails::id.eq(email_id)))
        .execute(conn)?;
    Ok(affected)
}

pub fn delete_all_emails(conn: &mut DbConnection) -> Result<usize, DieselError> {
    let affected = diesel::delete(schema::emails::table).execute(conn)?;
    Ok(affected)
}

pub fn mark_email_read(
    conn: &mut DbConnection,
    email_id: &str,
    read: bool,
) -> Result<usize, DieselError> {
    let affected = diesel::update(schema::emails::table.filter(schema::emails::id.eq(email_id)))
        .set(schema::emails::read.eq(read))
        .execute(conn)?;
    Ok(affected)
}

pub fn get_attachment_by_id(
    conn: &mut DbConnection,
    attachment_id: &str,
) -> Result<Option<AttachmentContent>, DieselError> {
    let attachment = schema::email_attachments::table
        .filter(schema::email_attachments::id.eq(attachment_id))
        .first::<EmailAttachment>(conn)
        .optional()?;

    Ok(attachment.map(|att| AttachmentContent {
        id: att.id,
        filename: att.filename,
        mime_type: att.mime_type,
        data: att.data,
    }))
}

pub fn get_rendered_data_by_id(
    conn: &mut DbConnection,
    email_id: &str,
) -> Result<Option<String>, DieselError> {
    let data = schema::emails::table
        .filter(schema::emails::id.eq(email_id))
        .select(schema::emails::rendered_body_html)
        .first::<Option<String>>(conn)
        .optional()?;

    Ok(data.flatten())
}

pub fn get_raw_data_by_id(
    conn: &mut DbConnection,
    email_id: &str,
) -> Result<Option<String>, DieselError> {
    let data = schema::emails::table
        .filter(schema::emails::id.eq(email_id))
        .select(schema::emails::raw_data)
        .first::<String>(conn)
        .optional()?;

    Ok(data)
}
