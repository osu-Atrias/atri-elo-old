use axum::{response::Html, extract::Path};
use maud::{html, DOCTYPE};
use reqwest::StatusCode;
use tower_cookies::Cookies;

use crate::{pages::header, general::User};

use super::{handle_error, oauth::get_user_by_cookie};

fn empty_user_page() -> Html<String> {
    Html(
        html! {
            (DOCTYPE)
            
            head {
                meta http-equiv="refresh" content="3; url='/'";
                (header("User Stat"))
            }

            body {
                .notification.is-info.is-light {
                    p .title {
                        i .fas.fa-info-circle {}
                        " Please Login"
                    }
                    p {
                        "You will be redirected to root page in 3 seconds."
                    }
                }
            }
        }
        .into_string(),
    )
}

pub async fn user(cookies: Cookies) -> Result<Html<String>, StatusCode> {
    let user = get_user_by_cookie(&cookies).map_err(handle_error)?;

    if user.is_none() {
        return Ok(empty_user_page());
    }

    let user = user.unwrap();

    Ok(Html(
        html! {
            (DOCTYPE)

            head {
                (header("User"))
            }

            body {
                section .section {
                    .box {
                        article .media {
                            figure .media-left {
                                p .image."is-128x128" {
                                    img src=(user.avatar_url);
                                }
                            }
                            .media-content {
                                p .title."is-1" {
                                    a href={"https://osu.ppy.sh/u/" (user.id)} {
                                        (user.username)
                                    }
                                }
                                p .subtitle."is-3" {
                                    "#" (user.rank)
                                }
                            }
                        }
                    }
                }
            }
        }
        .into_string(),
    ))
}

pub async fn user_with_id(Path(user_id): Path<u64>) -> Result<Html<String>, StatusCode> {
    let user = User::get(user_id).map_err(handle_error)?;

    if user.is_none() {
        return Ok(empty_user_page());
    }

    let user = user.unwrap();

    Ok(Html(
        html! {
            (DOCTYPE)

            head {
                (header("User"))
            }

            body {
                section .section {
                    .box {
                        article .media {
                            figure .media-left {
                                p .image."is-128x128" {
                                    img src=(user.avatar_url);
                                }
                            }
                            .media-content {
                                p .title."is-1" {
                                    a href={"https://osu.ppy.sh/u/" (user.id)} {
                                        (user.username)
                                    }
                                }
                                p .subtitle."is-3" {
                                    "#" (user.rank)
                                }
                            }
                        }
                    }
                }
            }
        }
        .into_string(),
    ))
}
