use sea_orm::{DatabaseConnection, DbErr, EntityTrait, Set};
use sea_orm::sea_query::OnConflict;

use crate::entity::verified_users::{Entity as VerifiedUser, ActiveModel, Column};

impl VerifiedUser {
    pub async fn is_player_verified(db: &DatabaseConnection, player_uuid: &str) -> Result<bool, DbErr> {
        let result = Self::find_by_id(player_uuid.to_string()).one(db).await?;
        Ok(result.is_some())
    }

    pub async fn register_player(db: &DatabaseConnection, mc_uuid: &str, discord_id: &str) -> Result<(), DbErr> {
        let user_entry = ActiveModel {
            uuid: Set(mc_uuid.to_string()),
            discord_id: Set(discord_id.to_string()),
        };

        Self::insert(user_entry)
            .on_conflict(
                OnConflict::column(Column::Uuid)
                    .update_column(Column::DiscordId)
                    .to_owned()
            )
            .exec(db)
            .await?;

        Ok(())
    }
}