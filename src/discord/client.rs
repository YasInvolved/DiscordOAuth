use reqwest::{Client, Error};
use serde::Deserialize;

#[derive(Clone)]
pub struct DiscordClient {
    http: Client
}

impl DiscordClient {
    pub fn new() -> Self {
        Self { http: Client::new() }
    }

    pub async fn get<T: for<'de> Deserialize<'de>>(
        &self, 
        endpoint: &str, 
        token: &str
    ) -> Result<T, Error> {
        let url = format!("https://discord.com/api/v10/{}", endpoint);

        let response = self.http.get(&url)
            .bearer_auth(token)
            .send()
            .await?;

        response.error_for_status()?.json::<T>().await
    }
}