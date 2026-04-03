use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "event_reminders")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub event_id: i32,
    pub remind_at: DateTimeWithTimeZone,
    #[sea_orm(column_type = "Text")]
    pub message: String,
    pub sent_at: Option<DateTimeWithTimeZone>,
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
    #[sea_orm(has_many = "super::reminder_sends::Entity")]
    ReminderSends,
}

impl Related<super::events::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Events.def()
    }
}

impl Related<super::reminder_sends::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ReminderSends.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
