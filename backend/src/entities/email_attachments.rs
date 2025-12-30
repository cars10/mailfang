use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "email_attachments")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub email_id: String,
    pub filename: Option<String>,
    pub mime_type: String,
    pub data: Vec<u8>,
    pub size: i32,
    pub content_id: Option<String>,
    pub headers: Option<String>, // Stored as JSON string in DB
    pub created_at: DateTime<Utc>,
    #[sea_orm(belongs_to, from = "email_id", to = "id")]
    pub email: HasOne<super::emails::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
