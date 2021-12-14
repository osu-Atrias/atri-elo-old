use axum::{http::StatusCode, response::Html};
use color_eyre::eyre::{eyre, Result};
use cookie::{Cookie, Key};

use maud::{html, DOCTYPE};

use tower_cookies::Cookies;

use crate::{general::User, pages::header};

use super::handle_error;

pub async fn root(cookies: Cookies) -> Result<Html<String>, StatusCode> {
    let user = match cookies.get("id") {
        Some(inner) => {
            let parsed_id = inner.value().parse().map_err(handle_error)?;
            let user = User::get(parsed_id).map_err(handle_error)?;
            match user {
                Some(user) => {
                    let trusted_id: u64 = cookies
                        .signed(&Key::from(user.cookie_master_key()))
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
                        removal_cookie.set_path("/");
                        cookies.remove(removal_cookie);
                        let mut removal_cookie = Cookie::named("trusted_id");
                        removal_cookie.set_path("/");
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
            (DOCTYPE)

            head {
                (header("atri-elo"))
            }

            body {
                nav .navbar.is-light role="navigation" aria-label="main navigation" {
                    .navbar-brand {
                        a .navbar-item href="/" {
                            img src="/favicon.ico";
                            "ATRI-ELO"
                        }
                    }

                    .navbar-end {
                        .navbar-item {
                            .buttons {
                                @if let Some(user) = &user {
                                    a .button.is-white href="/" {
                                        i .fas.fa-user {}
                                        (user.username())
                                    }

                                    a .button.is-white href="/oauth/logout" {
                                        i .fas.fa-sign-out-alt {}
                                        "Logout"
                                    }
                                } @else {
                                    a .button.is-primary href="/oauth/verify" {
                                        i .fas.fa-sign-in-alt {}
                                        strong {
                                            "Login"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                section .hero.is-primary {
                    .hero-body {
                        p .title {
                            "ATRI-ELO"
                        }
                        p .subtitle {
                            "A novel approach to rank osu! players."
                        }
                    }
                }
                section .section {
                    .container {
                        p .title {
                            "Welcome!"
                        }
                        p {
                            i .fas.fa-wrench style="color:orange" {}
                            " We are under construction now!"
                        }
                    }
                }
            }
        }
        .into_string(),
    ))
}
