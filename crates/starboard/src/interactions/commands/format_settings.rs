//! Utility for formatting starboard/override settings

use std::{
    fmt::{Debug, Write},
    time::Duration,
};

use humantime::format_duration;
use twilight_model::id::{marker::GuildMarker, Id};

use common::constants;
use database::{ExclusiveGroup, FilterGroup, StarboardFilterGroup};
use errors::StarboardResult;

use crate::{
    client::bot::StarboardBot,
    core::{
        emoji::{EmojiCommon, SimpleEmoji},
        starboard::config::StarboardConfig,
    },
};

async fn filters_field(bot: &StarboardBot, starboard_id: i32) -> StarboardResult<String> {
    let filter_groups = StarboardFilterGroup::list_by_starboard(&bot.db, starboard_id).await?;
    let filter_group_ids = filter_groups.into_iter().map(|g| g.filter_group_id);
    let mut filter_groups = Vec::new();
    for group_id in filter_group_ids {
        let filter_group = FilterGroup::get(&bot.db, group_id).await?;
        filter_groups.push(filter_group.name);
    }

    let mut filter_groups = filter_groups.join(", ");
    if filter_groups.is_empty() {
        filter_groups = "No filters set.".to_string();
    }

    Ok(format!(
        concat!(
            "These are the filters that must pass for a message to be starred:\n\n",
            "{}\n\n",
            "You can view filters using `/filters view`, and you can change which ",
            "ones apply using `/starboards filters [add|remove]`."
        ),
        filter_groups,
    ))
}

pub async fn format_settings(
    bot: &StarboardBot,
    guild_id: Id<GuildMarker>,
    config: &StarboardConfig,
) -> StarboardResult<FormattedStarboardSettings> {
    let ov_values = config
        .overrides
        .get(0)
        .map(|ov| ov.get_overrides().unwrap());

    macro_rules! settings {
        ($($setting: ident, $pretty_name: expr, $value: expr;)*) => {{
            let mut final_result = String::new();
            $(
                let is_bold = ov_values.as_ref().map_or(false, |ov| {
                    ov.$setting.is_some()
                });

                let setting_name = if is_bold {
                    format!("**{}**", $pretty_name)
                } else {
                    $pretty_name.to_string()
                };

                writeln!(final_result, "{}: {}", setting_name, $value).unwrap();
            )*
            final_result
        }};
    }

    let res = &config.resolved;

    let display_emoji = SimpleEmoji::from_stored(
        config
            .resolved
            .display_emoji
            .clone()
            .unwrap_or_else(|| "none".to_string()),
    )
    .into_readable(bot, guild_id);

    let upvote_emojis = Vec::from_stored(res.upvote_emojis.clone()).into_readable(bot, guild_id);
    let downvote_emojis =
        Vec::from_stored(res.downvote_emojis.clone()).into_readable(bot, guild_id);

    let older_than = if res.older_than <= 0 {
        "disabled".to_string()
    } else {
        format_duration(Duration::from_secs(res.older_than as u64)).to_string()
    };
    let newer_than = if res.newer_than <= 0 {
        "disabled".to_string()
    } else {
        format_duration(Duration::from_secs(res.newer_than as u64)).to_string()
    };

    let owner: String;
    let exclusive_group = {
        match res.exclusive_group {
            None => "*no group*",
            Some(group) => {
                let group = ExclusiveGroup::get(&bot.db, group).await?;
                match group {
                    Some(group) => {
                        owner = group.name;
                        &owner
                    }
                    None => "*no group*",
                }
            }
        }
    };

    let cooldown = {
        let is_bold = ov_values.as_ref().map_or(false, |ov| {
            ov.cooldown_count.is_some() || ov.cooldown_period.is_some()
        });

        let setting_name = if is_bold { "**cooldown**" } else { "cooldown" };

        format!(
            "{}: {} reactions per {} seconds\n",
            setting_name, res.cooldown_count, res.cooldown_period
        )
    };
    let on_delete = match res.on_delete {
        0 => "Refresh",
        1 => "Ignore",
        2 => "Trash All",
        3 => "Freeze All",
        _ => "Invalid",
    };
    let go_to_message = match res.go_to_message {
        0 => "None",
        1 => "Link",
        2 => "Button",
        3 => "Mention",
        _ => "Invalid",
    };

    let behavior = settings!(
        enabled, "enabled", res.enabled;
        autoreact_upvote, "autoreact-upvote", res.autoreact_upvote;
        autoreact_downvote, "autoreact-downvote", res.autoreact_downvote;
        remove_invalid_reactions, "remove-invalid-reactions", res.remove_invalid_reactions;
        link_deletes, "link-deletes", res.link_deletes;
        link_edits, "link-edits", res.link_edits;
        on_delete, "on-delete", on_delete;
        cooldown_enabled, "cooldown-enabled", res.cooldown_enabled;
    ) + &cooldown
        + &format!("xp-multiplier: {}\n", res.xp_multiplier)
        + &format!("private: {}\n", res.private)
        + &settings!(
            exclusive_group, "exclusive-group", exclusive_group;
            exclusive_group_priority, "exclusive-group-priority", res.exclusive_group_priority;
        );

    let must_match = if let Some(re) = &res.matches {
        format!("```rs\n{re}\n```")
    } else {
        "Nothing set.".to_string()
    };
    let must_not_match = if let Some(re) = &res.not_matches {
        format!("```rs\n{re}\n```")
    } else {
        "Nothing set.".to_string()
    };

    let resync = if ov_values.is_some() {
        concat!(
            "\n\nRe-sync these settings with the parent starboard by using `matches` and ",
            "`not-matches` in the reset command.",
        )
    } else {
        ""
    };

    let required = match res.required {
        Some(req) => req.to_string(),
        None => "unset".to_string(),
    };
    let required_remove = match res.required_remove {
        Some(req) => req.to_string(),
        None => "unset".to_string(),
    };

    let settings = FormattedStarboardSettings {
        style: settings!(
            display_emoji, "display-emoji", display_emoji;
            ping_author, "ping-author", res.ping_author;
            use_server_profile, "use-server-profile", res.use_server_profile;
            extra_embeds, "extra-embeds", res.extra_embeds;
            go_to_message, "go-to-message", go_to_message;
            use_webhook, "use-webhook", res.use_webhook;
        ),
        embed: settings!(
            color, "color", &format!(
                "#{:X}", res.color.unwrap_or(constants::BOT_COLOR as i32)
            );
            attachments_list, "attachments-list", res.attachments_list;
            replied_to, "replied-to", res.replied_to;
        ),
        requirements: settings!(
            required, "required", required;
            required_remove, "required-remove", required_remove;
            upvote_emojis, "upvote-emojis", upvote_emojis;
            downvote_emojis, "downvote-emojis", downvote_emojis;
            self_vote, "self-vote", res.self_vote;
            allow_bots, "allow-bots", res.allow_bots;
            require_image, "require-image", res.require_image;
            older_than, "older-than", older_than;
            newer_than, "newer-than", newer_than;
        ),
        behavior,
        regex: format!(
            concat!(
            "These settings are premium-only. You can input a simple phrase to match on, or you ",
            "can use regex for more advanced filtering. See [rustexp](https://rustexp.lpil.uk) ",
            "for more info on regex.{}\n\nMessages **must** match:\n{}\n",
            "Messages **must not** match:\n{}",
        ),
            resync, must_match, must_not_match
        ),
        filters: filters_field(bot, config.starboard.id).await?,
    };

    Ok(settings)
}

#[derive(Debug)]
pub struct FormattedStarboardSettings {
    pub style: String,
    pub embed: String,
    pub requirements: String,
    pub behavior: String,
    pub regex: String,
    pub filters: String,
}
