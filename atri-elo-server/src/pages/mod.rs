use std::fmt::Display;

use axum::{http::StatusCode, routing::get, Router};
use color_eyre::Report;

use maud::{html, Markup};
use tracing::error;

use crate::config;

use self::{
    oauth::{oauth_callback, oauth_logout, oauth_verify},
    root::root,
};

mod oauth;

mod root;

pub fn router() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/favicon.ico", get(favicon))
        .route("/oauth/callback", get(oauth_callback))
        .route("/oauth/verify", get(oauth_verify))
        .route("/oauth/logout", get(oauth_logout))
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
        meta name="viewport" content="width=device-width, initial-scale=1";
        title { (page_title) }
    }
}

async fn favicon() -> &'static [u8] {
    include_bytes!("../../favicon.ico")
}
