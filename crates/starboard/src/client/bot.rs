use std::{fmt::Debug, sync::Arc, time::Duration};

use chrono::{DateTime, Utc};
use database::DbClient;
use futures::Future;
use tokio::sync::RwLock;
use twilight_gateway::{Config as GatewayConfig, Intents};
use twilight_http::client::{Client as HttpClient, InteractionClient};
use twilight_model::{
    http::attachment::Attachment,
    id::{marker::ChannelMarker, Id},
    oauth::PartialApplication,
};
use twilight_standby::Standby;

use errors::{StarboardError, StarboardResult};

use crate::{cache::Cache, client::config::Config, utils::into_id::IntoId};

use super::{cooldowns::Cooldowns, locks::Locks};

pub struct StarboardBot {
    pub http: HttpClient,
    pub reqwest: reqwest::Client,
    pub cache: Cache,
    pub application: RwLock<Option<PartialApplication>>,
    pub db: DbClient,
    pub standby: Standby,
    pub config: Config,
    pub gw_config: GatewayConfig,
    pub cooldowns: Cooldowns,
    pub locks: Locks,
    pub start: DateTime<Utc>,
}

impl Debug for StarboardBot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Starboard")
    }
}

impl StarboardBot {
    pub async fn new(config: Config) -> StarboardResult<Self> {
        // Setup gateway connection
        let intents = Intents::GUILDS
            | Intents::GUILD_EMOJIS_AND_STICKERS
            | Intents::GUILD_MEMBERS
            | Intents::GUILD_MESSAGES
            | Intents::DIRECT_MESSAGES
            | Intents::MESSAGE_CONTENT
            | Intents::GUILD_MESSAGE_REACTIONS;

        let gw_config = GatewayConfig::new(config.token.clone(), intents);

        // Setup HTTP connection
        let mut http = HttpClient::builder()
            .token(config.token.clone())
            .timeout(Duration::from_secs(30));
        if let Some(proxy) = &config.proxy {
            http = http.proxy(proxy.to_owned(), true);
        }
        let http = http.build();

        // Setup database connection
        let db = DbClient::new(&config.db_url).await?;

        // run migrations
        db.run_migrations().await.expect("to run migrations");

        // load autostar channels
        let asc: Vec<_> = sqlx::query!(
            "SELECT DISTINCT channel_id FROM autostar_channels WHERE premium_locked=false"
        )
        .fetch_all(&db.pool)
        .await?
        .into_iter()
        .map(|rec| Id::<ChannelMarker>::new(rec.channel_id as u64))
        .collect();

        let mut map = dashmap::DashSet::new();
        map.extend(asc);

        // Setup cache
        let cache = Cache::new(map);

        // Return the bot struct
        Ok(Self {
            http,
            cache,
            application: RwLock::new(None),
            db,
            standby: Standby::new(),
            config,
            gw_config,
            cooldowns: Cooldowns::new(),
            locks: Locks::new(),
            reqwest: reqwest::Client::new(),
            start: Utc::now(),
        })
    }

    pub async fn interaction_client(&self) -> InteractionClient {
        match &*self.application.read().await {
            Some(info) => self.http.interaction(info.id),
            None => panic!("interaction_client called before bot was ready."),
        }
    }

    pub async fn handle_error(&self, err: &StarboardError) {
        sentry::capture_error(err);

        let msg = format!("{err:#?}").trim().to_string();
        let msg = if msg.is_empty() {
            "Some Error".to_string()
        } else {
            msg
        };

        eprintln!("{msg}");

        let attachment = Attachment::from_bytes("erorr.rs".into(), msg.bytes().collect(), 1);
        let attachments = &[attachment];

        if let Some(chid) = self.config.error_channel {
            let ret = self
                .http
                .create_message(chid.into_id())
                .attachments(attachments);
            let ret = match ret {
                Ok(ret) => ret,
                Err(why) => return eprintln!("{why}"),
            };
            if let Err(why) = ret.await {
                eprintln!("{why}");
            }
        }
    }

    pub async fn catch_future_errors<T, E: Into<StarboardError>>(
        bot: Arc<StarboardBot>,
        future: impl Future<Output = Result<T, E>>,
    ) {
        if let Err(err) = future.await {
            bot.handle_error(&err.into()).await;
        }
    }
}
