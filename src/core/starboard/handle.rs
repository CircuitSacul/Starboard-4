use std::sync::Arc;

use twilight_model::id::{marker::MessageMarker, Id};

use crate::{
    cache::models::message::CachedMessage,
    client::bot::StarboardBot,
    core::{
        embedder::Embedder,
        emoji::{EmojiCommon, SimpleEmoji},
    },
    database::{Message as DbMessage, StarboardMessage, Vote},
    errors::StarboardResult,
    utils::{id_as_i64::GetI64, into_id::IntoId},
};

use super::{
    config::StarboardConfig,
    msg_status::{get_message_status, MessageStatus},
};

#[derive(Clone)]
pub struct RefreshMessage {
    bot: Arc<StarboardBot>,
    /// The id of the inputted message. May or may not be the original.
    message_id: Id<MessageMarker>,
    sql_message: Option<Arc<DbMessage>>,
    orig_message: Option<Option<Arc<CachedMessage>>>,
    configs: Option<Arc<Vec<Arc<StarboardConfig>>>>,
}

impl RefreshMessage {
    pub fn new(bot: Arc<StarboardBot>, message_id: Id<MessageMarker>) -> RefreshMessage {
        RefreshMessage {
            bot,
            message_id,
            configs: None,
            sql_message: None,
            orig_message: None,
        }
    }

    pub async fn refresh(&mut self, force: bool) -> StarboardResult<bool> {
        let orig = self.get_sql_message().await?;
        let clone = self.bot.clone();
        let guard = clone.locks.post_update_lock.lock(orig.message_id);
        if guard.is_none() {
            return Ok(false);
        }

        let configs = self.get_configs().await?;

        let mut tasks = Vec::new();
        for c in configs.iter() {
            if !c.resolved.enabled || c.starboard.premium_locked {
                continue;
            }

            let mut refresh = RefreshStarboard::new(self.to_owned(), c.to_owned());
            tasks.push(tokio::spawn(async move { refresh.refresh(force).await }));
        }

        for t in tasks {
            if let Ok(Err(why)) = t.await {
                self.bot.handle_error(&why).await;
            }
        }

        Ok(true)
    }

    // caching methods
    pub fn set_configs(&mut self, configs: Vec<Arc<StarboardConfig>>) {
        self.configs.replace(Arc::new(configs));
    }

    async fn get_configs(&mut self) -> StarboardResult<Arc<Vec<Arc<StarboardConfig>>>> {
        if self.configs.is_none() {
            let msg = self.get_sql_message().await?;
            let guild_id = msg.guild_id.into_id();
            let channel_id = msg.channel_id.into_id();

            let configs =
                StarboardConfig::list_for_channel(&self.bot, guild_id, channel_id).await?;
            self.set_configs(configs.into_iter().map(Arc::new).collect());
        }

        Ok(self.configs.as_ref().unwrap().clone())
    }

    pub fn set_sql_message(&mut self, message: DbMessage) {
        self.sql_message.replace(Arc::new(message));
    }

    async fn get_sql_message(&mut self) -> sqlx::Result<Arc<DbMessage>> {
        if self.sql_message.is_none() {
            let sql_message =
                DbMessage::get_original(&self.bot.pool, self.message_id.get_i64()).await?;
            self.set_sql_message(sql_message.unwrap());
        }

        Ok(self.sql_message.as_ref().unwrap().clone())
    }

    pub fn set_orig_message(&mut self, message: Option<Arc<CachedMessage>>) {
        self.orig_message.replace(message);
    }

    async fn get_orig_message(&mut self) -> StarboardResult<Option<Arc<CachedMessage>>> {
        if self.orig_message.is_none() {
            let sql_message = self.get_sql_message().await?;
            let orig_message = self
                .bot
                .cache
                .fog_message(
                    &self.bot,
                    sql_message.channel_id.into_id(),
                    sql_message.message_id.into_id(),
                )
                .await?;

            self.set_orig_message(orig_message);
        }

        Ok(self.orig_message.as_ref().unwrap().clone())
    }
}

struct RefreshStarboard {
    refresh: RefreshMessage,
    config: Arc<StarboardConfig>,
}

impl RefreshStarboard {
    pub fn new(refresh: RefreshMessage, config: Arc<StarboardConfig>) -> Self {
        Self { refresh, config }
    }

    pub async fn refresh(&mut self, force: bool) -> StarboardResult<()> {
        // I use a loop because recursion inside async functions requires another crate :(
        let mut tries = 0;
        loop {
            if tries == 2 {
                return Ok(());
            }
            tries += 1;
            let retry = self.refresh_one(force).await?;
            match retry {
                true => continue,
                false => return Ok(()),
            }
        }
    }

    async fn refresh_one(&mut self, force: bool) -> StarboardResult<bool> {
        let orig = self.refresh.get_sql_message().await?;
        let points = Vote::count(
            &self.refresh.bot.pool,
            orig.message_id,
            self.config.starboard.id,
        )
        .await?;

        let orig_message = self.refresh.get_orig_message().await?;
        let sql_message = self.refresh.get_sql_message().await?;
        let orig_message_author = self
            .refresh
            .bot
            .cache
            .fog_user(&self.refresh.bot, sql_message.author_id.into_id())
            .await?;
        let (ref_msg, ref_msg_author) = if let Some(msg) = &orig_message {
            if let Some(id) = msg.referenced_message {
                let ref_msg = self
                    .refresh
                    .bot
                    .cache
                    .fog_message(&self.refresh.bot, sql_message.channel_id.into_id(), id)
                    .await?;

                let ref_msg_author = match &ref_msg {
                    None => None,
                    Some(ref_msg) => Some(
                        self.refresh
                            .bot
                            .cache
                            .fog_user(&self.refresh.bot, ref_msg.author_id)
                            .await?,
                    ),
                };

                (ref_msg, ref_msg_author.flatten())
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        let sb_msg = self.get_starboard_message().await?;
        let embedder = Embedder {
            bot: &self.refresh.bot,
            points,
            config: &self.config,
            orig_message,
            orig_message_author,
            referenced_message: ref_msg,
            referenced_message_author: ref_msg_author,
            orig_sql_message: sql_message,
        };

        let action = get_message_status(
            &self.refresh.bot,
            &self.config,
            &orig,
            embedder.orig_message.is_none(),
            points,
        )
        .await?;

        if let Some(sb_msg) = sb_msg {
            if !force && points == sb_msg.last_known_point_count as i32 {
                return Ok(false);
            }
            StarboardMessage::set_last_point_count(
                &self.refresh.bot.pool,
                sb_msg.starboard_message_id,
                points as i16,
            )
            .await?;

            let (delete, retry) = match action {
                MessageStatus::Remove => {
                    embedder
                        .delete(&self.refresh.bot, sb_msg.starboard_message_id.into_id())
                        .await?;
                    (true, false)
                }
                MessageStatus::Send(full_update) | MessageStatus::Update(full_update) => {
                    if self
                        .refresh
                        .bot
                        .cooldowns
                        .message_edit
                        .trigger(&self.config.starboard.channel_id.into_id())
                        .is_some()
                    {
                        (false, false)
                    } else {
                        let deleted = embedder
                            .edit(
                                &self.refresh.bot,
                                sb_msg.starboard_message_id.into_id(),
                                !full_update,
                            )
                            .await?;
                        (deleted, true)
                    }
                }
            };

            if delete {
                StarboardMessage::delete(&self.refresh.bot.pool, sb_msg.starboard_message_id)
                    .await?;
            }

            Ok(retry)
        } else {
            if !matches!(action, MessageStatus::Send(_)) {
                return Ok(false);
            }

            let msg = embedder.send(&self.refresh.bot).await?;
            StarboardMessage::create(
                &self.refresh.bot.pool,
                orig.message_id,
                msg.id.get_i64(),
                self.config.starboard.id,
                points,
            )
            .await?;

            let mut to_react: Vec<SimpleEmoji> = Vec::new();
            if self.config.resolved.autoreact_upvote {
                to_react.extend(Vec::<SimpleEmoji>::from_stored(
                    self.config.resolved.upvote_emojis.clone(),
                ));
            }
            if self.config.resolved.autoreact_downvote {
                to_react.extend(Vec::<SimpleEmoji>::from_stored(
                    self.config.resolved.downvote_emojis.clone(),
                ));
            }

            for emoji in to_react {
                let _ = self
                    .refresh
                    .bot
                    .http
                    .create_reaction(msg.channel_id, msg.id, &emoji.reactable())
                    .await;
            }

            Ok(false)
        }
    }

    async fn get_starboard_message(&mut self) -> sqlx::Result<Option<StarboardMessage>> {
        let orig = self.refresh.get_sql_message().await?;
        StarboardMessage::get_by_starboard(
            &self.refresh.bot.pool,
            orig.message_id,
            self.config.starboard.id,
        )
        .await
    }
}
