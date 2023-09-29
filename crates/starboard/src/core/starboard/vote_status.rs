//! tool for checking if a vote is valid

use std::time::Duration;

use twilight_model::id::{
    marker::{ChannelMarker, MessageMarker, UserMarker},
    Id,
};

use database::StarboardFilterGroup;
use errors::StarboardResult;

use crate::{
    client::bot::StarboardBot,
    core::{
        emoji::SimpleEmoji, filters::FilterEvaluater, has_image::has_image, permroles::Permissions,
    },
    utils::{into_id::IntoId, snowflake_age::SnowflakeAge},
};

use super::config::StarboardConfig;

pub struct VoteContext<'a> {
    pub emoji: &'a SimpleEmoji,
    pub reactor_id: Id<UserMarker>,
    pub message_id: Id<MessageMarker>,
    pub channel_id: Id<ChannelMarker>,
    pub message_author_id: Id<UserMarker>,
    pub message_author_is_bot: bool,
    pub message_has_image: Option<bool>,
    pub message_is_frozen: bool,
}

#[derive(Debug)]
pub enum VoteStatus<'a> {
    Ignore,
    Remove,
    Valid((Vec<&'a StarboardConfig>, Vec<&'a StarboardConfig>)),
}

impl<'a> VoteStatus<'a> {
    pub async fn get_vote_status(
        bot: &StarboardBot,
        vote: VoteContext<'_>,
        configs: &'a [StarboardConfig],
    ) -> StarboardResult<VoteStatus<'a>> {
        if vote.message_is_frozen {
            return Ok(VoteStatus::Ignore);
        }

        let message_has_image = match vote.message_has_image {
            Some(val) => Some(val),
            None => bot
                .cache
                .fog_message(bot, vote.channel_id, vote.message_id)
                .await?
                .into_option()
                .as_ref()
                .as_ref()
                .map(|msg| has_image(&msg.embeds, &msg.attachments)),
        };

        let mut invalid_exists = false;
        let mut allow_remove = true;

        #[derive(Clone, Copy, PartialEq, Eq)]
        enum VoteType {
            Upvote,
            Downvote,
        }

        let eval_config = |config: &'a StarboardConfig| -> Option<(&'a StarboardConfig, VoteType)> {
            // skip disabled configurations
            if !config.resolved.enabled || config.starboard.premium_locked {
                return None;
            }

            let vote_type = if config
                .resolved
                .upvote_emojis
                .iter()
                .any(|e| vote.emoji == e)
            {
                VoteType::Upvote
            } else if config
                .resolved
                .downvote_emojis
                .iter()
                .any(|e| vote.emoji == e)
            {
                VoteType::Downvote
            } else {
                return None;
            };

            // respect the `remove_invalid_reactions` setting
            if !config.resolved.remove_invalid_reactions {
                allow_remove = false;
            }

            // message age in seconds
            let message_age = vote.message_id.age().as_secs();

            let min_age = config.resolved.older_than;
            let max_age = config.resolved.newer_than;

            let self_vote_valid =
                config.resolved.self_vote || vote.reactor_id != vote.message_author_id;

            let bots_valid = config.resolved.allow_bots || !vote.message_author_is_bot;

            let images_valid = !config.resolved.require_image || (message_has_image == Some(true));

            let time_valid = {
                let min_age_valid = if min_age <= 0 {
                    true
                } else {
                    message_age > min_age as u64
                };
                let max_age_valid = if max_age <= 0 {
                    true
                } else {
                    message_age < max_age as u64
                };
                min_age_valid && max_age_valid
            };

            if self_vote_valid && bots_valid && images_valid && time_valid {
                Some((config, vote_type))
            } else {
                invalid_exists = true;
                None
            }
        };

        let mut upvote = Vec::new();
        let mut downvote = Vec::new();

        let mut invalid_exists_2 = false;

        for (config, vote_type) in configs.iter().filter_map(eval_config) {
            // check reactor/author role permissions
            let reactor_perms = Permissions::get_permissions(
                bot,
                vote.reactor_id,
                config.starboard.guild_id.into_id(),
                Some(config.starboard.id),
            )
            .await?;
            let author_perms = Permissions::get_permissions(
                bot,
                vote.message_author_id,
                config.starboard.guild_id.into_id(),
                Some(config.starboard.id),
            )
            .await?;

            if !reactor_perms.give_votes || !author_perms.receive_votes {
                invalid_exists_2 = true;
                continue;
            }

            // check filters
            let filter_groups =
                StarboardFilterGroup::list_by_starboard(&bot.db, config.starboard.id).await?;
            let mut evaluater = FilterEvaluater::new(
                bot,
                config.starboard.guild_id.into_id(),
                vote.message_author_id,
                Some(vote.reactor_id),
                Some(vote.channel_id),
                Some(vote.message_id),
                filter_groups.iter().map(|g| g.filter_group_id).collect(),
            );
            evaluater.set_user_is_bot(Some(vote.message_author_is_bot));
            let status = evaluater.status().await?;

            if !status {
                invalid_exists_2 = true;
                continue;
            }

            // check cooldown
            if config.resolved.cooldown_enabled
                && bot
                    .cooldowns
                    .starboard_custom_cooldown
                    .trigger(
                        &(vote.reactor_id, config.starboard.id),
                        config.resolved.cooldown_count as u64,
                        Duration::from_secs(config.resolved.cooldown_period as u64),
                    )
                    .is_some()
            {
                invalid_exists_2 = true;
                continue;
            }

            if vote_type == VoteType::Upvote {
                upvote.push(config);
            } else {
                downvote.push(config);
            }
        }

        if upvote.is_empty() && downvote.is_empty() {
            if (invalid_exists || invalid_exists_2) && allow_remove {
                Ok(VoteStatus::Remove)
            } else {
                Ok(VoteStatus::Ignore)
            }
        } else {
            Ok(VoteStatus::Valid((upvote, downvote)))
        }
    }
}
