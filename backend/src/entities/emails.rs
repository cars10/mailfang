use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "emails")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub message_id: Option<String>,
    pub subject: Option<String>,
    pub date: Option<DateTime<Utc>>,
    pub headers: Option<String>,
    pub from: String,
    pub size: i32,
    pub raw_data: String,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub rendered_body_html: Option<String>,
    pub read: bool,
    pub has_attachments: bool,
    pub created_at: DateTime<Utc>,
    #[sea_orm(has_many)]
    pub attachments: HasMany<super::email_attachments::Entity>,
    #[sea_orm(has_many, via = "email_recipients", from = "id", to = "email_id")]
    pub recipients: HasMany<super::recipients::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
