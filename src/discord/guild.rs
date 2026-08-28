use serde::Deserialize;
use reqwest::Error;

use crate::discord::client::DiscordClient;

#[derive(Deserialize)]
pub struct DiscordGuild {
    pub id: String,
    pub name: String
}

impl DiscordGuild {
    pub async fn fetch_for_user(api: &DiscordClient, token: &str) -> Result<Vec<Self>, Error> {
        api.get::<Vec<Self>>("users/@me/guilds", token).await
    }

    pub fn is_member(guilds: &[Self], target_id: &str) -> bool {
        guilds.iter().any(|g| g.id == target_id)
    }
}
