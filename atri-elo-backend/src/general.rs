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
use tokio::time::timeout;
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
    pub async fn verify() -> Result<Url> {
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

            let res = reqwest::Client::new()
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

            Result::<_>::Ok(id)
        });

        Ok(auth_url)
    }

    /// Get a reference to the user's id.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Get a reference to the user's username.
    pub fn username(&self) -> &str {
        self.username.as_ref()
    }

    pub async fn update_username(&mut self) -> Result<()> {
        let res = reqwest::Client::new()
            .get(config::OSU_USER_API_ENDPOINT())
            .bearer_auth(self.access_token().await?.secret())
            .send()
            .await?
            .json::<Value>()
            .await?;

        self.username = res
            .get("username")
            .ok_or_else(|| eyre!("username not presented in response"))?
            .as_str()
            .ok_or_else(|| eyre!("username not representable by &str"))?
            .to_string();

        Ok(())
    }

    /// Get a reference to the user's access token, will refresh if necessary.
    pub async fn access_token(&mut self) -> Result<&AccessToken> {
        if OffsetDateTime::now_utc() >= self.expires_in {
            let client = BasicClient::new(
                ClientId::new(config::oauth::CLIENT_ID()),
                Some(ClientSecret::new(config::oauth::CLIENT_SECRET())),
                AuthUrl::new(config::oauth::AUTH_URL())?,
                Some(TokenUrl::new(config::oauth::TOKEN_URL())?),
            );

            let token_result = client
                .exchange_refresh_token(&self.refresh_token)
                .request_async(async_http_client)
                .await?;

            if let Some(refresh_token) = token_result.refresh_token() {
                self.refresh_token = refresh_token.clone();
            }

            match token_result.expires_in() {
                Some(expires_in) => {
                    self.expires_in = OffsetDateTime::now_utc()
                        + expires_in / config::oauth::EXPIRE_TIME_FACTOR();
                }
                None => {
                    return Err(eyre!("expires info not presented in response"));
                }
            }

            self.access_token = token_result.access_token().clone();
        }
        Ok(&self.access_token)
    }

    /// Get a reference to the user's rating.
    pub fn rating(&self) -> f64 {
        self.rating
    }

    /// Get a reference to the user's rank.
    pub fn rank(&self) -> u64 {
        self.rank
    }

    /// Get a reference to the user's history.
    pub fn history(&self) -> &HashMap<u64, PlayerHistory> {
        &self.history
    }

    /// Get a mutable reference to the user's history.
    pub fn history_mut(&mut self) -> &mut HashMap<u64, PlayerHistory> {
        &mut self.history
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ContestDetail {
    uid: u64,
    perf: f64,
    rating: f64,
    contest_rank: f64,
    rating_rank: f64,
}

impl ContestDetail {
    /// Get a reference to the contest detail's uid.
    pub fn uid(&self) -> u64 {
        self.uid
    }

    /// Get a reference to the contest detail's perf.
    pub fn perf(&self) -> f64 {
        self.perf
    }

    /// Get a reference to the contest detail's rating.
    pub fn rating(&self) -> f64 {
        self.rating
    }

    /// Get a reference to the contest detail's contest rank.
    pub fn contest_rank(&self) -> f64 {
        self.contest_rank
    }

    /// Get a reference to the contest detail's rating rank.
    pub fn rating_rank(&self) -> f64 {
        self.rating_rank
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contest {
    id: u64,
    name: String,

    beatmap_id: u64,

    status: u64,
    open_time: OffsetDateTime,
    close_time: OffsetDateTime,
    rank_time: OffsetDateTime,

    detail: HashMap<u64, ContestDetail>,
}

impl Contest {}
