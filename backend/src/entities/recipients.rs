use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "recipients")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    #[sea_orm(unique)]
    pub email: String,
    #[sea_orm(has_many, via = "email_recipients", from = "id", to = "recipient_id")]
    pub emails: HasMany<super::emails::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
