use diesel::prelude::*;

use crate::{
    db::{AttachmentContent, DbConnection},
    models::Attachment,
    schema,
    web::error::DieselError,
};

pub fn get_attachment(
    conn: &mut DbConnection,
    attachment_id: &str,
) -> Result<Option<AttachmentContent>, DieselError> {
    let attachment = schema::attachments::table
        .filter(schema::attachments::id.eq(attachment_id))
        .first::<Attachment>(conn)
        .optional()?;

    Ok(attachment.map(|att| AttachmentContent {
        id: att.id,
        filename: att.filename,
        content_type: att.content_type,
        data: att.data,
    }))
}
