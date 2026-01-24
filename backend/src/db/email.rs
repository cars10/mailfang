use diesel::prelude::*;

use crate::{
    compression,
    db::{
        AttachmentPartial, AttachmentRecord, DbConnection, DbError, EmailPartial, EmailRecord,
        vacuum_database,
    },
    models::Header,
    schema,
};

pub fn get_email(conn: &mut DbConnection, email_id: &str) -> Result<EmailRecord, DbError> {
    let email = EmailPartial::query()
        .filter(schema::emails::id.eq(email_id))
        .first::<EmailPartial>(conn)?;

    let attachments = AttachmentPartial::query()
        .filter(schema::attachments::email_id.eq(&email.id))
        .load::<AttachmentPartial>(conn)?;

    let recipients: Vec<String> = schema::email_envelope_recipients::table
        .inner_join(schema::envelope_recipients::table)
        .filter(schema::email_envelope_recipients::email_id.eq(&email.id))
        .select(schema::envelope_recipients::email)
        .load(conn)?;

    let headers: Vec<Header> = schema::headers::table
        .filter(schema::headers::email_id.eq(&email.id))
        .load::<Header>(conn)?;

    let mut grouped_headers: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for header in headers {
        grouped_headers
            .entry(header.name)
            .or_insert_with(Vec::new)
            .push(header.value);
    }

    let attachment_records = attachments
        .into_iter()
        .map(|att| AttachmentRecord {
            id: att.id,
            filename: att.filename,
            content_type: att.content_type,
            size: att.size,
            content_id: att.content_id,
            disposition: att.disposition,
            created_at: att.created_at,
        })
        .collect();

    Ok(EmailRecord {
        id: email.id,
        message_id: email.message_id,
        subject: email.subject,
        date: email.date,
        headers: grouped_headers,
        created_at: email.created_at,
        envelope_from: email.envelope_from,
        size: email.size,
        body_text: email.body_text,
        body_html: email.body_html,
        read: email.read,
        recipients,
        attachments: attachment_records,
    })
}

pub fn mark_email_read(
    conn: &mut DbConnection,
    email_id: &str,
    read: bool,
) -> Result<usize, DbError> {
    conn.transaction::<_, DbError, _>(|conn| {
        let affected =
            diesel::update(schema::emails::table.filter(schema::emails::id.eq(email_id)))
                .set(schema::emails::read.eq(read))
                .execute(conn)?;
        Ok(affected)
    })
}

pub fn delete_email(conn: &mut DbConnection, email_id: &str) -> Result<usize, DbError> {
    let affected = conn.transaction::<_, DbError, _>(|conn| {
        diesel::delete(schema::emails::table.filter(schema::emails::id.eq(email_id)))
            .execute(conn)
            .map_err(DbError::from)
    })?;

    if affected > 0 {
        vacuum_database(conn).map_err(DbError::from)?;
    }

    Ok(affected)
}

pub fn get_rendered_data(conn: &mut DbConnection, email_id: &str) -> Result<String, DbError> {
    let data = schema::emails::table
        .filter(schema::emails::id.eq(email_id))
        .select(schema::emails::rendered_body_html)
        .first::<Option<String>>(conn)?;

    data.ok_or_else(|| DbError::Diesel(diesel::result::Error::NotFound))
}

pub fn get_raw_data(conn: &mut DbConnection, email_id: &str) -> Result<String, DbError> {
    let compressed_data = schema::emails::table
        .filter(schema::emails::id.eq(email_id))
        .select(schema::emails::compressed_data)
        .first::<Vec<u8>>(conn)?;

    let decompressed_bytes = compression::decompress(&compressed_data)?;
    String::from_utf8(decompressed_bytes).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Invalid UTF-8: {}", e),
        )
        .into()
    })
}
