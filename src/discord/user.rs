use crate::discord::client::DiscordClient;
use reqwest::Error;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct DiscordUser {
    pub id: String,
    pub username: String
}

impl DiscordUser {
    pub async fn fetch(api: &DiscordClient, token: &str) -> Result<Self, Error> {
        api.get("users/@me", token).await
    }
}