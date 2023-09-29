use database::DbMessage;
use errors::StarboardResult;

use crate::{cache::MessageResult, client::bot::StarboardBot, utils::into_id::IntoId};

use super::config::StarboardConfig;

#[derive(Debug)]
pub enum MessageStatus {
    /// true -> full update, false -> partial update
    Update(bool),
    Remove,
    /// true -> full update, false -> partial update
    Send(bool),
}

pub async fn get_message_status(
    bot: &StarboardBot,
    config: &StarboardConfig,
    message: &DbMessage,
    message_obj: &MessageResult,
    points: i32,
    violates_exclusive_group: bool,
    is_premium: bool,
) -> StarboardResult<MessageStatus> {
    let deleted = matches!(message_obj, MessageResult::Missing);

    let guild_id = config.starboard.guild_id.into_id();
    let sb_channel_id = config.starboard.channel_id.into_id();
    let sb_is_nsfw = bot
        .cache
        .fog_channel_nsfw(bot, guild_id, sb_channel_id)
        .await?;

    let sb_is_nsfw = match sb_is_nsfw {
        Some(val) => val,
        None => return Ok(MessageStatus::Update(config.resolved.link_edits)),
    };

    if (deleted && config.resolved.link_deletes)
        || (message.is_nsfw && !sb_is_nsfw)
        || message.trashed
    {
        return Ok(MessageStatus::Remove);
    }

    if message.forced_to.contains(&config.starboard.id) {
        return Ok(MessageStatus::Send(config.resolved.link_edits));
    }
    if violates_exclusive_group {
        return Ok(MessageStatus::Remove);
    }

    if message.frozen {
        return Ok(MessageStatus::Update(false));
    }

    if let Some(required_remove) = config.resolved.required_remove {
        if points <= required_remove as i32 {
            return Ok(MessageStatus::Remove);
        }
    }

    if let Some(required) = config.resolved.required {
        if validate_regex(config, message_obj, is_premium) {
            #[allow(clippy:collapsible_if)]
            if points >= required as i32 {
                return Ok(MessageStatus::Send(config.resolved.link_edits));
            }
        }
    }

    Ok(MessageStatus::Update(config.resolved.link_edits))
}

fn validate_regex(config: &StarboardConfig, message_obj: &MessageResult, is_premium: bool) -> bool {
    if !is_premium {
        return true;
    }

    if config.resolved.matches.is_none() && config.resolved.not_matches.is_none() {
        return true;
    }

    let MessageResult::Ok(message_obj) = message_obj else {
        return false;
    };

    if let Some(re) = &config.resolved.matches {
        if let Ok(re) = regex::Regex::new(re) {
            if !re.is_match(&message_obj.content) {
                return false;
            }
        }
    }
    if let Some(re) = &config.resolved.not_matches {
        if let Ok(re) = regex::Regex::new(re) {
            if re.is_match(&message_obj.content) {
                return false;
            }
        }
    }

    true
}
