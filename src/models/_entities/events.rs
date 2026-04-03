use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "events")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    #[sea_orm(column_type = "Text")]
    pub description: String,
    #[sea_orm(unique)]
    pub slug: String,
    #[sea_orm(column_type = "Text")]
    pub field_config: String,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::event_reminders::Entity")]
    EventReminders,
    #[sea_orm(has_many = "super::rsvps::Entity")]
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
