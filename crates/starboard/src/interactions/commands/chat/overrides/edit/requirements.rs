use twilight_interactions::command::{CommandModel, CreateCommand};

use database::{
    validation::{self, ToBotStr},
    Starboard, StarboardOverride,
};
use errors::StarboardResult;

use crate::{
    core::{
        emoji::{EmojiCommon, SimpleEmoji},
        premium::is_premium::is_guild_premium,
        starboard::config::StarboardConfig,
    },
    get_guild_id,
    interactions::context::CommandCtx,
    parsing,
    utils::id_as_i64::GetI64,
};

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "requirements",
    desc = "Edit the requirements for messages to appear on the starboard."
)]
pub struct EditRequirements {
    /// The override to edit.
    #[command(autocomplete = true)]
    name: String,

    /// The number of upvotes a message needs. Use "none" to unset.
    required: Option<String>,
    /// How few points the message can have before a starboarded post is removed. Use "none" to unset.
    #[command(rename = "required-remove")]
    required_remove: Option<String>,
    /// The emojis that can be used to upvote a post. Use 'none' to remove all.
    #[command(rename = "upvote-emojis")]
    upvote_emojis: Option<String>,
    /// The emojis that can be used to downvote a post. Use 'none' to remove all.
    #[command(rename = "downvote-emojis")]
    downvote_emojis: Option<String>,
    /// Whether to allow users to vote on their own posts.
    #[command(rename = "self-vote")]
    self_vote: Option<bool>,
    /// Whether to allow bot messages to be on the starboard.
    #[command(rename = "allow-bots")]
    allow_bots: Option<bool>,
    /// Whether to require posts to have an image to appear on the starboard.
    #[command(rename = "require-image")]
    require_image: Option<bool>,
    /// How old a post must be in order for it to be voted on (e.g. "1 hour"). Use 0 to disable.
    #[command(rename = "older-than")]
    older_than: Option<String>,
    /// How new a post must be in order for it to be voted on (e.g. "1 hour"). Use 0 to disable.
    #[command(rename = "newer-than")]
    newer_than: Option<String>,
    /// (Premium) Content that messages must match to be starred (supports regex). Use ".*" to disable.
    matches: Option<String>,
    #[command(rename = "not-matches")]
    /// (Premium) content that messages must not match to be starred (supports regex). Use ".*" to disable.
    not_matches: Option<String>,
}

impl EditRequirements {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let guild_id = get_guild_id!(ctx);
        let guild_id_i64 = guild_id.get_i64();
        let ov = match StarboardOverride::get(&ctx.bot.db, guild_id_i64, &self.name).await? {
            None => {
                ctx.respond_str("No override with that name was found.", true)
                    .await?;
                return Ok(());
            }
            Some(starboard) => starboard,
        };
        let (ov, resolved) = {
            let starboard = Starboard::get(&ctx.bot.db, ov.starboard_id).await?.unwrap();
            let mut resolved = StarboardConfig::new(starboard, &[], vec![ov])?;

            (resolved.overrides.remove(0), resolved.resolved)
        };
        let mut settings = ov.get_overrides()?;

        let is_prem = is_guild_premium(&ctx.bot, guild_id_i64, true).await?;

        if let Some(val) = self.required {
            let val =
                match parsing::starboard_settings::parse_required(&val, resolved.required_remove) {
                    Ok(val) => val,
                    Err(why) => {
                        ctx.respond_str(&why, true).await?;
                        return Ok(());
                    }
                };
            settings.required = Some(val);
        }
        if let Some(val) = self.required_remove {
            let val =
                match parsing::starboard_settings::parse_required_remove(&val, resolved.required) {
                    Ok(val) => val,
                    Err(why) => {
                        ctx.respond_str(&why, true).await?;
                        return Ok(());
                    }
                };
            settings.required_remove = Some(val);
        }

        if let Some(val) = self.upvote_emojis {
            let emojis = SimpleEmoji::from_user_input(&val, &ctx.bot, guild_id).into_stored();
            settings.upvote_emojis = Some(emojis);

            // delete cached value
            ctx.bot.cache.guild_vote_emojis.remove(&guild_id_i64);
        }
        if let Some(val) = self.downvote_emojis {
            let emojis = SimpleEmoji::from_user_input(&val, &ctx.bot, guild_id).into_stored();
            settings.downvote_emojis = Some(emojis);

            // delete cached value
            ctx.bot.cache.guild_vote_emojis.remove(&guild_id_i64);
        }
        if let Err(why) = validation::starboard_settings::validate_vote_emojis(
            settings
                .upvote_emojis
                .as_ref()
                .unwrap_or(&resolved.upvote_emojis),
            settings
                .downvote_emojis
                .as_ref()
                .unwrap_or(&resolved.downvote_emojis),
            is_prem,
        ) {
            ctx.respond_str(&why.to_bot_str(), true).await?;
            return Ok(());
        }

        if let Some(val) = self.self_vote {
            settings.self_vote = Some(val);
        }
        if let Some(val) = self.allow_bots {
            settings.allow_bots = Some(val);
        }
        if let Some(val) = self.require_image {
            settings.require_image = Some(val);
        }
        if let Some(val) = self.older_than {
            let delta = match parsing::time_delta::parse_time_delta(&val) {
                Err(why) => {
                    ctx.respond_str(&why, true).await?;
                    return Ok(());
                }
                Ok(delta) => delta,
            };
            settings.older_than = Some(delta);
        }
        if let Some(val) = self.newer_than {
            let delta = match parsing::time_delta::parse_time_delta(&val) {
                Err(why) => {
                    ctx.respond_str(&why, true).await?;
                    return Ok(());
                }
                Ok(delta) => delta,
            };
            settings.newer_than = Some(delta);
        }

        if let Err(why) = validation::relative_duration::validate_relative_duration(
            Some(settings.newer_than.unwrap_or(resolved.newer_than)),
            Some(settings.older_than.unwrap_or(resolved.older_than)),
        ) {
            ctx.respond_str(&why.to_bot_str(), true).await?;
            return Ok(());
        }

        if let Some(val) = self.matches {
            match validation::regex::validate_regex(val, is_prem) {
                Err(why) => {
                    ctx.respond_str(&why.to_bot_str(), true).await?;
                    return Ok(());
                }
                Ok(val) => settings.matches = Some(val),
            }
        }
        if let Some(val) = self.not_matches {
            match validation::regex::validate_regex(val, is_prem) {
                Err(why) => {
                    ctx.respond_str(&why.to_bot_str(), true).await?;
                    return Ok(());
                }
                Ok(val) => settings.not_matches = Some(val),
            }
        }

        StarboardOverride::update_settings(&ctx.bot.db, ov.id, settings).await?;
        ctx.respond_str(
            &format!("Updated settings for override '{}'.", self.name),
            false,
        )
        .await?;

        Ok(())
    }
}
