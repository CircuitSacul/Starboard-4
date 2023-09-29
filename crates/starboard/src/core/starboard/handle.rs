use std::{collections::HashMap, sync::Arc};

use cached::Cached;
use twilight_model::id::{marker::MessageMarker, Id};

use database::{DbMessage, StarboardMessage, Vote};
use errors::StarboardResult;

use crate::{
    cache::MessageResult,
    client::bot::StarboardBot,
    core::{
        embedder::Embedder,
        emoji::{EmojiCommon, SimpleEmoji},
    },
    utils::{id_as_i64::GetI64, into_id::IntoId},
};

use super::{
    config::StarboardConfig,
    msg_status::{get_message_status, MessageStatus},
};

async fn refresh_exclusive_group(
    refresh: RefreshMessage,
    configs: Vec<Arc<StarboardConfig>>,
    force: bool,
) -> StarboardResult<()> {
    let mut refreshers = Vec::new();
    for config in configs {
        let mut ref_sb = RefreshStarboard::new(refresh.clone(), config);
        let sort_key = (
            ref_sb.config.resolved.exclusive_group_priority,
            ref_sb.has_message_on_starboard().await?,
        );

        refreshers.push((sort_key, ref_sb));
    }

    refreshers.sort_by(|left, right| right.0.cmp(&left.0));

    let mut message_exists = false;
    for (_, mut ref_sb) in refreshers {
        let ret = ref_sb.refresh(force, message_exists).await;

        message_exists = match ret {
            Err(why) => {
                refresh.bot.handle_error(&why).await;
                continue;
            }
            Ok(has_message) => has_message || message_exists,
        };
    }

    Ok(())
}

#[derive(Clone)]
pub struct RefreshMessage {
    bot: Arc<StarboardBot>,
    /// The id of the inputted message. May or may not be the original.
    message_id: Id<MessageMarker>,
    sql_message: Option<Arc<DbMessage>>,
    orig_message: Option<MessageResult>,
    configs: Option<Arc<Vec<Arc<StarboardConfig>>>>,
    is_premium: bool,
}

impl RefreshMessage {
    pub fn new(
        bot: Arc<StarboardBot>,
        message_id: Id<MessageMarker>,
        is_premium: bool,
    ) -> RefreshMessage {
        RefreshMessage {
            bot,
            message_id,
            configs: None,
            sql_message: None,
            orig_message: None,
            is_premium,
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
        let mut lone = Vec::new();
        let mut grouped = HashMap::new();

        for c in configs.iter() {
            if !c.resolved.enabled || c.starboard.premium_locked {
                continue;
            }

            if let Some(group_id) = c.resolved.exclusive_group {
                grouped
                    .entry(group_id)
                    .or_insert_with(Vec::new)
                    .push(Arc::clone(c));
            } else {
                lone.push(Arc::clone(c));
            }
        }

        let mut tasks = Vec::new();

        for group in grouped.into_values() {
            tasks.push(tokio::spawn(refresh_exclusive_group(
                self.to_owned(),
                group,
                force,
            )));
        }
        for config in lone {
            let mut refresh = RefreshStarboard::new(self.to_owned(), config.to_owned());
            tasks.push(tokio::spawn(async move {
                refresh.refresh(force, false).await.map(|_| ())
            }));
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
                DbMessage::get_original(&self.bot.db, self.message_id.get_i64()).await?;
            self.set_sql_message(sql_message.unwrap());
        }

        Ok(self.sql_message.as_ref().unwrap().clone())
    }

    pub fn set_orig_message(&mut self, message: MessageResult) {
        self.orig_message.replace(message);
    }

    async fn get_orig_message(&mut self) -> StarboardResult<MessageResult> {
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

    pub async fn has_message_on_starboard(&mut self) -> StarboardResult<bool> {
        Ok(self.get_starboard_message().await?.is_some())
    }

    pub async fn refresh(
        &mut self,
        force: bool,
        violates_exclusive_group: bool,
    ) -> StarboardResult<bool> {
        // I use a loop because recursion inside async functions requires another crate :(
        let mut tries = 0;
        loop {
            if tries == 2 {
                return Ok(false);
            }
            tries += 1;
            let (retry, exists) = self.refresh_one(force, violates_exclusive_group).await?;
            match retry {
                true => continue,
                false => return Ok(exists),
            }
        }
    }

    async fn refresh_one(
        &mut self,
        force: bool,
        violates_exclusive_group: bool,
    ) -> StarboardResult<(bool, bool)> {
        let orig = self.refresh.get_sql_message().await?;
        let points = Vote::count(
            &self.refresh.bot.db,
            orig.message_id,
            self.config.starboard.id,
        )
        .await?;

        let orig_message = self.refresh.get_orig_message().await?;
        let sql_message = self.refresh.get_sql_message().await?;
        let ref_msg = if let MessageResult::Ok(msg) = &orig_message {
            if let Some(id) = msg.referenced_message {
                self.refresh
                    .bot
                    .cache
                    .fog_message(&self.refresh.bot, sql_message.channel_id.into_id(), id)
                    .await?
                    .into_option()
            } else {
                None
            }
        } else {
            None
        };

        let sb_msg = self.get_starboard_message().await?;
        let embedder = Embedder {
            bot: self.refresh.bot.clone(),
            points,
            config: self.config.clone(),
            orig_message,
            referenced_message: ref_msg,
            orig_sql_message: sql_message,
            is_premium: self.refresh.is_premium,
        };

        let action = get_message_status(
            &self.refresh.bot,
            &self.config,
            &orig,
            &embedder.orig_message,
            points,
            violates_exclusive_group,
            self.refresh.is_premium,
        )
        .await?;

        if let Some(sb_msg) = sb_msg {
            if !force
                && points == sb_msg.last_known_point_count as i32
                && !matches!(action, MessageStatus::Remove)
            {
                return Ok((false, true));
            }
            StarboardMessage::set_last_point_count(
                &self.refresh.bot.db,
                sb_msg.starboard_message_id,
                points as i16,
            )
            .await?;

            let (retry, deleted) = match action {
                MessageStatus::Remove => {
                    let sb_message_id = sb_msg.starboard_message_id.into_id();
                    self.refresh
                        .bot
                        .cache
                        .auto_deleted_posts
                        .write()
                        .await
                        .cache_set(sb_message_id, ());
                    let deleted = embedder.delete(&self.refresh.bot, sb_message_id).await?;
                    (false, deleted)
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
                        (deleted, deleted)
                    }
                }
            };

            if deleted {
                StarboardMessage::delete(&self.refresh.bot.db, sb_msg.starboard_message_id).await?;
            }

            Ok((retry, !deleted))
        } else {
            if !matches!(action, MessageStatus::Send(_)) {
                return Ok((false, false));
            }

            let msg = embedder.send(&self.refresh.bot).await;
            let msg = match msg {
                Ok(msg) => msg,
                Err(why) => {
                    if why.http_status() == Some(403) {
                        return Ok((false, false));
                    } else {
                        return Err(why);
                    }
                }
            };

            StarboardMessage::create(
                &self.refresh.bot.db,
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

            Ok((false, true))
        }
    }

    async fn get_starboard_message(&mut self) -> sqlx::Result<Option<StarboardMessage>> {
        let orig = self.refresh.get_sql_message().await?;
        StarboardMessage::get_by_starboard(
            &self.refresh.bot.db,
            orig.message_id,
            self.config.starboard.id,
        )
        .await
    }
}
