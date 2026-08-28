use reqwest::{Client, Error};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct DiscordTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64
}

pub async fn exchange_token(
    http: &Client,
    client_id: &str,
    client_secret: &str,
    code: &str,
    redirect_uri: &str
) -> Result<DiscordTokenResponse, Error> {
    let token_res = http
        .post("https://discord.com/api/v10/oauth2/token")
        .form(&[
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", redirect_uri)
        ])
        .send()
        .await?
        .json::<DiscordTokenResponse>()
        .await?;

    Ok(token_res)
}

pub async fn refresh_token(
    http: &Client,
    client_id: &str,
    client_secret: &str,
    refresh_token: &str
) -> Result<DiscordTokenResponse, Error> {
    let token_res = http
        .post("https://discord.com/api/v10/oauth2/token")
        .form(&[
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token)
        ])
        .send()
        .await?
        .json::<DiscordTokenResponse>()
        .await?;

    Ok(token_res)
}

pub async fn revoke_token(
    http: &Client,
    client_id: &str,
    client_secret: &str,
    token: &str
) -> Result<(), Error> {
    http.post("https://discord.com/api/v10/oauth2/token/revoke")
        .form(&[
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("token", token),
            ("token_type_hint", "access_token")
        ])
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}