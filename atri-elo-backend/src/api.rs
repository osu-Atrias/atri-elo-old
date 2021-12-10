use axum::{extract::Query, routing::get, Router, http::StatusCode};
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
    Router::new().route("/oauth/callback", get(oauth_callback))
    .route("/oauth/verify", get(oauth_verify))
}

#[derive(Deserialize, Clone, Debug)]
pub struct OauthCallbackParam {
    pub code: String,
    pub state: CsrfToken,
}

async fn oauth_callback(Query(param): Query<OauthCallbackParam>) -> StatusCode {
    match OAUTH_QUEUE.send(param) {
        Ok(_) => StatusCode::OK,
        Err(err) => {
            error!("no OAuth queue receiver: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        },
    }
}

async fn oauth_verify() -> (StatusCode, String) {
    match User::verify().await {
        Ok((url, _)) => {
            (StatusCode::OK, url.to_string())
        },
        Err(err) => {
            error!("error when verifying: {}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, "".to_string())
        },
    }
}
