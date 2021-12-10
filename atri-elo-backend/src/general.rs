use std::collections::HashMap;

use color_eyre::eyre::{eyre, Result};
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, url::Url, AccessToken, AuthUrl,
    AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, RedirectUrl,
    RefreshToken, Scope, TokenResponse, TokenUrl,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sled::Db;
use time::OffsetDateTime;
use tokio::{task::JoinHandle, time::timeout};
use tracing::info;

use crate::{api::OAUTH_QUEUE, config, util::serialize};

pub static DATABASE: Lazy<Db> =
    Lazy::new(|| sled::open(config::database::NAME()).expect("couldn't open database"));

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PlayerHistory {
    contest_id: u64,
    perf: f64,
    rating: f64,
    contest_rank: u64,
    rating_rank: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    id: u64,
    username: String,

    access_token: AccessToken,
    expires_in: OffsetDateTime,
    refresh_token: RefreshToken,

    rating: f64,
    rank: u64,
    history: HashMap<u64, PlayerHistory>,
}

impl User {
    pub async fn verify() -> Result<(Url, JoinHandle<Result<u64>>)> {
        let client = BasicClient::new(
            ClientId::new(config::oauth::CLIENT_ID()),
            Some(ClientSecret::new(config::oauth::CLIENT_SECRET())),
            AuthUrl::new(config::oauth::AUTH_URL())?,
            Some(TokenUrl::new(config::oauth::TOKEN_URL())?),
        )
        .set_redirect_uri(RedirectUrl::new(config::oauth::REDIRECT_URL())?);

        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let (auth_url, csrf_token) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("identify".to_string()))
            .set_pkce_challenge(pkce_challenge)
            .url();

        Ok((
            auth_url,
            tokio::spawn(async move {
                let code = timeout(
                    tokio::time::Duration::from_secs(config::oauth::TIMEOUT()),
                    tokio::spawn(async move {
                        let mut receiver = OAUTH_QUEUE.subscribe();

                        Result::<_>::Ok(loop {
                            let param = receiver.recv().await?;
                            if param.state.secret() == csrf_token.secret() {
                                break param.code;
                            }
                        })
                    }),
                )
                .await???;

                let token_result = client
                    .exchange_code(AuthorizationCode::new(code))
                    .set_pkce_verifier(pkce_verifier)
                    .request_async(async_http_client)
                    .await?;

                let client = reqwest::Client::new();

                let res = client
                    .get(config::OSU_USER_API_ENDPOINT())
                    .bearer_auth(token_result.access_token().secret())
                    .send()
                    .await?
                    .json::<Value>()
                    .await?;

                let id = res
                    .get("id")
                    .ok_or_else(|| eyre!("id not presented in response"))?
                    .as_u64()
                    .ok_or_else(|| eyre!("id not representable by u64"))?;

                let user = User {
                    id,
                    username: res
                        .get("username")
                        .ok_or_else(|| eyre!("username not presented in response"))?
                        .as_str()
                        .ok_or_else(|| eyre!("username not representable by &str"))?
                        .to_string(),
                    access_token: token_result.access_token().clone(),
                    expires_in: OffsetDateTime::now_utc()
                        + token_result
                            .expires_in()
                            .ok_or_else(|| eyre!("token expire time not presented in response"))?
                            / config::oauth::EXPIRE_TIME_FACTOR(),
                    refresh_token: token_result
                        .refresh_token()
                        .ok_or_else(|| eyre!("refresh token not presented in response"))?
                        .clone(),
                    rating: config::elo::MU_INIT(),
                    rank: 0,
                    history: HashMap::new(),
                };

                info!("user {}({}) authorized", user.username, user.id);

                DATABASE
                    .open_tree("users")?
                    .insert(&id.to_be_bytes(), serialize(&user)?)?;

                Ok(id)
            }),
        ))
    }
}
