use std::sync::Arc;

use twilight_model::id::{
    marker::{ChannelMarker, GuildMarker, MessageMarker, UserMarker},
    Id,
};

use database::{DbUser, Filter};
use errors::StarboardResult;

use crate::{
    cache::{models::member::CachedMember, MessageResult},
    client::bot::StarboardBot,
    utils::{id_as_i64::GetI64, snowflake_age::SnowflakeAge},
};

use super::premium::is_premium::is_guild_premium;

fn has_all_roles(user_roles: &[i64], required: &[i64]) -> bool {
    for role in required {
        if !user_roles.contains(role) {
            return false;
        }
    }

    true
}

fn has_any_role(user_roles: &[i64], required: &[i64]) -> bool {
    for role in required {
        if user_roles.contains(role) {
            return true;
        }
    }

    false
}

pub struct FilterEvaluater<'a> {
    bot: &'a StarboardBot,
    filter_group_ids: Vec<i32>,

    // contextual info
    guild_id: Id<GuildMarker>,
    user_id: Id<UserMarker>,
    channel_id: Option<Id<ChannelMarker>>,
    message_id: Option<Id<MessageMarker>>,
    voter_id: Option<Id<UserMarker>>,

    // cached info
    user: Option<Option<Arc<CachedMember>>>,
    user_is_bot: Option<Option<bool>>,
    voter: Option<Option<Arc<CachedMember>>>,
    message: Option<MessageResult>,
    filters: Option<Arc<Vec<Vec<Filter>>>>,
}

impl<'a> FilterEvaluater<'a> {
    pub fn new(
        bot: &'a StarboardBot,
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
        voter_id: Option<Id<UserMarker>>,
        channel_id: Option<Id<ChannelMarker>>,
        message_id: Option<Id<MessageMarker>>,
        filter_ids: Vec<i32>,
    ) -> Self {
        Self {
            bot,
            filter_group_ids: filter_ids,
            guild_id,
            user_id,
            voter_id,
            channel_id,
            message_id,
            user: None,
            user_is_bot: None,
            voter: None,
            message: None,
            filters: None,
        }
    }

    async fn evaluate_check(&mut self, check: &Filter, premium: bool) -> StarboardResult<bool> {
        // user context
        if let Some(req) = check.user_is_bot {
            let Some(is_bot) = self.get_user_is_bot().await? else {
                return Ok(false);
            };

            if is_bot != req {
                return Ok(false);
            }
        }

        if let Some(req) = &check.user_has_all_of {
            if !has_all_roles(&self.get_user_roles().await?, req) {
                return Ok(false);
            }
        }

        if let Some(req) = &check.user_has_some_of {
            if !has_any_role(&self.get_user_roles().await?, req) {
                return Ok(false);
            }
        }

        if let Some(req) = &check.user_missing_all_of {
            if has_any_role(&self.get_user_roles().await?, req) {
                return Ok(false);
            }
        }

        if let Some(req) = &check.user_missing_some_of {
            if has_all_roles(&self.get_user_roles().await?, req) {
                return Ok(false);
            }
        }

        // message context
        // if there's no channel_id or message_id, then we're not in the message context,
        // and as such should not attempt validation for it.
        let Some(mut channel_id) = self.channel_id else {
            return Ok(true);
        };
        let Some(message_id) = self.message_id else {
            return Ok(true);
        };

        // the initial message for a forum thread is treated as belonging to the forum
        // channel itself, for two reasons:
        // 1. This allows you to distinguish between initial posts and followup messages
        // 2. You can't actually send any messages in a forum
        //
        // Only forum starter messages will have self.id == self.channel.id
        if channel_id.get() == message_id.get() {
            if let Some(parent_id) = self
                .bot
                .cache
                .fog_parent_channel_id(self.bot, self.guild_id, channel_id)
                .await?
            {
                channel_id = parent_id;
            }
        }

        if let Some(req) = &check.in_channel {
            if !req.contains(&channel_id.get_i64()) {
                return Ok(false);
            }
        }

        if let Some(req) = &check.not_in_channel {
            if req.contains(&channel_id.get_i64()) {
                return Ok(false);
            }
        }

        if let Some(req) = &check.in_channel_or_sub_channels {
            let qualified_channel_ids = self
                .bot
                .cache
                .qualified_channel_ids(self.bot, self.guild_id, channel_id)
                .await?;
            let mut any_valid = false;
            for id in qualified_channel_ids {
                if req.contains(&id.get_i64()) {
                    any_valid = true;
                    break;
                }
            }
            if !any_valid {
                return Ok(false);
            }
        }

        if let Some(req) = &check.not_in_channel_or_sub_channels {
            let qualified_channel_ids = self
                .bot
                .cache
                .qualified_channel_ids(self.bot, self.guild_id, channel_id)
                .await?;
            let mut any_valid = false;
            for id in qualified_channel_ids {
                if req.contains(&id.get_i64()) {
                    any_valid = true;
                    break;
                }
            }
            if any_valid {
                return Ok(false);
            }
        }

        if let Some(req) = check.min_length {
            let MessageResult::Ok(message) = self.get_message().await? else {
                return Ok(false);
            };
            if message.content.len() < req as usize {
                return Ok(false);
            }
        }

        if let Some(req) = check.max_length {
            let MessageResult::Ok(message) = self.get_message().await? else {
                return Ok(false);
            };
            if message.content.len() > req as usize {
                return Ok(false);
            }
        }

        if let Some(req) = check.min_attachments {
            let MessageResult::Ok(message) = self.get_message().await? else {
                return Ok(false);
            };
            let count = message.attachments.len() + message.embeds.len();
            if count < req as usize {
                return Ok(false);
            }
        }

        if let Some(req) = check.max_attachments {
            let MessageResult::Ok(message) = self.get_message().await? else {
                return Ok(false);
            };
            let count = message.attachments.len() + message.embeds.len();
            if count > req as usize {
                return Ok(false);
            }
        }

        if premium {
            if let Some(req) = &check.matches {
                let MessageResult::Ok(message) = self.get_message().await? else {
                    return Ok(false);
                };

                let re = regex::Regex::new(req)?;
                if !re.is_match(&message.content) {
                    return Ok(false);
                }
            }

            if let Some(req) = &check.not_matches {
                let MessageResult::Ok(message) = self.get_message().await? else {
                    return Ok(false);
                };

                let re = regex::Regex::new(req)?;
                if re.is_match(&message.content) {
                    return Ok(false);
                }
            }
        }

        // vote context
        if self.voter_id.is_none() {
            return Ok(true);
        }

        if let Some(req) = &check.voter_has_all_of {
            if !has_all_roles(&self.get_voter_roles().await?, req) {
                return Ok(false);
            }
        }

        if let Some(req) = &check.voter_has_some_of {
            if !has_any_role(&self.get_voter_roles().await?, req) {
                return Ok(false);
            }
        }

        if let Some(req) = &check.voter_missing_all_of {
            if has_any_role(&self.get_voter_roles().await?, req) {
                return Ok(false);
            }
        }

        if let Some(req) = &check.voter_missing_some_of {
            if has_all_roles(&self.get_voter_roles().await?, req) {
                return Ok(false);
            }
        }

        let age_secs = message_id.age().as_secs();
        if let Some(req) = check.newer_than {
            if age_secs > req as u64 {
                return Ok(false);
            }
        }
        if let Some(req) = check.older_than {
            if age_secs < req as u64 {
                return Ok(false);
            }
        }

        Ok(true)
    }

    pub async fn status(&mut self) -> StarboardResult<bool> {
        let filters = self.get_filters().await?;
        let premium = is_guild_premium(self.bot, self.guild_id.get_i64(), true).await?;
        let filters_iter: &[_] = &filters;
        for filter in filters_iter {
            for check in filter {
                let status = self.evaluate_check(check, premium).await?;
                if !status && check.instant_fail {
                    return Ok(false);
                }
                if status && check.instant_pass {
                    return Ok(true);
                }
            }
        }

        Ok(true)
    }

    // caching
    pub fn set_user(&mut self, user: Option<Arc<CachedMember>>) {
        self.user.replace(user);
    }

    async fn get_user(&mut self) -> StarboardResult<Option<Arc<CachedMember>>> {
        if let Some(value) = self.user.clone() {
            return Ok(value);
        }

        let value = self
            .bot
            .cache
            .fog_member(self.bot, self.guild_id, self.user_id)
            .await?;
        self.set_user(value.clone());
        Ok(value)
    }

    async fn get_user_roles(&mut self) -> StarboardResult<Vec<i64>> {
        match self.get_user().await? {
            None => Ok(vec![self.guild_id.get_i64()]),
            Some(user) => Ok(user.roles.iter().map(|r| r.get_i64()).collect()),
        }
    }

    pub fn set_user_is_bot(&mut self, user_is_bot: Option<bool>) {
        self.user_is_bot.replace(user_is_bot);
    }

    async fn get_user_is_bot(&mut self) -> StarboardResult<Option<bool>> {
        if let Some(value) = self.user_is_bot {
            return Ok(value);
        }

        let sql_user = DbUser::get(&self.bot.db, self.user_id.get_i64()).await?;
        if let Some(sql_user) = sql_user {
            self.set_user_is_bot(Some(sql_user.is_bot));
            return Ok(Some(sql_user.is_bot));
        }

        self.set_user_is_bot(None);
        Ok(None)
    }

    pub fn set_voter(&mut self, voter: Option<Arc<CachedMember>>) {
        self.voter.replace(voter);
    }

    async fn get_voter(&mut self) -> StarboardResult<Option<Arc<CachedMember>>> {
        if let Some(value) = self.voter.clone() {
            return Ok(value);
        }

        let voter_id = self.voter_id.expect("get_voter called without voter_id");

        let value = self
            .bot
            .cache
            .fog_member(self.bot, self.guild_id, voter_id)
            .await?;
        self.set_voter(value.clone());
        Ok(value)
    }

    async fn get_voter_roles(&mut self) -> StarboardResult<Vec<i64>> {
        let roles = match self.get_voter().await? {
            None => vec![self.guild_id.get_i64()],
            Some(user) => user.roles.iter().map(|r| r.get_i64()).collect(),
        };

        Ok(roles)
    }

    pub fn set_message(&mut self, message: impl Into<MessageResult>) {
        self.message.replace(message.into());
    }

    async fn get_message(&mut self) -> StarboardResult<MessageResult> {
        if let Some(value) = self.message.clone() {
            return Ok(value);
        }

        let message_id = self
            .message_id
            .expect("get_message called without message_id");
        let channel_id = self
            .channel_id
            .expect("get_message called without channel_id");

        let value = self
            .bot
            .cache
            .fog_message(self.bot, channel_id, message_id)
            .await?;
        self.set_message(value.clone());
        Ok(value)
    }

    pub fn set_filters(&mut self, filters: Arc<Vec<Vec<Filter>>>) {
        self.filters.replace(filters);
    }

    async fn get_filters(&mut self) -> StarboardResult<Arc<Vec<Vec<Filter>>>> {
        if let Some(value) = self.filters.clone() {
            return Ok(value);
        }

        let mut filters = Vec::new();
        for filter_id in &self.filter_group_ids {
            let checks = Filter::list_by_filter(&self.bot.db, *filter_id).await?;
            filters.push(checks);
        }

        let filters = Arc::new(filters);
        self.set_filters(filters.clone());
        Ok(filters)
    }
}
