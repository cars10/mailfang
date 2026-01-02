use diesel::prelude::*;

use crate::{
    db::{AttachmentContent, DbConnection},
    models::EmailAttachment,
    schema,
    web::error::DieselError,
};

pub fn get_attachment(
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
