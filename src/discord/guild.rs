use serde::Deserialize;
use reqwest::Error;

use crate::discord::{client::DiscordClient, user::DiscordUser};

#[derive(Deserialize)]
pub struct DiscordGuild {
    pub id: String,
    pub name: String
}

#[derive(Deserialize, Debug)]
pub struct GuildMember {
    pub user: Option<DiscordUser>,
    pub nick: Option<String>,
    pub avatar: Option<String>,
    pub roles: Vec<String>,
    pub joined_at: String,
    pub premium_since: Option<String>,

    #[serde(default)]
    pub deaf: bool,
    #[serde(default)]
    pub mute: bool,
    pub pending: Option<bool>,
    pub communication_disabled_until: Option<String>
}

impl DiscordGuild {
    pub async fn fetch_for_user(api: &DiscordClient, token: &str) -> Result<Vec<Self>, Error> {
        api.get::<Vec<Self>>("users/@me/guilds", token).await
    }

    pub async fn get_membership_of_id(api: &DiscordClient, token: &str, guild_id: &str) -> Result<GuildMember, Error> {
        let endpoint = format!("users/@me/guilds/{}/member", guild_id);
        api.get::<GuildMember>(&endpoint, token).await
    }

    pub async fn get_membership(&self, api: &DiscordClient, token: &str) -> Result<GuildMember, Error> {
        DiscordGuild::get_membership_of_id(api, token, &self.id).await
    }
}
