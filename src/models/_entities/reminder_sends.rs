use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "reminder_sends")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub event_reminder_id: i32,
    pub rsvp_id: i32,
    pub sent_at: DateTimeWithTimeZone,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::event_reminders::Entity",
        from = "Column::EventReminderId",
        to = "super::event_reminders::Column::Id"
    )]
    EventReminders,
    #[sea_orm(
        belongs_to = "super::rsvps::Entity",
        from = "Column::RsvpId",
        to = "super::rsvps::Column::Id"
    )]
    Rsvps,
}

impl Related<super::event_reminders::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::EventReminders.def()
    }
}

impl Related<super::rsvps::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Rsvps.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
