use axum::{http::StatusCode, response::Html};
use color_eyre::eyre::Result;

use maud::{html, DOCTYPE};

use tower_cookies::Cookies;

use crate::pages::header;

use super::{handle_error, oauth::get_user_by_cookie};

pub async fn root(cookies: Cookies) -> Result<Html<String>, StatusCode> {
    let user = get_user_by_cookie(&cookies).map_err(handle_error)?;

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
                                    a .button.is-white href="/user" {
                                        i .fas.fa-user {}
                                        (&user.username)
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
