use loco_rs::schema::table_auto;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                table_auto(Rsvps::Table)
                    .col(pk_auto(Rsvps::Id))
                    .col(integer(Rsvps::EventId))
                    .col(string(Rsvps::Name))
                    .col(string(Rsvps::PhoneNumber))
                    .col(integer(Rsvps::PartySize).default(1))
                    .col(integer(Rsvps::KidsCount).default(0))
                    .col(boolean(Rsvps::SmsOptIn).default(false))
                    .col(boolean(Rsvps::HasAllergies).default(false))
                    .col(text_null(Rsvps::CustomResponse))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_rsvps_event_id")
                            .from(Rsvps::Table, Rsvps::EventId)
                            .to(
                                super::m20250329_000001_create_events::Events::Table,
                                super::m20250329_000001_create_events::Events::Id,
                            )
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_rsvps_event_phone")
                    .table(Rsvps::Table)
                    .col(Rsvps::EventId)
                    .col(Rsvps::PhoneNumber)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Rsvps::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Rsvps {
    Table,
    Id,
    EventId,
    Name,
    PhoneNumber,
    PartySize,
    KidsCount,
    SmsOptIn,
    HasAllergies,
    CustomResponse,
}
