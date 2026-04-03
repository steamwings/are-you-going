use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "rsvps")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub event_id: i32,
    pub name: String,
    pub phone_number: String,
    pub party_size: i32,
    pub kids_count: i32,
    pub sms_opt_in: bool,
    pub has_allergies: bool,
    #[sea_orm(column_type = "Text", nullable)]
    pub allergies_text: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub custom_response: Option<String>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::events::Entity",
        from = "Column::EventId",
        to = "super::events::Column::Id"
    )]
    Events,
    #[sea_orm(has_many = "super::magic_links::Entity")]
    MagicLinks,
    #[sea_orm(has_many = "super::reminder_sends::Entity")]
    ReminderSends,
}

impl Related<super::events::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Events.def()
    }
}

impl Related<super::magic_links::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MagicLinks.def()
    }
}

impl Related<super::reminder_sends::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ReminderSends.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
