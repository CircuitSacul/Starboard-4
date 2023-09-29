use twilight_interactions::command::{CommandModel, CommandOption, CreateCommand, CreateOption};

use common::constants;
use database::{
    validation::{self, ToBotStr},
    Filter, FilterGroup,
};
use errors::StarboardResult;

use crate::{
    core::premium::is_premium::is_guild_premium, get_guild_id, interactions::context::CommandCtx,
    parsing, utils::id_as_i64::GetI64,
};

fn validate_roles(length: usize) -> Result<(), String> {
    if length > constants::MAX_FILTER_ROLES {
        Err(format!(
            "You can only have up to {} roles in a list.",
            constants::MAX_FILTER_ROLES
        ))
    } else {
        Ok(())
    }
}

fn validate_channels(length: usize) -> Result<(), String> {
    if length > constants::MAX_FILTER_CHANNELS {
        Err(format!(
            "You can only have up to {} channels in a list.",
            constants::MAX_FILTER_CHANNELS
        ))
    } else {
        Ok(())
    }
}

#[derive(CreateOption, CommandOption)]
pub enum UserBotRequirement {
    #[option(name = "User must be a bot", value = 0)]
    MustBeBot,
    #[option(name = "User must not be a bot", value = 1)]
    MustBeHuman,
    #[option(name = "Disabled", value = 2)]
    Disabled,
}

impl From<UserBotRequirement> for Option<bool> {
    fn from(val: UserBotRequirement) -> Self {
        match val {
            UserBotRequirement::MustBeBot => Some(true),
            UserBotRequirement::MustBeHuman => Some(false),
            UserBotRequirement::Disabled => None,
        }
    }
}

#[derive(CommandModel, CreateCommand)]
#[command(name = "edit", desc = "Edit a filters conditions.")]
pub struct Edit {
    /// The name of the filter group containing the filter to be edited.
    #[command(autocomplete = true)]
    group: String,
    /// The position of the filter to edit.
    #[command(min_value = 1, max_value = 1_000)]
    position: i64,

    // general info
    /// If true and this filter passes, the entire filter groups passes.
    #[command(rename = "instant-pass")]
    instant_pass: Option<bool>,
    /// If true and this filter fails, then the entire filter group fails.
    #[command(rename = "instant-fail")]
    instant_fail: Option<bool>,

    // default context
    /// Require that the user/author has all of these roles.
    #[command(rename = "user-has-all-of")]
    user_has_all_of: Option<String>,
    /// Require that the user/author has at least one of these roles.
    #[command(rename = "user-has-some-of")]
    user_has_some_of: Option<String>,
    /// Require that the user/author is missing all of these roles.
    #[command(rename = "user-missing-all-of")]
    user_missing_all_of: Option<String>,
    /// Require that the user/author is missing at least one of these roles.
    #[command(rename = "user-missing-some-of")]
    user_missing_some_of: Option<String>,
    /// Require that the user is or is not a bot.
    #[command(rename = "user-is-bot")]
    user_is_bot: Option<UserBotRequirement>,

    // message context
    /// Require that the message was sent in one of these channels.
    #[command(rename = "in-channel")]
    in_channel: Option<String>,
    /// Require that the message was not sent in one of these channels.
    #[command(rename = "not-in-channel")]
    not_in_channel: Option<String>,
    /// Require that the message was sent in one of these channels or their sub-channels.
    #[command(rename = "in-channel-or-sub-channels")]
    in_channel_or_sub_channels: Option<String>,
    /// Require that the message was not sent in one of these channels or their sub-channels.
    #[command(rename = "not-in-channel-or-sub-channels")]
    not_in_channel_or_sub_channels: Option<String>,
    /// Require that the message has at least this many attachments. Use 0 to disable.
    #[command(rename = "min-attachments")]
    min_attachments: Option<i64>,
    /// Require that the message have at most this many attachments. Use -1 to disable.
    #[command(rename = "max-attachments")]
    max_attachments: Option<i64>,
    /// Require that the message be at least this many characters long. Use 0 to disable.
    #[command(rename = "min-length")]
    min_length: Option<i64>,
    /// Require that the message be at most this many characters long. Use -1 to disable.
    #[command(rename = "max-length")]
    max_length: Option<i64>,
    /// (Premium) Require that the message match this regex. Use `.*` to disable.
    matches: Option<String>,
    /// (Premium) Require that the message not match this regex. Use `.*` to disable.
    #[command(rename = "not-matches")]
    not_matches: Option<String>,

    // vote context
    /// Require that the voter has all of these roles.
    #[command(rename = "voter-has-all-of")]
    voter_has_all_of: Option<String>,
    /// Require that the voter has at least one of these roles.
    #[command(rename = "voter-has-some-of")]
    voter_has_some_of: Option<String>,
    /// Require that the voter is missing all of these roles.
    #[command(rename = "voter-missing-all-of")]
    voter_missing_all_of: Option<String>,
    /// Require that the voter is missing at least one of these roles.
    #[command(rename = "voter-missing-some-of")]
    voter_missing_some_of: Option<String>,
    /// Require that the message being voted on is over a certain age. Use "disable" to disable.
    #[command(rename = "older-than")]
    older_than: Option<String>,
    /// Require that the message being voted on is under a certain age. Use "disable" to disable.
    #[command(rename = "newer-than")]
    newer_than: Option<String>,
}

impl Edit {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let guild_id = get_guild_id!(ctx);
        let guild_id_i64 = guild_id.get_i64();

        let Some(group) = FilterGroup::get_by_name(&ctx.bot.db, guild_id_i64, &self.group).await?
        else {
            ctx.respond_str(
                &format!("No filter group named '{}' exists.", self.group),
                true,
            )
            .await?;
            return Ok(());
        };

        let Some(mut filter) =
            Filter::get_by_position(&ctx.bot.db, group.id, self.position as i16).await?
        else {
            ctx.respond_str(
                &format!(
                    "No filter for group '{}' at {} exists.",
                    self.group, self.position
                ),
                true,
            )
            .await?;
            return Ok(());
        };

        let premium = is_guild_premium(&ctx.bot, guild_id_i64, true).await?;

        // general info
        if let Some(val) = self.instant_pass {
            filter.instant_pass = val;
        }
        if let Some(val) = self.instant_fail {
            filter.instant_fail = val;
        }

        // default context
        if let Some(val) = self.user_has_all_of {
            let roles = parsing::mentions::parse_role_ids(&ctx.bot, guild_id, &val);
            if let Err(why) = validate_roles(roles.len()) {
                ctx.respond_str(&why, true).await?;
                return Ok(());
            }
            if roles.is_empty() {
                filter.user_has_all_of = None;
            } else {
                filter.user_has_all_of = Some(roles.into_iter().collect());
            }
        }
        if let Some(val) = self.user_has_some_of {
            let roles = parsing::mentions::parse_role_ids(&ctx.bot, guild_id, &val);
            if let Err(why) = validate_roles(roles.len()) {
                ctx.respond_str(&why, true).await?;
                return Ok(());
            }
            if roles.is_empty() {
                filter.user_has_some_of = None;
            } else {
                filter.user_has_some_of = Some(roles.into_iter().collect());
            }
        }
        if let Some(val) = self.user_missing_all_of {
            let roles = parsing::mentions::parse_role_ids(&ctx.bot, guild_id, &val);
            if let Err(why) = validate_roles(roles.len()) {
                ctx.respond_str(&why, true).await?;
                return Ok(());
            }
            if roles.is_empty() {
                filter.user_missing_all_of = None;
            } else {
                filter.user_missing_all_of = Some(roles.into_iter().collect());
            }
        }
        if let Some(val) = self.user_missing_some_of {
            let roles = parsing::mentions::parse_role_ids(&ctx.bot, guild_id, &val);
            if let Err(why) = validate_roles(roles.len()) {
                ctx.respond_str(&why, true).await?;
                return Ok(());
            }
            if roles.is_empty() {
                filter.user_missing_some_of = None;
            } else {
                filter.user_missing_some_of = Some(roles.into_iter().collect());
            }
        }
        if let Some(val) = self.user_is_bot {
            filter.user_is_bot = val.into();
        }

        // message context
        if let Some(val) = self.in_channel {
            let channels =
                parsing::mentions::textable_channel_ids(&ctx.bot, guild_id, &val).await?;
            if let Err(why) = validate_channels(channels.len()) {
                ctx.respond_str(&why, true).await?;
                return Ok(());
            }
            if channels.is_empty() {
                filter.in_channel = None;
            } else {
                filter.in_channel = Some(channels.into_iter().collect());
            }
        }
        if let Some(val) = self.not_in_channel {
            let channels =
                parsing::mentions::textable_channel_ids(&ctx.bot, guild_id, &val).await?;
            if let Err(why) = validate_channels(channels.len()) {
                ctx.respond_str(&why, true).await?;
                return Ok(());
            }
            if channels.is_empty() {
                filter.not_in_channel = None;
            } else {
                filter.not_in_channel = Some(channels.into_iter().collect());
            }
        }
        if let Some(val) = self.in_channel_or_sub_channels {
            let channels =
                parsing::mentions::textable_channel_ids(&ctx.bot, guild_id, &val).await?;
            if let Err(why) = validate_channels(channels.len()) {
                ctx.respond_str(&why, true).await?;
                return Ok(());
            }
            if channels.is_empty() {
                filter.in_channel_or_sub_channels = None;
            } else {
                filter.in_channel_or_sub_channels = Some(channels.into_iter().collect());
            }
        }
        if let Some(val) = self.not_in_channel_or_sub_channels {
            let channels =
                parsing::mentions::textable_channel_ids(&ctx.bot, guild_id, &val).await?;
            if let Err(why) = validate_channels(channels.len()) {
                ctx.respond_str(&why, true).await?;
                return Ok(());
            }
            if channels.is_empty() {
                filter.not_in_channel_or_sub_channels = None;
            } else {
                filter.not_in_channel_or_sub_channels = Some(channels.into_iter().collect());
            }
        }
        if let Some(val) = self.min_attachments {
            if val > constants::MAX_ATTACHMENTS {
                ctx.respond_str(
                    &format!(
                        "You can only have up to {} attachments.",
                        constants::MAX_ATTACHMENTS
                    ),
                    true,
                )
                .await?;
                return Ok(());
            } else if val < 0 {
                ctx.respond_str("`min-attachments` must be at least 0.", true)
                    .await?;
                return Ok(());
            }

            if val == 0 {
                filter.min_attachments = None;
            } else {
                filter.min_attachments = Some(val.try_into().unwrap());
            }
        }
        if let Some(val) = self.max_attachments {
            if val > constants::MAX_ATTACHMENTS {
                ctx.respond_str(
                    &format!(
                        "You can only have up to {} attachments.",
                        constants::MAX_ATTACHMENTS
                    ),
                    true,
                )
                .await?;
                return Ok(());
            } else if val < -1 {
                ctx.respond_str("`max-attachments` must be at least -1.", true)
                    .await?;
                return Ok(());
            }

            if val == -1 {
                filter.min_attachments = None;
            } else {
                filter.min_attachments = Some(val.try_into().unwrap());
            }
        }
        if let Some(val) = self.min_length {
            if val > constants::MAX_LENGTH {
                ctx.respond_str(
                    &format!(
                        "`min-length` cannot be longer than {}.",
                        constants::MAX_LENGTH
                    ),
                    true,
                )
                .await?;
                return Ok(());
            } else if val < 0 {
                ctx.respond_str("`min-length` must be at least 0.", true)
                    .await?;
                return Ok(());
            }

            if val == 0 {
                filter.min_length = None;
            } else {
                filter.min_length = Some(val.try_into().unwrap());
            }
        }
        if let Some(val) = self.max_length {
            if val > constants::MAX_LENGTH {
                ctx.respond_str(
                    &format!(
                        "`max-length` cannot be longer than {}.",
                        constants::MAX_LENGTH
                    ),
                    true,
                )
                .await?;
                return Ok(());
            } else if val < -1 {
                ctx.respond_str("`max-length` must be at least -1.", true)
                    .await?;
                return Ok(());
            }

            if val == -1 {
                filter.max_length = None;
            } else {
                filter.max_length = Some(val.try_into().unwrap());
            }
        }
        if let Some(val) = self.matches {
            if val.len() > constants::MAX_REGEX_LENGTH as usize {
                ctx.respond_str(
                    &format!(
                        "`matches` cannot be longer than {}.",
                        constants::MAX_REGEX_LENGTH
                    ),
                    true,
                )
                .await?;
                return Ok(());
            }

            if val == ".*" {
                filter.matches = None;
            } else {
                if !premium {
                    ctx.respond_str(
                        "Only premium servers can use the `matches` condition.",
                        true,
                    )
                    .await?;
                    return Ok(());
                }

                filter.matches = Some(val);
            }
        }
        if let Some(val) = self.not_matches {
            if val.len() > constants::MAX_REGEX_LENGTH as usize {
                ctx.respond_str(
                    &format!(
                        "`not-matches` cannot be longer than {}.",
                        constants::MAX_REGEX_LENGTH
                    ),
                    true,
                )
                .await?;
                return Ok(());
            }

            if val == ".*" {
                filter.not_matches = None;
            } else {
                if !premium {
                    ctx.respond_str(
                        "Only premium servers can use the `not-matches` condition.",
                        true,
                    )
                    .await?;
                    return Ok(());
                }

                filter.not_matches = Some(val);
            }
        }

        // vote context
        if let Some(val) = self.voter_has_all_of {
            let roles = parsing::mentions::parse_role_ids(&ctx.bot, guild_id, &val);
            if let Err(why) = validate_roles(roles.len()) {
                ctx.respond_str(&why, true).await?;
                return Ok(());
            }
            if roles.is_empty() {
                filter.voter_has_all_of = None;
            } else {
                filter.voter_has_all_of = Some(roles.into_iter().collect());
            }
        }
        if let Some(val) = self.voter_has_some_of {
            let roles = parsing::mentions::parse_role_ids(&ctx.bot, guild_id, &val);
            if let Err(why) = validate_roles(roles.len()) {
                ctx.respond_str(&why, true).await?;
                return Ok(());
            }
            if roles.is_empty() {
                filter.voter_has_some_of = None;
            } else {
                filter.voter_has_some_of = Some(roles.into_iter().collect());
            }
        }
        if let Some(val) = self.voter_missing_all_of {
            let roles = parsing::mentions::parse_role_ids(&ctx.bot, guild_id, &val);
            if let Err(why) = validate_roles(roles.len()) {
                ctx.respond_str(&why, true).await?;
                return Ok(());
            }
            if roles.is_empty() {
                filter.voter_missing_all_of = None;
            } else {
                filter.voter_missing_all_of = Some(roles.into_iter().collect());
            }
        }
        if let Some(val) = self.voter_missing_some_of {
            let roles = parsing::mentions::parse_role_ids(&ctx.bot, guild_id, &val);
            if let Err(why) = validate_roles(roles.len()) {
                ctx.respond_str(&why, true).await?;
                return Ok(());
            }
            if roles.is_empty() {
                filter.voter_missing_some_of = None;
            } else {
                filter.voter_missing_some_of = Some(roles.into_iter().collect());
            }
        }
        if let Some(val) = self.older_than {
            if val == "disable" {
                filter.older_than = None;
            } else {
                let delta = match parsing::time_delta::parse_time_delta(&val) {
                    Ok(val) => val,
                    Err(why) => {
                        ctx.respond_str(&why, true).await?;
                        return Ok(());
                    }
                };
                filter.older_than = Some(delta);
            }
        }
        if let Some(val) = self.newer_than {
            if val == "disable" {
                filter.newer_than = None;
            } else {
                let delta = match parsing::time_delta::parse_time_delta(&val) {
                    Ok(val) => val,
                    Err(why) => {
                        ctx.respond_str(&why, true).await?;
                        return Ok(());
                    }
                };
                filter.newer_than = Some(delta);
            }
        }

        if let Err(why) = validation::relative_duration::validate_relative_duration(
            filter.newer_than,
            filter.older_than,
        ) {
            ctx.respond_str(&why.to_bot_str(), true).await?;
            return Ok(());
        }

        filter.update_settings(&ctx.bot.db).await?;

        ctx.respond_str(
            &format!(
                "Updated settings for filter at {} for group '{}'.",
                self.position, group.name
            ),
            false,
        )
        .await?;

        Ok(())
    }
}
