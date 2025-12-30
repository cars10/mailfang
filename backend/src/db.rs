use crate::models::*;
use crate::schema;
use crate::smtp;
use ::r2d2::PooledConnection;
use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

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

pub fn save_email(
    conn: &mut DbConnection,
    message: &smtp::Email,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    conn.transaction::<_, Box<dyn std::error::Error + Send + Sync>, _>(|conn| {
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

pub fn get_all_emails(
    conn: &mut DbConnection,
    query_params: &ListQuery,
) -> Result<(Vec<EmailListRecord>, u64), Box<dyn std::error::Error + Send + Sync>> {
    let build_query = || {
        let mut query = schema::emails::table.into_boxed();

        if let Some(ref search_term) = query_params.search {
            let pattern = format!("%{}%", search_term);
            query = query.filter(
                schema::emails::subject
                    .like(pattern.clone())
                    .or(schema::emails::message_id.like(pattern.clone()))
                    .or(schema::emails::from.like(pattern.clone()))
                    .or(schema::emails::body_text.like(pattern.clone()))
                    .or(schema::emails::body_html.like(pattern)),
            );
        }
        query
    };

    let total_count: i64 = build_query().count().get_result(conn)?;
    let num_pages = (total_count as f64 / query_params.per_page as f64).ceil() as u64;

    let emails = build_query()
        .order(schema::emails::created_at.desc())
        .offset(((query_params.page - 1) * query_params.per_page) as i64)
        .limit(query_params.per_page as i64)
        .load::<Email>(conn)?;

    let all_recipients = EmailRecipient::belonging_to(&emails)
        .inner_join(schema::recipients::table)
        .select((EmailRecipient::as_select(), Recipient::as_select()))
        .load::<(EmailRecipient, Recipient)>(conn)?;

    let recipients_per_email = all_recipients.grouped_by(&emails);

    let records = emails
        .into_iter()
        .zip(recipients_per_email)
        .map(|(email, recipients)| EmailListRecord {
            id: email.id,
            subject: email.subject,
            date: email.date,
            created_at: email.created_at,
            from: email.from,
            read: email.read,
            has_attachments: email.has_attachments,
            recipients: recipients.into_iter().map(|(_, r)| r.email).collect(),
        })
        .collect();

    Ok((records, num_pages))
}

pub fn get_email_by_id(
    conn: &mut DbConnection,
    email_id: &str,
) -> Result<Option<EmailRecord>, Box<dyn std::error::Error + Send + Sync>> {
    let email = schema::emails::table
        .filter(schema::emails::id.eq(email_id))
        .first::<Email>(conn)
        .optional()?;

    if let Some(email) = email {
        let attachments = EmailAttachment::belonging_to(&email)
            .select(EmailAttachment::as_select())
            .load::<EmailAttachment>(conn)?;

        let recipients = EmailRecipient::belonging_to(&email)
            .inner_join(schema::recipients::table)
            .select(Recipient::as_select())
            .load::<Recipient>(conn)?;

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
            recipients: recipients.into_iter().map(|r| r.email).collect(),
            attachments: attachment_records,
        }))
    } else {
        Ok(None)
    }
}

pub fn delete_email_by_id(
    conn: &mut DbConnection,
    email_id: &str,
) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    let affected = diesel::delete(schema::emails::table.filter(schema::emails::id.eq(email_id)))
        .execute(conn)?;
    Ok(affected)
}

pub fn delete_all_emails(
    conn: &mut DbConnection,
) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    let affected = diesel::delete(schema::emails::table).execute(conn)?;
    Ok(affected)
}

pub fn mark_email_read(
    conn: &mut DbConnection,
    email_id: &str,
    read: bool,
) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    let affected = diesel::update(schema::emails::table.filter(schema::emails::id.eq(email_id)))
        .set(schema::emails::read.eq(read))
        .execute(conn)?;
    Ok(affected)
}

pub fn get_attachment_by_id(
    conn: &mut DbConnection,
    attachment_id: &str,
) -> Result<Option<AttachmentContent>, Box<dyn std::error::Error + Send + Sync>> {
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
) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
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
) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
    let data = schema::emails::table
        .filter(schema::emails::id.eq(email_id))
        .select(schema::emails::raw_data)
        .first::<String>(conn)
        .optional()?;

    Ok(data)
}

pub fn get_email_stats(
    conn: &mut DbConnection,
) -> Result<EmailStats, Box<dyn std::error::Error + Send + Sync>> {
    let inbox_count: i64 = schema::emails::table.count().get_result(conn)?;

    let rows = schema::recipients::table
        .inner_join(schema::email_recipients::table)
        .group_by((schema::recipients::id, schema::recipients::email))
        .select((
            schema::recipients::email,
            diesel::dsl::count(schema::email_recipients::email_id),
        ))
        .order_by(schema::recipients::email.asc())
        .load::<(String, i64)>(conn)?;

    let recipients = rows
        .into_iter()
        .map(|(recipient, count)| RecipientStats {
            recipient,
            count: count as u64,
        })
        .collect();

    Ok(EmailStats {
        inbox: inbox_count as u64,
        recipients,
    })
}

#[derive(serde::Serialize, Clone)]
pub struct RecipientStats {
    pub recipient: String,
    pub count: u64,
}

#[derive(serde::Serialize, Clone)]
pub struct EmailStats {
    pub inbox: u64,
    pub recipients: Vec<RecipientStats>,
}

pub fn get_emails_by_recipient(
    conn: &mut DbConnection,
    recipient_email: &str,
    query_params: &ListQuery,
) -> Result<(Vec<EmailListRecord>, u64), Box<dyn std::error::Error + Send + Sync>> {
    let build_query = || {
        let mut query = schema::emails::table
            .inner_join(schema::email_recipients::table)
            .inner_join(
                schema::recipients::table
                    .on(schema::email_recipients::recipient_id.eq(schema::recipients::id)),
            )
            .filter(schema::recipients::email.eq(recipient_email))
            .select(schema::emails::all_columns)
            .into_boxed();

        if let Some(ref search_term) = query_params.search {
            let pattern = format!("%{}%", search_term);
            query = query.filter(
                schema::emails::subject
                    .like(pattern.clone())
                    .or(schema::emails::message_id.like(pattern.clone()))
                    .or(schema::emails::from.like(pattern.clone()))
                    .or(schema::emails::body_text.like(pattern.clone()))
                    .or(schema::emails::body_html.like(pattern)),
            );
        }
        query
    };

    let total_count: i64 = build_query().count().get_result(conn)?;
    let num_pages = (total_count as f64 / query_params.per_page as f64).ceil() as u64;

    let emails = build_query()
        .order(schema::emails::created_at.desc())
        .offset(((query_params.page - 1) * query_params.per_page) as i64)
        .limit(query_params.per_page as i64)
        .load::<Email>(conn)?;

    let all_recipients = EmailRecipient::belonging_to(&emails)
        .inner_join(schema::recipients::table)
        .select((EmailRecipient::as_select(), Recipient::as_select()))
        .load::<(EmailRecipient, Recipient)>(conn)?;

    let recipients_per_email = all_recipients.grouped_by(&emails);

    let records = emails
        .into_iter()
        .zip(recipients_per_email)
        .map(|(email, recipients)| EmailListRecord {
            id: email.id,
            subject: email.subject,
            date: email.date,
            created_at: email.created_at,
            from: email.from,
            read: email.read,
            has_attachments: email.has_attachments,
            recipients: recipients.into_iter().map(|(_, r)| r.email).collect(),
        })
        .collect();

    Ok((records, num_pages))
}
