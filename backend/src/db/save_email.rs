use crate::{
    compression,
    db::{DbConnection, DbError},
    models::{Attachment, Email},
    schema, smtp,
};
use chrono::Utc;
use diesel::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

pub fn save_email(conn: &mut DbConnection, message: &smtp::Email) -> Result<String, DbError> {
    conn.transaction::<_, DbError, _>(|conn| {
        let attachment_ids = generate_attachment_ids(&message.attachments);
        let rendered_body_html =
            process_html_body(&message.body_html, &message.attachments, &attachment_ids);
        let now = Utc::now().naive_utc();
        let new_email = create_email_record(message, rendered_body_html, now)?;

        diesel::insert_into(schema::emails::table)
            .values(&new_email)
            .execute(conn)?;

        save_recipients(conn, &new_email.id, &message.to)?;
        save_attachments(
            conn,
            &new_email.id,
            &message.attachments,
            &attachment_ids,
            now,
        )?;

        Ok(new_email.id)
    })
}

fn generate_attachment_ids(attachments: &[smtp::EmailAttachment]) -> Vec<String> {
    attachments
        .iter()
        .map(|_| Uuid::new_v4().to_string())
        .collect()
}

fn process_html_body(
    body_html: &str,
    attachments: &[smtp::EmailAttachment],
    attachment_ids: &[String],
) -> Option<String> {
    if body_html.is_empty() {
        return None;
    }

    let cid_map = build_cid_map(attachments, attachment_ids);
    if cid_map.is_empty() {
        return Some(body_html.to_string());
    }

    use regex::Regex;
    let re = Regex::new(r#"(?i)src\s*=\s*(["']?)cid:([^"'\s>]+)"#).unwrap();
    let processed_html = re
        .replace_all(body_html, |caps: &regex::Captures| {
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

    Some(processed_html)
}

fn build_cid_map(
    attachments: &[smtp::EmailAttachment],
    attachment_ids: &[String],
) -> HashMap<String, String> {
    let mut cid_map = HashMap::new();

    for (att, id) in attachments.iter().zip(attachment_ids.iter()) {
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

    cid_map
}

fn create_email_record(
    message: &smtp::Email,
    rendered_body_html: Option<String>,
    now: chrono::NaiveDateTime,
) -> Result<Email, DbError> {
    let compressed_data = compression::compress(message.data.as_bytes())?;
    Ok(Email {
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
        compressed_data,
        body_text: Some(message.body_text.clone()),
        body_html: Some(message.body_html.clone()),
        rendered_body_html,
        read: false,
        has_attachments: !message.attachments.is_empty(),
        created_at: now,
    })
}

fn save_recipients(
    conn: &mut DbConnection,
    email_id: &str,
    recipient_emails: &[String],
) -> Result<(), DbError> {
    for recipient_email in recipient_emails {
        if recipient_email.trim().is_empty() {
            continue;
        }

        let recipient_id = get_or_create_recipient(conn, recipient_email)?;
        link_recipient_to_email(conn, email_id, &recipient_id)?;
    }

    Ok(())
}

fn get_or_create_recipient(conn: &mut DbConnection, email: &str) -> Result<String, DbError> {
    match schema::recipients::table
        .filter(schema::recipients::email.eq(email))
        .select(schema::recipients::id)
        .first::<String>(conn)
    {
        Ok(id) => Ok(id),
        Err(_) => {
            let new_id = Uuid::new_v4().to_string();
            diesel::insert_into(schema::recipients::table)
                .values((
                    schema::recipients::id.eq(&new_id),
                    schema::recipients::email.eq(email),
                ))
                .execute(conn)?;
            Ok(new_id)
        }
    }
}

fn link_recipient_to_email(
    conn: &mut DbConnection,
    email_id: &str,
    recipient_id: &str,
) -> Result<(), DbError> {
    diesel::insert_into(schema::email_recipients::table)
        .values((
            schema::email_recipients::email_id.eq(email_id),
            schema::email_recipients::recipient_id.eq(recipient_id),
        ))
        .execute(conn)?;
    Ok(())
}

fn save_attachments(
    conn: &mut DbConnection,
    email_id: &str,
    attachments: &[smtp::EmailAttachment],
    attachment_ids: &[String],
    now: chrono::NaiveDateTime,
) -> Result<(), DbError> {
    for (attachment, attachment_id) in attachments.iter().zip(attachment_ids.iter()) {
        let compressed_data = compression::compress(&attachment.data)?;

        let new_attachment = Attachment {
            id: attachment_id.clone(),
            email_id: email_id.to_string(),
            filename: attachment.filename.clone(),
            content_type: attachment.content_type.clone(),
            compressed_data,
            size: attachment.data.len() as i32,
            content_id: attachment.content_id.clone(),
            disposition: attachment.disposition.clone(),
            created_at: now,
        };
        diesel::insert_into(schema::attachments::table)
            .values(&new_attachment)
            .execute(conn)?;
    }

    Ok(())
}
