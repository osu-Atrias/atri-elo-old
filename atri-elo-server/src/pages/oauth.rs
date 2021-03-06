use axum::{
    extract::Query,
    http::StatusCode,
    response::{Html, Redirect},
};
use color_eyre::eyre::{eyre, Result};
use cookie::{Cookie, Key, SameSite};
use dashmap::DashMap;
use maud::{html, DOCTYPE};
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
use tracing::info;

use crate::{config, general::User, pages::header};

use super::handle_error;

pub static OAUTH_WAITING_QUEUE: Lazy<DashMap<String, PkceCodeVerifier>> = Lazy::new(DashMap::new);

fn clear_cookies(cookies: &Cookies) {
    let mut removal_cookie = Cookie::named("id");
    removal_cookie.set_path("/");
    cookies.remove(removal_cookie);
    let mut removal_cookie = Cookie::named("trusted_id");
    removal_cookie.set_path("/");
    cookies.remove(removal_cookie);
}

pub fn get_user_by_cookie(cookies: &Cookies) -> Result<Option<User>> {
    Ok(match cookies.get("id") {
        Some(inner) => {
            let parsed_id = inner.value().parse()?;
            let user = User::get(parsed_id)?;
            match user {
                Some(user) => {
                    let trusted_id: u64 = cookies
                        .signed(&Key::from(&user.cookie_master_key))
                        .get("trusted_id")
                        .ok_or_else(|| eyre!("cookie verification failed"))?
                        .value()
                        .parse()?;

                    if trusted_id == parsed_id {
                        Some(user)
                    } else {
                        clear_cookies(cookies);
                        None
                    }
                }
                None => None,
            }
        }
        None => None,
    })
}

pub async fn oauth_verify() -> Result<Redirect, StatusCode> {
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

pub async fn oauth_callback(
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

    match User::get(id).map_err(handle_error)? {
        Some(mut user) => {
            user.access_token = token_result.access_token().clone();
            user.expires_in = OffsetDateTime::now_utc()
                + token_result
                    .expires_in()
                    .ok_or_else(|| eyre!("token expire time not presented in response"))
                    .map_err(handle_error)?
                    / config::oauth::EXPIRE_TIME_FACTOR();
            user.refresh_token = token_result
                .refresh_token()
                .ok_or_else(|| eyre!("refresh token not presented in response"))
                .map_err(handle_error)?
                .clone();
            user.cookie_master_key = cookie_master_key.master().to_vec();
            user.save().map_err(handle_error)?;
            info!("user {}({}) reauthorized", user.username, user.id);
        }
        None => {
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
                res.get("avatar_url")
                    .ok_or_else(|| eyre!("username not presented in response"))
                    .map_err(handle_error)?
                    .as_str()
                    .ok_or_else(|| eyre!("username not representable by &str"))
                    .map_err(handle_error)?
                    .to_string(),
            );

            user.save().map_err(handle_error)?;

            info!("user {}({}) authorized", user.username, user.id);
        }
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
            (DOCTYPE)

            head {
                meta http-equiv="refresh" content="3; url='/'";
                (header("OAuth Verification"))
            }

            body {
                section .section {
                    .notification.is-success.is-light {
                        p .title {
                            i .fas.fa-check {}
                            " OAuth Verification Succeeded"
                        }
                        p {
                            "Your account is authorized now and you will be redirected to root page in 3 seconds."
                        }
                    }
                }
            }
        }.into_string()))
}

pub async fn oauth_logout(cookies: Cookies) -> Html<String> {
    clear_cookies(&cookies);
    Html(
        html! {
            (DOCTYPE)

            head {
                meta http-equiv="refresh" content="3; url='/'";
                (header("OAuth Logout"))
            }

            body {
                section .section {
                    .notification.is-info.is-light {
                        p .title {
                            i .fas.fa-info-circle {}
                            " OAuth Logout Succeeded"
                        }
                        p {
                            "You will be redirected to root page in 3 seconds."
                        }
                    }
                }
            }
        }
        .into_string(),
    )
}
