use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260827_215704_add_challenge_token"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        _manager.get_connection().execute_unprepared("DELETE FROM oauth_states").await?;
        _manager.alter_table(
            Table::alter()
                .table(OauthStates::Table)
                .add_column(
                    ColumnDef::new(Alias::new("challenge_token"))
                        .string()
                        .not_null()
                        .default("invalidated_legacy_state"),
                )
                .to_owned(),
        ).await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        _manager.alter_table(
            Table::alter()
                .table(OauthStates::Table)
                .drop_column(Alias::new("challenge_token"))
                .to_owned(),
        ).await
    }
}

#[derive(Iden)]
enum OauthStates { Table }
