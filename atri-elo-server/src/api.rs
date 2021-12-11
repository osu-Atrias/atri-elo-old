use axum::{
    extract::Query,
    http::StatusCode,
    response::{Html, Redirect},
    routing::get,
    Router,
};
use color_eyre::eyre::Result;
use oauth2::CsrfToken;
use once_cell::sync::Lazy;
use serde::Deserialize;
use tokio::sync::broadcast::{channel, Sender};
use tracing::error;

use crate::general::User;

pub static OAUTH_QUEUE: Lazy<Sender<OauthCallbackParam>> = Lazy::new(|| {
    let (sender, _) = channel(16);
    sender
});

pub fn router() -> Router {
    Router::new()
        .route("/oauth/callback", get(oauth_callback))
        .route("/oauth/verify", get(oauth_verify))
}

#[derive(Deserialize, Clone, Debug)]
pub struct OauthCallbackParam {
    pub code: String,
    pub state: CsrfToken,
}

async fn oauth_callback(
    Query(param): Query<OauthCallbackParam>,
) -> Result<Html<&'static str>, (StatusCode, String)> {
    match OAUTH_QUEUE.send(param) {
        Ok(_) => Ok(Html(
            "<h1> Server is authorizing your account and you can safely close this page now.",
        )),
        Err(err) => {
            error!("no OAuth queue receiver: {}", err);
            Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
        }
    }
}

async fn oauth_verify() -> Result<Redirect, (StatusCode, String)> {
    let url = match match User::verify().await {
        Ok(it) => it,
        Err(err) => return Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string())),
    }
    .to_string()
    .try_into()
    {
        Ok(it) => it,
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "invalid url returned from osu! server".to_string(),
            ))
        }
    };
    Ok(Redirect::to(url))
}
