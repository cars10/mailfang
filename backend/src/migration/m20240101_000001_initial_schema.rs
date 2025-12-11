use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Emails::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Emails::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(Emails::MessageId).string().null())
                    .col(ColumnDef::new(Emails::Subject).string().null())
                    .col(ColumnDef::new(Emails::Date).date_time().null())
                    .col(ColumnDef::new(Emails::Headers).text().null())
                    .col(ColumnDef::new(Emails::From).string().not_null())
                    .col(ColumnDef::new(Emails::To).string().not_null())
                    .col(ColumnDef::new(Emails::Recipients).string().not_null())
                    .col(ColumnDef::new(Emails::Size).integer().not_null())
                    .col(ColumnDef::new(Emails::RawData).text().not_null())
                    .col(ColumnDef::new(Emails::BodyText).text().null())
                    .col(ColumnDef::new(Emails::BodyHtml).text().null())
                    .col(ColumnDef::new(Emails::RenderedBodyHtml).text().null())
                    .col(
                        ColumnDef::new(Emails::Read)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(Emails::HasAttachments)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(Emails::CreatedAt)
                            .date_time()
                            .not_null()
                            .default("(datetime('now'))"),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(EmailAttachments::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(EmailAttachments::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(EmailAttachments::EmailId)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(EmailAttachments::Filename).string().null())
                    .col(
                        ColumnDef::new(EmailAttachments::MimeType)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(EmailAttachments::Data).binary().not_null())
                    .col(ColumnDef::new(EmailAttachments::Size).integer().not_null())
                    .col(ColumnDef::new(EmailAttachments::ContentId).string().null())
                    .col(ColumnDef::new(EmailAttachments::Headers).text().null())
                    .col(
                        ColumnDef::new(EmailAttachments::CreatedAt)
                            .date_time()
                            .not_null()
                            .default("(datetime('now'))"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_email_attachments_email_id")
                            .from(EmailAttachments::Table, EmailAttachments::EmailId)
                            .to(Emails::Table, Emails::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(EmailAttachments::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Emails::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Emails {
    Table,
    Id,
    MessageId,
    Subject,
    Date,
    Headers,
    From,
    To,
    Recipients,
    Size,
    RawData,
    BodyText,
    BodyHtml,
    RenderedBodyHtml,
    Read,
    HasAttachments,
    CreatedAt,
}

#[derive(DeriveIden)]
enum EmailAttachments {
    Table,
    Id,
    EmailId,
    Filename,
    MimeType,
    Data,
    Size,
    ContentId,
    Headers,
    CreatedAt,
}
