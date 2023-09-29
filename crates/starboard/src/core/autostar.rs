use std::{sync::Arc, time::Duration};

use twilight_model::id::{
    marker::{ChannelMarker, GuildMarker, MessageMarker},
    Id,
};

use database::{AutoStarChannel, AutostarChannelFilterGroup};
use errors::StarboardResult;

use crate::{
    cache::{models::message::CachedMessage, MessageResult},
    client::bot::StarboardBot,
    core::emoji::{EmojiCommon, SimpleEmoji},
    utils::{id_as_i64::GetI64, notify},
};

use super::{
    filters::FilterEvaluater, has_image::has_image, premium::is_premium::is_guild_premium,
};

pub async fn handle(
    bot: &StarboardBot,
    guild_id: Id<GuildMarker>,
    autostar_channel_id: Id<ChannelMarker>,
    channel_id: Id<ChannelMarker>,
    message_id: Id<MessageMarker>,
    message: Option<Arc<CachedMessage>>,
) -> StarboardResult<()> {
    // Check the cache...
    if !bot
        .cache
        .autostar_channel_ids
        .contains(&autostar_channel_id)
    {
        return Ok(());
    }

    // Check cooldown
    if is_guild_premium(bot, guild_id.get_i64(), true).await? {
        if bot
            .cooldowns
            .prem_autostar_send
            .trigger(&guild_id)
            .is_some()
        {
            return Ok(());
        }
    } else if bot.cooldowns.autostar_send.trigger(&guild_id).is_some() {
        return Ok(());
    }

    // Fetch the autostar channels
    let asc = AutoStarChannel::list_by_channel(&bot.db, autostar_channel_id.get_i64()).await?;
    let asc: Vec<_> = asc.into_iter().filter(|a| !a.premium_locked).collect();

    // If none, remove the channel id from the cache
    if asc.is_empty() {
        bot.cache.autostar_channel_ids.remove(&autostar_channel_id);
        return Ok(());
    }

    let message = match message {
        Some(msg) => msg,
        None => {
            let Some(msg) = bot
                .cache
                .fog_message(bot, channel_id, message_id)
                .await?
                .into_option()
            else {
                return Ok(());
            };
            msg
        }
    };

    // Handle the autostar channels
    let mut to_react = Vec::new();
    for a in asc {
        let status = get_status(bot, &a, guild_id, channel_id, message_id, message.clone()).await?;

        if matches!(status, Status::InvalidStay) {
            continue;
        }
        if let Status::InvalidRemove(reasons) = status {
            let _ = bot.http.delete_message(channel_id, message_id).await;

            let send = bot
                .cache
                .fog_user(bot, message.author_id)
                .await?
                .map_or(false, |u| !u.is_bot);
            if send {
                let to_send = {
                    format!(
                        "Your message in <#{channel_id}> was deleted for the following reason(s):\n"
                    ) + &reasons.join("\n")
                };
                notify::notify(bot, message.author_id, &to_send).await?;
            }

            return Ok(());
        }

        to_react.extend(Vec::<SimpleEmoji>::from_stored(a.emojis));
    }

    for emoji in to_react {
        let _ = bot
            .http
            .create_reaction(channel_id, message_id, &emoji.reactable())
            .await;
    }

    Ok(())
}

enum Status {
    Valid,
    InvalidStay,
    InvalidRemove(Vec<String>),
}

async fn get_status(
    bot: &StarboardBot,
    asc: &AutoStarChannel,
    guild_id: Id<GuildMarker>,
    channel_id: Id<ChannelMarker>,
    message_id: Id<MessageMarker>,
    event: Arc<CachedMessage>,
) -> StarboardResult<Status> {
    let mut invalid = Vec::new();

    if asc.min_chars != 0 && event.content.len() < asc.min_chars as usize {
        invalid.push(format!(
            "- Your message must have at least {} characters.",
            asc.min_chars
        ));
    }
    if let Some(max_chars) = asc.max_chars {
        if event.content.len() > max_chars as usize {
            invalid.push(format!(
                "- Your message cannot be longer than {max_chars} characters.",
            ));
        }
    }
    if asc.require_image && !has_image(&event.embeds, &event.attachments) {
        tokio::time::sleep(Duration::from_secs(3)).await;

        let updated_msg = bot.cache.fog_message(bot, channel_id, message_id).await?;
        let mut still_invalid = true;

        let msg = match updated_msg.into_option() {
            None => return Ok(Status::InvalidStay),
            Some(msg) => msg,
        };
        if has_image(&msg.embeds, &msg.attachments) {
            still_invalid = false;
        }

        if still_invalid {
            invalid.push("- Your message must include an image.".to_string());
        }
    }

    let filter_groups =
        AutostarChannelFilterGroup::list_by_autostar_channel(&bot.db, asc.id).await?;
    let mut filters = FilterEvaluater::new(
        bot,
        guild_id,
        event.author_id,
        None,
        Some(channel_id),
        Some(message_id),
        filter_groups.iter().map(|g| g.filter_group_id).collect(),
    );
    filters.set_message(MessageResult::Ok(event));
    if !filters.status().await? {
        invalid.push("- Your message does not meet the filter requirements.".to_string());
    }

    if invalid.is_empty() {
        Ok(Status::Valid)
    } else if asc.delete_invalid {
        Ok(Status::InvalidRemove(invalid))
    } else {
        Ok(Status::InvalidStay)
    }
}
