use std::collections::{HashMap, HashSet};

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
    pub contest_id: u64,
    pub perf: f64,
    pub rating: f64,
    pub contest_rank: u64,
    pub rating_rank: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub access_token: AccessToken,
    pub expires_in: OffsetDateTime,
    pub refresh_token: RefreshToken,
    pub avatar_url: String,
    pub cookie_master_key: Vec<u8>,
    pub rating: f64,
    pub rank: u64,
    pub history: HashMap<u64, PlayerHistory>,
}

impl User {
    pub fn new(
        id: u64,
        username: String,
        access_token: AccessToken,
        expires_in: OffsetDateTime,
        refresh_token: RefreshToken,
        cookie_master_key: Vec<u8>,
        avatar_url: String,
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
            avatar_url,
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

        self.save()?;

        Ok(())
    }

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

            self.save()?;
        }
        Ok(&self.access_token)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContestGroup {
    pub id: u64,
    pub name: String,
    pub contests: HashSet<u64>,
}

impl ContestGroup {
    pub fn new(id: u64, name: String, contests: HashSet<u64>) -> Self {
        Self { id, name, contests }
    }

    pub fn get(id: u64) -> Result<Option<ContestGroup>> {
        Ok(
            match DATABASE
                .open_tree("contest_groups")?
                .get(&id.to_be_bytes())?
            {
                Some(buf) => Some(deserialize(&buf)?),
                None => None,
            },
        )
    }

    pub fn save(&self) -> Result<()> {
        let buf = serialize(&self)?;

        DATABASE
            .open_tree("contest_groups")?
            .insert(&self.id.to_be_bytes(), buf)?;

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ContestDetail {
    pub uid: u64,
    pub perf: f64,
    pub rating: f64,
    pub contest_rank: f64,
    pub rating_rank: f64,
}

impl ContestDetail {
    pub fn new(uid: u64, perf: f64, rating: f64, contest_rank: f64, rating_rank: f64) -> Self {
        Self {
            uid,
            perf,
            rating,
            contest_rank,
            rating_rank,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contest {
    pub id: u64,
    pub name: String,
    pub group_id: u64,
    pub beatmap_id: u64,
    pub status: u64,
    pub open_time: OffsetDateTime,
    pub close_time: OffsetDateTime,
    pub rank_time: Option<OffsetDateTime>,
    pub scores: HashMap<u64, u64>,
    pub detail: HashMap<u64, ContestDetail>,
}

impl Contest {
    pub fn new(
        id: u64,
        name: String,
        group_id: u64,
        beatmap_id: u64,
        open_time: OffsetDateTime,
        close_time: OffsetDateTime,
    ) -> Self {
        Self {
            id,
            name,
            group_id,
            beatmap_id,
            status: 0,
            open_time,
            close_time,
            rank_time: None,
            scores: HashMap::new(),
            detail: HashMap::new(),
        }
    }

    pub fn get(id: u64) -> Result<Option<Contest>> {
        Ok(
            match DATABASE.open_tree("contests")?.get(&id.to_be_bytes())? {
                Some(buf) => Some(deserialize(&buf)?),
                None => None,
            },
        )
    }

    pub fn save(&self) -> Result<()> {
        let buf = serialize(&self)?;

        DATABASE
            .open_tree("contests")?
            .insert(&self.id.to_be_bytes(), buf)?;

        Ok(())
    }
}
