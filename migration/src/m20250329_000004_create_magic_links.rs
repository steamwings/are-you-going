use loco_rs::schema::table_auto;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                table_auto(MagicLinks::Table)
                    .col(pk_auto(MagicLinks::Id))
                    .col(integer(MagicLinks::RsvpId))
                    .col(string_uniq(MagicLinks::Token))
                    .col(timestamp(MagicLinks::ExpiresAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_magic_links_rsvp_id")
                            .from(MagicLinks::Table, MagicLinks::RsvpId)
                            .to(
                                super::m20250329_000003_create_rsvps::Rsvps::Table,
                                super::m20250329_000003_create_rsvps::Rsvps::Id,
                            )
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(MagicLinks::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum MagicLinks {
    Table,
    Id,
    RsvpId,
    Token,
    ExpiresAt,
}
