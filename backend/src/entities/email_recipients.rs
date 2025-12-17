use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "email_recipients")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub email_id: String,
    #[sea_orm(primary_key)]
    pub recipient_id: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::emails::Entity",
        from = "Column::EmailId",
        to = "super::emails::Column::Id"
    )]
    Email,
    #[sea_orm(
        belongs_to = "super::recipients::Entity",
        from = "Column::RecipientId",
        to = "super::recipients::Column::Id"
    )]
    Recipient,
}

impl Related<super::emails::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Email.def()
    }
}

impl Related<super::recipients::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Recipient.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
