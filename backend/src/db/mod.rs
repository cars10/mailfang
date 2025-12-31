use crate::models::*;
use crate::schema;
use crate::smtp;
use crate::web::error::DieselError;
use ::r2d2::PooledConnection;
use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

pub mod counts;
pub mod emails;

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

pub fn save_email(conn: &mut DbConnection, message: &smtp::Email) -> Result<String, DieselError> {
    conn.transaction::<_, DieselError, _>(|conn| {
        // Generate attachment IDs
        let mut attachment_ids: Vec<String> = Vec::new();
        for _ in &message.attachments {
            attachment_ids.push(Uuid::new_v4().to_string());
        }

        let rendered_body_html = if !message.body_html.is_empty() {
            let mut processed_html = message.body_html.clone();
            let mut cid_map = std::collections::HashMap::new();
            for (att, id) in message.attachments.iter().zip(attachment_ids.iter()) {
                if let Some(ref cid) = att.content_id {
                    let attachment_url = format!("/api/attachments/{}", id);
                    let cid_clean = cid
                        .trim_start_matches('<')
                        .trim_end_matches('>')
                        .to_string();
                    cid_map.insert(cid_clean.clone(), attachment_url.clone());
                    if cid.starts_with('<') && cid.ends_with('>') {
                        cid_map.insert(cid.clone(), attachment_url);
                    }
                }
            }

            if !cid_map.is_empty() {
                use regex::Regex;
                let re = Regex::new(r#"(?i)src\s*=\s*(["']?)cid:([^"'\s>]+)"#).unwrap();
                processed_html = re
                    .replace_all(&processed_html, |caps: &regex::Captures| {
                        let quote = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                        let cid = caps.get(2).map(|m| m.as_str()).unwrap_or("");
                        let cid_clean = cid.trim_start_matches('<').trim_end_matches('>');
                        if let Some(attachment_url) = cid_map.get(cid_clean) {
                            format!("src={}{}{}", quote, attachment_url, quote)
                        } else if let Some(attachment_url) = cid_map.get(cid) {
                            format!("src={}{}{}", quote, attachment_url, quote)
                        } else {
                            caps.get(0)
                                .map(|m| m.as_str().to_string())
                                .unwrap_or_default()
                        }
                    })
                    .to_string();
            }
            Some(processed_html)
        } else {
            None
        };

        let now = Utc::now().naive_utc();
        let new_email = Email {
            id: message.id.to_string(),
            message_id: message.message_id.clone(),
            subject: message.subject.clone(),
            date: message.date.map(|d| d.naive_utc()),
            headers: message
                .headers
                .as_ref()
                .map(|h| serde_json::to_string(h).unwrap()),
            from: message.from.clone(),
            size: message.size as i32,
            raw_data: message.data.clone(),
            body_text: Some(message.body_text.clone()),
            body_html: Some(message.body_html.clone()),
            rendered_body_html,
            read: false,
            has_attachments: !message.attachments.is_empty(),
            created_at: now,
        };

        diesel::insert_into(schema::emails::table)
            .values(&new_email)
            .execute(conn)?;

        for recipient_email in &message.to {
            if recipient_email.trim().is_empty() {
                continue;
            }

            let recipient_id = match schema::recipients::table
                .filter(schema::recipients::email.eq(recipient_email))
                .select(schema::recipients::id)
                .first::<String>(conn)
            {
                Ok(id) => id,
                Err(_) => {
                    let new_id = Uuid::new_v4().to_string();
                    diesel::insert_into(schema::recipients::table)
                        .values((
                            schema::recipients::id.eq(&new_id),
                            schema::recipients::email.eq(recipient_email),
                        ))
                        .execute(conn)?;
                    new_id
                }
            };

            diesel::insert_into(schema::email_recipients::table)
                .values((
                    schema::email_recipients::email_id.eq(&new_email.id),
                    schema::email_recipients::recipient_id.eq(&recipient_id),
                ))
                .execute(conn)?;
        }

        for (attachment, attachment_id) in message.attachments.iter().zip(attachment_ids.iter()) {
            let new_attachment = EmailAttachment {
                id: attachment_id.clone(),
                email_id: new_email.id.clone(),
                filename: attachment.filename.clone(),
                mime_type: attachment.mime_type.clone(),
                data: attachment.data.clone(),
                size: attachment.data.len() as i32,
                content_id: attachment.content_id.clone(),
                headers: attachment
                    .headers
                    .as_ref()
                    .map(|h| serde_json::to_string(h).unwrap()),
                created_at: now,
            };
            diesel::insert_into(schema::email_attachments::table)
                .values(&new_attachment)
                .execute(conn)?;
        }

        Ok(new_email.id)
    })
}

// Macro to apply search filters to a query
macro_rules! apply_search_filters {
    ($query:expr, $search_term:expr) => {{
        let pattern = format!("%{}%", $search_term);
        $query.filter(
            schema::emails::subject
                .like(pattern.clone())
                .or(schema::emails::message_id.like(pattern.clone()))
                .or(schema::emails::from.like(pattern.clone()))
                .or(schema::emails::body_text.like(pattern.clone()))
                .or(schema::emails::body_html.like(pattern)),
        )
    }};
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
