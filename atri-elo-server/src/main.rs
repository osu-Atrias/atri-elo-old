use itconfig::config;
use pages::router;
use tower::ServiceBuilder;
use tower_cookies::CookieManagerLayer;
use tower_http::trace::TraceLayer;
use tracing::Level;

mod general;

mod util;

mod pages;

config! {
    database {
        NAME => "db",
    },

    elo {
        RHO:f64 => 1.0,
        BETA:f64 => 200.0,
        GAMMA:f64 => 80.0,
        MU_INIT:f64 => 1500.0,
        SIGMA_INIT:f64 => 350.0,
    },

    oauth {
        CLIENT_ID: String,
        CLIENT_SECRET: String,
        AUTH_URL: String,
        TOKEN_URL: String,
        REDIRECT_URL: String,
        TIMEOUT: u64 => 30,
        EXPIRE_TIME_FACTOR: u32 => 2,
    },


    admin {
        KEY: String
    },

    frontend {
        FONTAWESOME_KIT_CODE: String,
    },

    OSU_USER_API_ENDPOINT => "https://osu.ppy.sh/api/v2/me",
}

#[tokio::main]
async fn main() {
    color_eyre::install().unwrap();

    config::init();

    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    axum::Server::bind(&"0.0.0.0:10818".parse().unwrap())
        .serve(
            router()
                .layer(
                    ServiceBuilder::new()
                        .layer(CookieManagerLayer::new()),
                )
                .into_make_service(),
        )
        .await
        .unwrap();
}
