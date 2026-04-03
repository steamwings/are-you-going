use chrono::Utc;
use sea_orm::{entity::prelude::*, ActiveValue};

use super::_entities::reminder_sends::{self, ActiveModel, Entity};

pub async fn already_sent(
    db: &DatabaseConnection,
    event_reminder_id: i32,
    rsvp_id: i32,
) -> Result<bool, DbErr> {
    let count = Entity::find()
        .filter(reminder_sends::Column::EventReminderId.eq(event_reminder_id))
        .filter(reminder_sends::Column::RsvpId.eq(rsvp_id))
        .count(db)
        .await?;
    Ok(count > 0)
}

pub async fn record_send(
    db: &DatabaseConnection,
    event_reminder_id: i32,
    rsvp_id: i32,
) -> Result<(), DbErr> {
    let model = ActiveModel {
        event_reminder_id: ActiveValue::set(event_reminder_id),
        rsvp_id: ActiveValue::set(rsvp_id),
        sent_at: ActiveValue::set(Utc::now().into()),
        ..Default::default()
    };
    model.insert(db).await?;
    Ok(())
}
