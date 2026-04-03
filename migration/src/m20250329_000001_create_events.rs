use loco_rs::schema::table_auto;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                table_auto(Events::Table)
                    .col(pk_auto(Events::Id))
                    .col(string(Events::Name))
                    .col(text(Events::Description))
                    .col(string_uniq(Events::Slug))
                    .col(text(Events::FieldConfig))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Events::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Events {
    Table,
    Id,
    Name,
    Description,
    Slug,
    FieldConfig,
}
