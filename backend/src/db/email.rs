use diesel::prelude::*;

use crate::{
    db::{
        AttachmentPartial, AttachmentRecord, DbConnection, EmailPartial, EmailRecord,
        vacuum_database,
    },
    schema,
    web::error::DieselError,
};

pub fn get_email(
    conn: &mut DbConnection,
    email_id: &str,
) -> Result<Option<EmailRecord>, DieselError> {
    let email = EmailPartial::query()
        .filter(schema::emails::id.eq(email_id))
        .first::<EmailPartial>(conn)
        .optional()?;

    if let Some(email) = email {
        let attachments = AttachmentPartial::query()
            .filter(schema::attachments::email_id.eq(&email.id))
            .load::<AttachmentPartial>(conn)?;

        let recipients: Vec<String> = schema::email_recipients::table
            .inner_join(schema::recipients::table)
            .filter(schema::email_recipients::email_id.eq(&email.id))
            .select(schema::recipients::email)
            .load(conn)?;

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

pub fn mark_email_read(
    conn: &mut DbConnection,
    email_id: &str,
    read: bool,
) -> Result<usize, DieselError> {
    conn.transaction::<_, DieselError, _>(|conn| {
        let affected = diesel::update(schema::emails::table.filter(schema::emails::id.eq(email_id)))
            .set(schema::emails::read.eq(read))
            .execute(conn)?;
        Ok(affected)
    })
}

pub fn delete_email(conn: &mut DbConnection, email_id: &str) -> Result<usize, DieselError> {
    conn.transaction::<_, DieselError, _>(|conn| {
        let affected = diesel::delete(schema::emails::table.filter(schema::emails::id.eq(email_id)))
            .execute(conn)?;

        if affected > 0 {
            vacuum_database(conn)?;
        }

        Ok(affected)
    })
}

pub fn get_rendered_data(
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

pub fn get_raw_data(
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
