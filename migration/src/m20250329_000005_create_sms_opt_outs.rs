use loco_rs::schema::table_auto;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                table_auto(SmsOptOuts::Table)
                    .col(pk_auto(SmsOptOuts::Id))
                    .col(string_uniq(SmsOptOuts::PhoneNumber))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SmsOptOuts::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum SmsOptOuts {
    Table,
    Id,
    PhoneNumber,
}
