pub mod cache;
pub mod client;
pub mod core;
pub mod events;
pub mod interactions;
pub mod macros;
pub mod owner;
pub mod parsing;
pub mod utils;

use tokio::main;
use tracing_subscriber::{fmt, EnvFilter};

use crate::client::{bot::StarboardBot, config::Config, runner::run};

fn init_tracing() {
    tracing::subscriber::set_global_default(
        fmt::Subscriber::builder()
            .with_env_filter(EnvFilter::from_default_env())
            .finish(),
    )
    .expect("Unable to set global tracing subscriber");
}

#[main]
async fn main() {
    init_tracing();

    let config = Config::from_env();

    let _sentry_guard = config.sentry.as_ref().map(|url| {
        sentry::init((
            url.to_owned(),
            sentry::ClientOptions {
                release: sentry::release_name!(),
                ..Default::default()
            },
        ))
    });

    let bot = match StarboardBot::new(config).await {
        Ok(val) => val,
        Err(why) => {
            eprintln!("{:#?}", &why);
            sentry::capture_error(&why);
            return;
        }
    };

    run(bot).await;
}
