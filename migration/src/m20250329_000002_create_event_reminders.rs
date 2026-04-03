use loco_rs::schema::table_auto;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                table_auto(EventReminders::Table)
                    .col(pk_auto(EventReminders::Id))
                    .col(integer(EventReminders::EventId))
                    .col(timestamp(EventReminders::RemindAt))
                    .col(text(EventReminders::Message))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_event_reminders_event_id")
                            .from(EventReminders::Table, EventReminders::EventId)
                            .to(
                                super::m20250329_000001_create_events::Events::Table,
                                super::m20250329_000001_create_events::Events::Id,
                            )
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(EventReminders::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum EventReminders {
    Table,
    Id,
    EventId,
    RemindAt,
    Message,
}
