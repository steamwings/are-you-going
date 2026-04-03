use sea_orm::{entity::prelude::*, ActiveValue};

use super::_entities::sms_opt_outs::{self, ActiveModel, Entity};

pub async fn is_opted_out(db: &DatabaseConnection, phone: &str) -> Result<bool, DbErr> {
    let count = Entity::find()
        .filter(sms_opt_outs::Column::PhoneNumber.eq(phone))
        .count(db)
        .await?;
    Ok(count > 0)
}

pub async fn remove_opt_out(db: &DatabaseConnection, phone: &str) -> Result<(), DbErr> {
    Entity::delete_many()
        .filter(sms_opt_outs::Column::PhoneNumber.eq(phone))
        .exec(db)
        .await?;
    Ok(())
}

pub async fn upsert_opt_out(db: &DatabaseConnection, phone: &str) -> Result<(), DbErr> {
    let existing = Entity::find()
        .filter(sms_opt_outs::Column::PhoneNumber.eq(phone))
        .one(db)
        .await?;

    if existing.is_none() {
        let model = ActiveModel {
            phone_number: ActiveValue::set(phone.to_string()),
            ..Default::default()
        };
        model.insert(db).await?;
    }
    Ok(())
}
