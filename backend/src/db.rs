use crate::entities::{email_attachments, email_recipients, emails, recipients};
use crate::smtp::Email;
use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, FromQueryResult, Insert,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, QueryTrait, RelationTrait, Set,
    TransactionTrait,
};
use std::sync::Arc;
use uuid::Uuid;

pub type DbPool = Arc<DatabaseConnection>;

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

#[derive(serde::Serialize)]
pub struct EmailRecord {
    pub id: String,
    pub message_id: Option<String>,
    pub subject: Option<String>,
    pub date: Option<DateTime<Utc>>,
    pub headers: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub from: String,
    pub to: Vec<String>,         // From "To" header
    pub recipients: Vec<String>, // All SMTP envelope recipients
    pub size: u64,
    pub body_text: String,
    pub body_html: String,
    pub read: bool,
    pub attachments: Vec<EmailAttachmentRecord>,
}

#[derive(serde::Serialize, Clone)]
pub struct EmailListRecord {
    pub id: String,
    pub subject: Option<String>,
    pub date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub from: String,
    pub to: Vec<String>,
    pub read: bool,
    pub has_attachments: bool,
}

impl From<EmailRecord> for EmailListRecord {
    fn from(record: EmailRecord) -> Self {
        Self {
            id: record.id,
            subject: record.subject,
            date: record.date,
            created_at: record.created_at,
            from: record.from,
            to: record.to,
            read: record.read,
            has_attachments: !record.attachments.is_empty(),
        }
    }
}

#[derive(serde::Serialize)]
pub struct EmailAttachmentRecord {
    pub id: String,
    pub filename: Option<String>,
    pub mime_type: String,
    pub size: usize,
    pub content_id: Option<String>,
    pub headers: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

pub struct AttachmentContent {
    pub id: String,
    pub filename: Option<String>,
    pub mime_type: String,
    pub data: Vec<u8>,
}

pub async fn save_email(db: &DatabaseConnection, message: &Email) -> Result<String, DbErr> {
    let txn = db.begin().await?;

    // Generate attachment IDs first so we can process HTML with correct URLs
    let mut attachment_ids: Vec<String> = Vec::new();
    for _attachment in &message.attachments {
        attachment_ids.push(Uuid::new_v4().to_string());
    }

    // Process HTML to replace CID references with attachment URLs
    // Use empty base_url for relative paths (works for same-origin requests)
    let rendered_body_html = if !message.body_html.is_empty() {
        // Create temporary attachment records with IDs for processing
        let temp_attachments: Vec<(String, Option<String>)> = message
            .attachments
            .iter()
            .zip(attachment_ids.iter())
            .map(|(att, id)| (id.clone(), att.content_id.clone()))
            .collect();

        let mut processed_html = message.body_html.clone();
        let mut cid_map = std::collections::HashMap::new();
        for (attachment_id, content_id) in temp_attachments {
            if let Some(ref cid) = content_id {
                let attachment_url = format!("/api/attachments/{}", attachment_id);
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
            // Match src="cid:..." or src='cid:...' or src=cid:... (no backreferences needed)
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

    // Insert email using exec() to avoid fetching the inserted record
    let has_attachments = !message.attachments.is_empty();
    let email_model = emails::ActiveModel {
        id: Set(message.id.to_string()),
        message_id: Set(message.message_id.clone()),
        subject: Set(message.subject.clone()),
        date: Set(message.date),
        headers: Set(message
            .headers
            .as_ref()
            .map(|h| serde_json::to_string(h).unwrap())),
        from: Set(message.from.clone()),
        to: Set(serde_json::to_string(&message.to).unwrap()),
        size: Set(message.size as i32),
        raw_data: Set(message.data.clone()),
        body_text: Set(Some(message.body_text.clone())),
        body_html: Set(Some(message.body_html.clone())),
        rendered_body_html: Set(rendered_body_html),
        read: Set(false),
        has_attachments: Set(has_attachments),
        created_at: Set(Utc::now()),
    };
    Insert::one(email_model).exec(&txn).await?;

    for recipient_email in &message.recipients {
        if recipient_email.trim().is_empty() {
            continue;
        }

        let recipient_id = if let Some(existing) = recipients::Entity::find()
            .filter(recipients::Column::Email.eq(recipient_email.clone()))
            .one(&txn)
            .await?
        {
            existing.id
        } else {
            let new_id = Uuid::new_v4().to_string();
            let active = recipients::ActiveModel {
                id: Set(new_id.clone()),
                email: Set(recipient_email.clone()),
            };
            recipients::Entity::insert(active).exec(&txn).await?;
            new_id
        };

        let join = email_recipients::ActiveModel {
            email_id: Set(message.id.to_string()),
            recipient_id: Set(recipient_id),
        };
        // Ignore duplicate errors if the combination already exists.
        let _ = email_recipients::Entity::insert(join).exec(&txn).await;
    }

    // Insert attachments using the pre-generated IDs
    for (attachment, attachment_id) in message.attachments.iter().zip(attachment_ids.iter()) {
        let attachment_model = email_attachments::ActiveModel {
            id: Set(attachment_id.clone()),
            email_id: Set(message.id.to_string()),
            filename: Set(attachment.filename.clone()),
            mime_type: Set(attachment.mime_type.clone()),
            data: Set(attachment.data.clone()),
            size: Set(attachment.data.len() as i32),
            content_id: Set(attachment.content_id.clone()),
            headers: Set(attachment
                .headers
                .as_ref()
                .map(|h| serde_json::to_string(h).unwrap())),
            created_at: Set(Utc::now()),
        };
        Insert::one(attachment_model).exec(&txn).await?;
    }

    txn.commit().await?;
    Ok(message.id.to_string())
}

fn apply_search_filter(
    query: sea_orm::Select<emails::Entity>,
    search: Option<&str>,
) -> sea_orm::Select<emails::Entity> {
    if let Some(search_term) = search {
        let search_pattern = format!("%{}%", search_term);
        query.filter(
            sea_orm::Condition::any()
                .add(emails::Column::Subject.like(&search_pattern))
                .add(emails::Column::MessageId.like(&search_pattern))
                .add(emails::Column::From.like(&search_pattern))
                .add(emails::Column::To.like(&search_pattern))
                .add(emails::Column::BodyText.like(&search_pattern))
                .add(emails::Column::BodyHtml.like(&search_pattern)),
        )
    } else {
        query
    }
}

fn convert_emails_to_list_records(email_models: Vec<emails::Model>) -> Vec<EmailListRecord> {
    email_models
        .into_iter()
        .map(email_model_to_list_record)
        .collect()
}

fn email_model_to_record(
    email: emails::Model,
    attachments: Vec<email_attachments::Model>,
    recipients: Vec<String>,
) -> EmailRecord {
    let to: Vec<String> =
        serde_json::from_str(&email.to).unwrap_or_else(|_| vec![email.from.clone()]);

    EmailRecord {
        id: email.id,
        message_id: email.message_id,
        subject: email.subject,
        date: email.date,
        headers: email
            .headers
            .as_ref()
            .and_then(|h| serde_json::from_str(h).ok()),
        created_at: email.created_at,
        from: email.from,
        to,
        recipients,
        size: email.size as u64,
        body_text: email.body_text.unwrap_or_default(),
        body_html: email.body_html.unwrap_or_default(),
        read: email.read,
        attachments: attachments
            .into_iter()
            .map(|a| EmailAttachmentRecord {
                id: a.id,
                filename: a.filename,
                mime_type: a.mime_type,
                size: a.size as usize,
                content_id: a.content_id,
                headers: a
                    .headers
                    .as_ref()
                    .and_then(|h| serde_json::from_str(h).ok()),
                created_at: a.created_at,
            })
            .collect(),
    }
}

fn email_model_to_list_record(email: emails::Model) -> EmailListRecord {
    let to: Vec<String> =
        serde_json::from_str(&email.to).unwrap_or_else(|_| vec![email.from.clone()]);

    EmailListRecord {
        id: email.id,
        subject: email.subject,
        date: email.date,
        created_at: email.created_at,
        from: email.from,
        to,
        read: email.read,
        has_attachments: email.has_attachments,
    }
}

pub async fn get_all_emails(
    db: &DatabaseConnection,
    query_params: &ListQuery,
) -> Result<(Vec<EmailListRecord>, u64), DbErr> {
    let query = emails::Entity::find();
    let query = apply_search_filter(query, query_params.search.as_deref());
    let query = query.order_by_desc(emails::Column::CreatedAt);
    let paginator = query.paginate(db, query_params.per_page);
    let num_pages = paginator.num_pages().await?;
    let email_models = paginator.fetch_page(query_params.page - 1).await?;
    let records = convert_emails_to_list_records(email_models);
    Ok((records, num_pages))
}

pub async fn get_email_by_id(
    db: &DatabaseConnection,
    email_id: &str,
) -> Result<Option<EmailRecord>, DbErr> {
    let email = emails::Entity::find_by_id(email_id.to_string())
        .one(db)
        .await?;

    if let Some(email) = email {
        let attachments = email_attachments::Entity::find()
            .filter(email_attachments::Column::EmailId.eq(&email.id))
            .all(db)
            .await?;

        // Load recipients for this email from the join table.
        let recipient_rows = email_recipients::Entity::find()
            .filter(email_recipients::Column::EmailId.eq(&email.id))
            .find_also_related(recipients::Entity)
            .all(db)
            .await?;

        let recipients_vec: Vec<String> = recipient_rows
            .into_iter()
            .filter_map(|(_join, recipient_opt)| recipient_opt.map(|r| r.email))
            .collect();

        Ok(Some(email_model_to_record(
            email,
            attachments,
            recipients_vec,
        )))
    } else {
        Ok(None)
    }
}

pub async fn delete_email_by_id(db: &DatabaseConnection, email_id: &str) -> Result<u64, DbErr> {
    let result = emails::Entity::delete_by_id(email_id.to_string())
        .exec(db)
        .await?;
    Ok(result.rows_affected)
}

pub async fn delete_all_emails(db: &DatabaseConnection) -> Result<u64, DbErr> {
    let result = emails::Entity::delete_many().exec(db).await?;
    Ok(result.rows_affected)
}

pub async fn mark_email_read(
    db: &DatabaseConnection,
    email_id: &str,
    read: bool,
) -> Result<u64, DbErr> {
    let email = emails::Entity::find_by_id(email_id.to_string())
        .one(db)
        .await?
        .ok_or_else(|| DbErr::RecordNotFound(format!("Email with id {} not found", email_id)))?;

    let mut email: emails::ActiveModel = email.into();
    email.read = Set(read);
    email.update(db).await?;

    Ok(1)
}

pub async fn get_attachment_by_id(
    db: &DatabaseConnection,
    attachment_id: &str,
) -> Result<Option<AttachmentContent>, DbErr> {
    let attachment = email_attachments::Entity::find_by_id(attachment_id.to_string())
        .one(db)
        .await?;

    if let Some(attachment) = attachment {
        Ok(Some(AttachmentContent {
            id: attachment.id,
            filename: attachment.filename,
            mime_type: attachment.mime_type,
            data: attachment.data,
        }))
    } else {
        Ok(None)
    }
}

pub async fn get_raw_data_by_id(
    db: &DatabaseConnection,
    email_id: &str,
) -> Result<Option<String>, DbErr> {
    let email = emails::Entity::find_by_id(email_id.to_string())
        .one(db)
        .await?;

    Ok(email.map(|e| e.raw_data))
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

#[derive(Debug, FromQueryResult)]
struct RecipientStatsRow {
    recipient: String,
    count: i64,
}

pub async fn get_email_stats(db: &DatabaseConnection) -> Result<EmailStats, DbErr> {
    let inbox_count = emails::Entity::find().count(db).await?;

    use sea_orm::JoinType;

    // Aggregate counts per recipient using GROUP BY in the database.
    // Start from the join table to avoid ambiguous recipients aliases.
    let rows: Vec<RecipientStatsRow> = email_recipients::Entity::find()
        .join(
            JoinType::InnerJoin,
            email_recipients::Relation::Recipient.def(),
        )
        .select_only()
        .column_as(recipients::Column::Email, "recipient")
        .column_as(email_recipients::Column::EmailId.count(), "count")
        .group_by(recipients::Column::Email)
        .order_by_asc(recipients::Column::Email)
        .into_model::<RecipientStatsRow>()
        .all(db)
        .await?;

    let recipients: Vec<RecipientStats> = rows
        .into_iter()
        .map(|row| RecipientStats {
            recipient: row.recipient,
            count: row.count as u64,
        })
        .collect();

    Ok(EmailStats {
        inbox: inbox_count,
        recipients,
    })
}

pub async fn get_emails_by_recipient(
    db: &DatabaseConnection,
    recipient_email: &str,
    query_params: &ListQuery,
) -> Result<(Vec<EmailListRecord>, u64), DbErr> {
    // Find the recipient row first. If it doesn't exist, return empty results.
    if let Some(recipient) = recipients::Entity::find()
        .filter(recipients::Column::Email.eq(recipient_email))
        .one(db)
        .await?
    {
        // Use a subquery on the join table to avoid ambiguous aliases when joining.
        let subquery = email_recipients::Entity::find()
            .select_only()
            .column(email_recipients::Column::EmailId)
            .filter(email_recipients::Column::RecipientId.eq(recipient.id))
            .into_query();

        let query = emails::Entity::find().filter(emails::Column::Id.in_subquery(subquery));

        let query = apply_search_filter(query, query_params.search.as_deref());
        let query = query.order_by_desc(emails::Column::CreatedAt);
        let paginator = query.paginate(db, query_params.per_page);
        let num_pages = paginator.num_pages().await?;
        let email_models = paginator.fetch_page(query_params.page - 1).await?;
        let records = convert_emails_to_list_records(email_models);

        Ok((records, num_pages))
    } else {
        Ok((Vec::new(), 0))
    }
}
