use chrono::Utc;
use sea_orm::{entity::prelude::*, ActiveValue};
use uuid::Uuid;

use super::_entities::magic_links::{self, ActiveModel, Entity};

impl ActiveModel {
    pub fn new(rsvp_id: i32, expiration_seconds: i64) -> Self {
        let token = Uuid::new_v4().to_string();
        let expires_at = Utc::now() + chrono::Duration::seconds(expiration_seconds);
        Self {
            rsvp_id: ActiveValue::set(rsvp_id),
            token: ActiveValue::set(token),
            expires_at: ActiveValue::set(expires_at.into()),
            ..Default::default()
        }
    }
}

pub async fn find_valid_by_token(
    db: &DatabaseConnection,
    token: &str,
) -> Result<Option<magic_links::Model>, DbErr> {
    let now: DateTimeWithTimeZone = Utc::now().into();
    Entity::find()
        .filter(magic_links::Column::Token.eq(token))
        .filter(magic_links::Column::ExpiresAt.gt(now))
        .one(db)
        .await
}
