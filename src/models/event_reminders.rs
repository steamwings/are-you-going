use chrono::Utc;
use sea_orm::{entity::prelude::*, ActiveValue, IntoActiveModel};

use super::_entities::event_reminders::{self, ActiveModel, Entity};

impl ActiveModel {
    pub fn new(event_id: i32, remind_at: DateTimeWithTimeZone, message: &str) -> Self {
        Self {
            event_id: ActiveValue::set(event_id),
            remind_at: ActiveValue::set(remind_at),
            message: ActiveValue::set(message.to_string()),
            ..Default::default()
        }
    }
}

pub async fn find_due_reminders(
    db: &DatabaseConnection,
    now: DateTimeWithTimeZone,
) -> Result<Vec<event_reminders::Model>, DbErr> {
    Entity::find()
        .filter(event_reminders::Column::RemindAt.lte(now))
        .filter(event_reminders::Column::SentAt.is_null())
        .all(db)
        .await
}

pub async fn mark_sent(db: &DatabaseConnection, reminder: event_reminders::Model) -> Result<(), DbErr> {
    let mut active = reminder.into_active_model();
    active.sent_at = ActiveValue::set(Some(Utc::now().into()));
    active.update(db).await?;
    Ok(())
}

pub async fn find_by_event(
    db: &DatabaseConnection,
    event_id: i32,
) -> Result<Vec<event_reminders::Model>, DbErr> {
    Entity::find()
        .filter(event_reminders::Column::EventId.eq(event_id))
        .all(db)
        .await
}

pub async fn find_unsent_by_event(
    db: &DatabaseConnection,
    event_id: i32,
) -> Result<Vec<event_reminders::Model>, DbErr> {
    Entity::find()
        .filter(event_reminders::Column::EventId.eq(event_id))
        .filter(event_reminders::Column::SentAt.is_null())
        .all(db)
        .await
}
