use diesel::prelude::*;

use crate::{
    compression,
    db::{AttachmentContent, DbConnection, DbError},
    models::Attachment,
    schema,
};

pub fn get_attachment(
    conn: &mut DbConnection,
    attachment_id: &str,
) -> Result<AttachmentContent, DbError> {
    let attachment = schema::attachments::table
        .filter(schema::attachments::id.eq(attachment_id))
        .first::<Attachment>(conn)?;

    let data = compression::decompress(&attachment.compressed_data)?;

    Ok(AttachmentContent {
        id: attachment.id,
        filename: attachment.filename,
        content_type: attachment.content_type,
        data,
    })
}
