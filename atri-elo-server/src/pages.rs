use std::fmt::Display;

use axum::{
    extract::Query,
    http::StatusCode,
    response::{Html, Redirect},
    routing::get,
    Router,
};
use color_eyre::{
    eyre::{eyre, Result},
    Report,
};
use cookie::{Cookie, Key, SameSite};
use dashmap::DashMap;
use maud::{html, Markup, DOCTYPE};
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthUrl, AuthorizationCode, ClientId,
    ClientSecret, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope,
    TokenResponse, TokenUrl,
};
use once_cell::sync::Lazy;
use serde::Deserialize;
use serde_json::Value;
use time::OffsetDateTime;
use tower_cookies::Cookies;
use tracing::{error, info};

use crate::{
    config,
    general::{User, DATABASE},
    util::serialize,
};

pub static OAUTH_WAITING_QUEUE: Lazy<DashMap<String, PkceCodeVerifier>> =
    Lazy::new(|| DashMap::new());

pub fn router() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/oauth/callback", get(oauth_callback))
        .route("/oauth/verify", get(oauth_verify))
}

fn handle_error(err: impl Into<Report> + Display) -> StatusCode {
    error!("error when handling req: {}", err);
    StatusCode::INTERNAL_SERVER_ERROR
}

fn header(page_title: &str) -> Markup {
    html! {
        link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bulma@0.9.3/css/bulma.min.css";
        script src={"https://kit.fontawesome.com/" (config::frontend::FONTAWESOME_KIT_CODE()) ".js"} crossorigin="anonymous" {}
        meta charset="utf-8";
        title { (page_title) }
    }
}

async fn root(cookies: Cookies) -> Result<Html<String>, StatusCode> {
    let user = match cookies.get("id") {
        Some(inner) => {
            let parsed_id = inner.value().parse().map_err(handle_error)?;
            let user = User::get(parsed_id).map_err(handle_error)?;
            match user {
                Some(user) => {
                    let trusted_id: u64 = cookies
                        .signed(&Key::from(&user.cookie_master_key()))
                        .get("trusted_id")
                        .ok_or_else(|| eyre!("cookie verification failed"))
                        .map_err(handle_error)?
                        .value()
                        .parse()
                        .map_err(handle_error)?;

                    if trusted_id == parsed_id {
                        Some(user)
                    } else {
                        let mut removal_cookie = Cookie::named("id");
                        removal_cookie.make_removal();
                        cookies.remove(removal_cookie);
                        None
                    }
                }
                None => None,
            }
        }
        None => None,
    };

    Ok(Html(
        html! {
            head {
                (DOCTYPE)
                (header("atri-elo"))
            }

            body {
                section .section {
                    .box {
                        @if let Some(user) = user {
                            p .title {
                                "Welcome! "
                                i .fas.fa-user {}
                                " "
                                (user.username())
                            }
                        } @else {
                            p .title {
                                "Welcome! "
                            }
                            p {
                                "You can visit "
                                a href="/oauth/verify" { "here" }
                                " to login."
                            }
                        }
                    }
                    .box {
                        p .title {
                            i .fas.fa-wrench style="color:orange" {}
                            " We are under construction now! Hold your breath!"
                        }
                    }
                }
            }
        }
        .into_string(),
    ))
}

async fn oauth_verify() -> Result<Redirect, StatusCode> {
    let client = BasicClient::new(
        ClientId::new(config::oauth::CLIENT_ID()),
        Some(ClientSecret::new(config::oauth::CLIENT_SECRET())),
        AuthUrl::new(config::oauth::AUTH_URL()).map_err(handle_error)?,
        Some(TokenUrl::new(config::oauth::TOKEN_URL()).map_err(handle_error)?),
    )
    .set_redirect_uri(RedirectUrl::new(config::oauth::REDIRECT_URL()).map_err(handle_error)?);

    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("identify".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    OAUTH_WAITING_QUEUE.insert(csrf_token.secret().clone(), pkce_verifier);

    Ok(Redirect::to(
        auth_url.to_string().try_into().map_err(handle_error)?,
    ))
}

#[derive(Deserialize, Clone, Debug)]
pub struct OauthCallbackParam {
    pub code: String,
    pub state: CsrfToken,
}

async fn oauth_callback(
    cookies: Cookies,
    Query(param): Query<OauthCallbackParam>,
) -> Result<Html<String>, StatusCode> {
    let pkce_verifier = OAUTH_WAITING_QUEUE
        .remove(param.state.secret())
        .ok_or_else(|| eyre!("csrf token not matched"))
        .map_err(handle_error)?
        .1;

    let client = BasicClient::new(
        ClientId::new(config::oauth::CLIENT_ID()),
        Some(ClientSecret::new(config::oauth::CLIENT_SECRET())),
        AuthUrl::new(config::oauth::AUTH_URL()).map_err(handle_error)?,
        Some(TokenUrl::new(config::oauth::TOKEN_URL()).map_err(handle_error)?),
    )
    .set_redirect_uri(RedirectUrl::new(config::oauth::REDIRECT_URL()).map_err(handle_error)?);

    let token_result = client
        .exchange_code(AuthorizationCode::new(param.code))
        .set_pkce_verifier(pkce_verifier)
        .request_async(async_http_client)
        .await
        .map_err(handle_error)?;

    let res = reqwest::Client::new()
        .get(config::OSU_USER_API_ENDPOINT())
        .bearer_auth(token_result.access_token().secret())
        .send()
        .await
        .map_err(handle_error)?
        .json::<Value>()
        .await
        .map_err(handle_error)?;

    let id = res
        .get("id")
        .ok_or_else(|| eyre!("id not presented in response"))
        .map_err(handle_error)?
        .as_u64()
        .ok_or_else(|| eyre!("id not representable by u64"))
        .map_err(handle_error)?;

    let cookie_master_key = Key::generate();

    if DATABASE
        .open_tree("users")
        .map_err(handle_error)?
        .contains_key(&id.to_be_bytes())
        .map_err(handle_error)?
    {
        let mut user = User::get(id).map_err(handle_error)?.unwrap();
        user.set_cookie_master_key(cookie_master_key.master().to_vec());
        user.save().map_err(handle_error)?;

        info!("user {}({}) reauthorized", user.username(), user.id());
    } else {
        let user = User::new(
            id,
            res.get("username")
                .ok_or_else(|| eyre!("username not presented in response"))
                .map_err(handle_error)?
                .as_str()
                .ok_or_else(|| eyre!("username not representable by &str"))
                .map_err(handle_error)?
                .to_string(),
            token_result.access_token().clone(),
            OffsetDateTime::now_utc()
                + token_result
                    .expires_in()
                    .ok_or_else(|| eyre!("token expire time not presented in response"))
                    .map_err(handle_error)?
                    / config::oauth::EXPIRE_TIME_FACTOR(),
            token_result
                .refresh_token()
                .ok_or_else(|| eyre!("refresh token not presented in response"))
                .map_err(handle_error)?
                .clone(),
            cookie_master_key.master().to_vec(),
        );

        info!("user {}({}) authorized", user.username(), user.id());

        DATABASE
            .open_tree("users")
            .map_err(handle_error)?
            .insert(&id.to_be_bytes(), serialize(&user).map_err(handle_error)?)
            .map_err(handle_error)?;
    }

    cookies.add(
        Cookie::build("id", id.to_string())
            .path("/")
            .permanent()
            .same_site(SameSite::Strict)
            .finish(),
    );

    cookies.signed(&cookie_master_key).add(
        Cookie::build("trusted_id", id.to_string())
            .path("/")
            .permanent()
            .same_site(SameSite::Strict)
            .finish(),
    );

    Ok(Html(html! {
            head {
                (DOCTYPE)
                meta http-equiv="refresh" content="5; url='/'";
                (header("OAuth Verification"))
            }

            body {
                section .section {
                    .box {
                        p .title {
                            i .fas.fa-check style="color:green" {}
                            " OAuth Verification Succeeded"
                        }
                        p {
                            "Your account is authorized and you can safely leave this page now."
                        }
                    }
                }
            }
        }.into_string()))
}
