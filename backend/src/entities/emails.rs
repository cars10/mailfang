use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "emails")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub message_id: Option<String>,
    pub subject: Option<String>,
    pub date: Option<DateTime<Utc>>,
    pub headers: Option<String>, // Stored as JSON string in DB
    pub from: String,
    pub to: String,         // Stored as JSON string in DB - from "To" header
    pub recipients: String, // Stored as JSON string in DB - all SMTP envelope recipients
    pub size: i32,
    pub raw_data: String,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub rendered_body_html: Option<String>,
    pub read: bool,
    pub archived: bool,
    pub has_attachments: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::email_attachments::Entity")]
    EmailAttachments,
}

impl Related<super::email_attachments::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::EmailAttachments.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
