use loco_rs::schema::table_auto;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                table_auto(ReminderSends::Table)
                    .col(pk_auto(ReminderSends::Id))
                    .col(integer(ReminderSends::EventReminderId))
                    .col(integer(ReminderSends::RsvpId))
                    .col(timestamp(ReminderSends::SentAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_reminder_sends_event_reminder_id")
                            .from(ReminderSends::Table, ReminderSends::EventReminderId)
                            .to(
                                super::m20250329_000002_create_event_reminders::EventReminders::Table,
                                super::m20250329_000002_create_event_reminders::EventReminders::Id,
                            )
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_reminder_sends_rsvp_id")
                            .from(ReminderSends::Table, ReminderSends::RsvpId)
                            .to(
                                super::m20250329_000003_create_rsvps::Rsvps::Table,
                                super::m20250329_000003_create_rsvps::Rsvps::Id,
                            )
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_reminder_sends_unique")
                    .table(ReminderSends::Table)
                    .col(ReminderSends::EventReminderId)
                    .col(ReminderSends::RsvpId)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ReminderSends::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum ReminderSends {
    Table,
    Id,
    EventReminderId,
    RsvpId,
    SentAt,
}
