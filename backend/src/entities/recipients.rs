use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "recipients")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    #[sea_orm(unique)]
    pub email: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::email_recipients::Entity")]
    EmailRecipients,
}

impl Related<super::email_recipients::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::EmailRecipients.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
