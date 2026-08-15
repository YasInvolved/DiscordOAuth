use sea_orm_migration::{prelude::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(VerifiedUser::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(VerifiedUser::Uuid)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(VerifiedUser::DiscordId)
                            .string()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(VerifiedUser::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum VerifiedUser {
    Table,
    Uuid,
    DiscordId
}
