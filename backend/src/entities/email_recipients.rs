use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "email_recipients")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub email_id: String,
    #[sea_orm(primary_key)]
    pub recipient_id: String,
    #[sea_orm(belongs_to, from = "email_id", to = "id")]
    pub email: HasOne<super::emails::Entity>,
    #[sea_orm(belongs_to, from = "recipient_id", to = "id")]
    pub recipient: HasOne<super::recipients::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
