use sea_orm::{entity::prelude::*, ActiveValue};

use super::_entities::rsvps::{self, ActiveModel, Entity};

impl ActiveModel {
    pub fn new(
        event_id: i32,
        name: &str,
        phone_number: &str,
        party_size: i32,
        kids_count: i32,
        sms_opt_in: bool,
        allergies_text: Option<String>,
        custom_response: Option<String>,
    ) -> Self {
        let has_allergies = allergies_text.as_ref().map_or(false, |s| !s.is_empty());
        Self {
            event_id: ActiveValue::set(event_id),
            name: ActiveValue::set(name.to_string()),
            phone_number: ActiveValue::set(phone_number.to_string()),
            party_size: ActiveValue::set(party_size),
            kids_count: ActiveValue::set(kids_count),
            sms_opt_in: ActiveValue::set(sms_opt_in),
            has_allergies: ActiveValue::set(has_allergies),
            allergies_text: ActiveValue::set(allergies_text),
            custom_response: ActiveValue::set(custom_response),
            ..Default::default()
        }
    }
}

pub async fn find_by_event_and_phone(
    db: &DatabaseConnection,
    event_id: i32,
    phone: &str,
) -> Result<Option<rsvps::Model>, DbErr> {
    Entity::find()
        .filter(rsvps::Column::EventId.eq(event_id))
        .filter(rsvps::Column::PhoneNumber.eq(phone))
        .one(db)
        .await
}

pub async fn find_opted_in_for_event(
    db: &DatabaseConnection,
    event_id: i32,
) -> Result<Vec<rsvps::Model>, DbErr> {
    Entity::find()
        .filter(rsvps::Column::EventId.eq(event_id))
        .filter(rsvps::Column::SmsOptIn.eq(true))
        .all(db)
        .await
}

pub async fn find_by_event(
    db: &DatabaseConnection,
    event_id: i32,
) -> Result<Vec<rsvps::Model>, DbErr> {
    Entity::find()
        .filter(rsvps::Column::EventId.eq(event_id))
        .all(db)
        .await
}

pub async fn opt_out_by_phone(
    db: &DatabaseConnection,
    phone: &str,
) -> Result<(), DbErr> {
    Entity::update_many()
        .filter(rsvps::Column::PhoneNumber.eq(phone))
        .col_expr(rsvps::Column::SmsOptIn, Expr::value(false))
        .exec(db)
        .await?;
    Ok(())
}

pub fn normalize_phone(raw: &str) -> Result<String, String> {
    crate::models::phone::normalize_phone(raw)
}
