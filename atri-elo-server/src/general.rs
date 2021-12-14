use std::collections::HashMap;

use color_eyre::eyre::{eyre, Result};
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AccessToken, AuthUrl, ClientId, ClientSecret,
    RefreshToken, TokenResponse, TokenUrl,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sled::Db;
use time::OffsetDateTime;

use crate::{
    config,
    util::{deserialize, serialize},
};

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
    cookie_master_key: Vec<u8>,

    rating: f64,
    rank: u64,
    history: HashMap<u64, PlayerHistory>,
}

impl User {
    pub fn new(
        id: u64,
        username: String,
        access_token: AccessToken,
        expires_in: OffsetDateTime,
        refresh_token: RefreshToken,
        cookie_master_key: Vec<u8>,
    ) -> Self {
        Self {
            id,
            username,
            access_token,
            expires_in,
            refresh_token,
            cookie_master_key,
            rating: config::elo::MU_INIT(),
            rank: 0,
            history: HashMap::new(),
        }
    }

    pub fn get(id: u64) -> Result<Option<User>> {
        Ok(match DATABASE.open_tree("users")?.get(&id.to_be_bytes())? {
            Some(buf) => Some(deserialize(&buf)?),
            None => None,
        })
    }

    pub fn save(&self) -> Result<()> {
        let buf = serialize(&self)?;

        DATABASE
            .open_tree("users")?
            .insert(&self.id.to_be_bytes(), buf)?;

        Ok(())
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

    /// Get a reference to the user's cookie master key.
    pub fn cookie_master_key(&self) -> &[u8] {
        self.cookie_master_key.as_ref()
    }

    /// Set the user's cookie master key.
    pub fn set_cookie_master_key(&mut self, cookie_master_key: Vec<u8>) {
        self.cookie_master_key = cookie_master_key;
    }

    /// Set the user's access token.
    pub fn set_access_token(&mut self, access_token: AccessToken) {
        self.access_token = access_token;
    }

    /// Set the user's expires in.
    pub fn set_expires_in(&mut self, expires_in: OffsetDateTime) {
        self.expires_in = expires_in;
    }

    /// Set the user's refresh token.
    pub fn set_refresh_token(&mut self, refresh_token: RefreshToken) {
        self.refresh_token = refresh_token;
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

    scores: HashMap<u64, u64>,
    detail: HashMap<u64, ContestDetail>,
}

impl Contest {}
