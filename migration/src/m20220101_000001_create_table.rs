use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager.create_table(
            Table::create()
                .table(Users::Table)
                .if_not_exists()
                .col(ColumnDef::new(Users::Id).uuid().not_null().primary_key())
                .col(ColumnDef::new(Users::DiscordId).string().not_null().unique_key())
                .col(ColumnDef::new(Users::MinecraftUuid).string().unique_key())
                .to_owned()
        ).await?;

        manager.create_table(
            Table::create()
                .table(OauthTokens::Table)
                .if_not_exists()
                .col(ColumnDef::new(OauthTokens::Id).uuid().not_null().primary_key())
                .col(ColumnDef::new(OauthTokens::UserId).uuid().not_null())
                .col(ColumnDef::new(OauthTokens::AccessToken).string().not_null())
                .col(ColumnDef::new(OauthTokens::RefreshToken).blob().not_null())
                .col(ColumnDef::new(OauthTokens::ExpiresAt).timestamp().not_null())
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_oauth_tokens_user_id")
                        .from(OauthTokens::Table, OauthTokens::UserId)
                        .to(Users::Table, Users::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                )
                .to_owned()
        ).await?;

        manager.create_table(
            Table::create()
                .table(OauthStates::Table)
                .if_not_exists()
                .col(ColumnDef::new(OauthStates::StateId).uuid().not_null().primary_key())
                .col(ColumnDef::new(OauthStates::MinecraftUuid).string().not_null())
                .col(ColumnDef::new(OauthStates::ExpiresAt).timestamp().not_null())
                .to_owned()
        ).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(OauthStates::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(OauthTokens::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(Users::Table).to_owned()).await?;
        Ok(())
    }
}

#[derive(Iden)]
enum Users { Table, Id, DiscordId, MinecraftUuid }

#[derive(Iden)]
enum OauthTokens { Table, Id, UserId, AccessToken, RefreshToken, ExpiresAt }

#[derive(Iden)]
enum OauthStates { Table, StateId, MinecraftUuid, ExpiresAt }