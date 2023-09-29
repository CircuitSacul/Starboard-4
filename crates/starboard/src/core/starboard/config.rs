use std::cmp::Ordering;

use twilight_model::id::{
    marker::{ChannelMarker, GuildMarker},
    Id,
};

use database::{call_with_override_settings, Starboard, StarboardOverride, StarboardSettings};
use errors::{StarboardError, StarboardResult};

use crate::{
    client::bot::StarboardBot,
    core::emoji::{EmojiCommon, SimpleEmoji},
    utils::id_as_i64::GetI64,
};

#[derive(Debug)]
pub struct StarboardConfig {
    pub starboard: Starboard,
    pub overrides: Vec<StarboardOverride>,
    pub resolved: StarboardSettings,
}

macro_rules! update_from_override {
    ($settings: expr, $override: expr, $($field: ident),*) => {
        {
            $(
                match $override.$field {
                    Option::None => {},
                    Option::Some(value) => $settings.$field = value,
                }
            )*
        }
    };
}

impl StarboardConfig {
    pub fn new(
        starboard: Starboard,
        channel_ids: &[i64],
        mut overrides: Vec<StarboardOverride>,
    ) -> serde_json::Result<Self> {
        overrides.sort_by(|a, b| {
            for cmp in channel_ids
                .iter()
                .map(|id| (a.channel_ids.contains(id), b.channel_ids.contains(id)))
            {
                match cmp.0.cmp(&cmp.1) {
                    Ordering::Equal => continue,
                    val => return val,
                }
            }

            Ordering::Equal
        });

        let mut settings = starboard.settings.clone();
        for ov in overrides.iter() {
            call_with_override_settings!(update_from_override, settings, ov.get_overrides()?)
        }

        Ok(Self {
            starboard,
            overrides,
            resolved: settings,
        })
    }

    pub async fn list_for_channel(
        bot: &StarboardBot,
        guild_id: Id<GuildMarker>,
        channel_id: Id<ChannelMarker>,
    ) -> Result<Vec<Self>, StarboardError> {
        let starboards = Starboard::list_by_guild(&bot.db, guild_id.get_i64()).await?;
        let mut configs = Vec::new();

        let channel_ids: Vec<i64> = bot
            .cache
            .qualified_channel_ids(bot, guild_id, channel_id)
            .await?
            .into_iter()
            .map(|cid| cid.get_i64())
            .collect();

        for sb in starboards.into_iter() {
            let overrides =
                StarboardOverride::list_by_starboard_and_channels(&bot.db, sb.id, &channel_ids)
                    .await?;
            configs.push(Self::new(sb, &channel_ids, overrides)?);
        }

        Ok(configs)
    }

    pub async fn is_guild_vote_emoji(
        bot: &StarboardBot,
        guild_id: i64,
        emoji_raw: &SimpleEmoji,
    ) -> StarboardResult<bool> {
        if let Some(is_vote_emoji) = bot.cache.guild_vote_emojis.with(&guild_id, |_, emojis| {
            emojis.as_ref().map(|emojis| emojis.contains(emoji_raw))
        }) {
            Ok(is_vote_emoji)
        } else {
            let mut emojis = Vec::new();
            let starboards = Starboard::list_by_guild(&bot.db, guild_id).await?;
            for sb in starboards {
                emojis.extend(sb.settings.upvote_emojis);
                emojis.extend(sb.settings.downvote_emojis);

                let configs = StarboardOverride::list_by_starboard(&bot.db, sb.id).await?;
                for c in configs {
                    let ov = c.get_overrides()?;
                    if let Some(upvote_emojis) = ov.upvote_emojis {
                        emojis.extend(upvote_emojis);
                    }
                    if let Some(downvote_emojis) = ov.downvote_emojis {
                        emojis.extend(downvote_emojis);
                    }
                }
            }

            let emojis = Vec::<SimpleEmoji>::from_stored(emojis);
            let is_vote_emoji = emojis.contains(emoji_raw);
            // cache the value
            bot.cache.guild_vote_emojis.insert(guild_id, emojis);

            Ok(is_vote_emoji)
        }
    }
}
