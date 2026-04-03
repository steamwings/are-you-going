use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Rsvps::Table)
                    .add_column(text_null(Rsvps::AllergiesText))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Rsvps::Table)
                    .drop_column(Rsvps::AllergiesText)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Rsvps {
    Table,
    AllergiesText,
}
