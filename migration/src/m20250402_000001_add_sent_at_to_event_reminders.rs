use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(EventReminders::Table)
                    .add_column(timestamp_null(EventReminders::SentAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(EventReminders::Table)
                    .drop_column(EventReminders::SentAt)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum EventReminders {
    Table,
    SentAt,
}
