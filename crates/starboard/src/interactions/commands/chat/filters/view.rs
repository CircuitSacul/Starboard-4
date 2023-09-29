use std::time::Duration;
use thousands::Separable;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::channel::message::Embed;
use twilight_util::builder::embed::{EmbedBuilder, EmbedFieldBuilder};

use common::constants;
use database::{DbClient, Filter, FilterGroup};
use errors::StarboardResult;

use crate::{
    core::premium::is_premium::is_guild_premium,
    get_guild_id,
    interactions::context::CommandCtx,
    utils::{
        id_as_i64::GetI64,
        views::select_paginator::{SelectPaginatorBuilder, SelectPaginatorPageBuilder},
    },
};

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "view",
    desc = "View filter groups and filters for this server."
)]
pub struct View {
    /// The filter group to view.
    #[command(autocomplete = true)]
    group: Option<String>,
}

impl View {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let guild_id = get_guild_id!(ctx).get_i64();

        let groups = FilterGroup::list_by_guild(&ctx.bot.db, guild_id).await?;

        if groups.is_empty() {
            ctx.respond_str(
                &format!(
                    concat!(
                        "This server does not have any filter groups. ",
                        "Read the [filter docs](<{}>) to get started."
                    ),
                    constants::DOCS_FILTERS
                ),
                true,
            )
            .await?;
            return Ok(());
        }

        let premium = is_guild_premium(&ctx.bot, guild_id, true).await?;

        let bot = ctx.bot.clone();
        let mut paginator = SelectPaginatorBuilder::new(ctx);
        let mut start = 0;
        for (x, group) in groups.into_iter().enumerate() {
            if Some(&group.name) == self.group.as_ref() {
                start = x;
            }
            let embeds = group_embed(&bot.db, &group, premium).await?;
            let page = SelectPaginatorPageBuilder::new(format!("Filter Group '{}'", group.name))
                .embeds(embeds);
            paginator = paginator.add_page(page);
        }

        paginator.current(start).build().run().await
    }
}

async fn group_embed(
    db: &DbClient,
    group: &FilterGroup,
    premium: bool,
) -> StarboardResult<Vec<Embed>> {
    let mut ret = Vec::new();
    let emb = EmbedBuilder::new()
        .color(constants::EMBED_DARK_BG)
        .title(format!("Filter Group '{}'", group.name))
        .description(format!(
            "Read the [filter docs]({}) to get started.",
            constants::DOCS_FILTERS
        ))
        .build();
    ret.push(emb);

    let filters = Filter::list_by_filter(db, group.id).await?;
    if filters.is_empty() {
        let emb = EmbedBuilder::new()
            .color(constants::EMBED_DARK_BG)
            .description("This filter group has no filters.")
            .build();
        ret.push(emb);
    }

    for filter in filters {
        ret.push(filter_embed(filter, premium));
    }

    Ok(ret)
}

fn format_roles(role_ids: &[i64]) -> String {
    if role_ids.is_empty() {
        "None set. This condition always passes.".to_string()
    } else {
        role_ids
            .iter()
            .map(|r| format!("<@&{r}>"))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn format_channels(channel_ids: &[i64]) -> String {
    if channel_ids.is_empty() {
        "None set. This condition always passes.".to_string()
    } else {
        channel_ids
            .iter()
            .map(|c| format!("<#{c}>"))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn filter_embed(filter: Filter, premium: bool) -> Embed {
    let mut default_context = Vec::new();
    let mut message_context = Vec::new();
    let mut vote_context = Vec::new();

    // default context
    if let Some(val) = filter.user_has_all_of {
        let desc = format!("User must have all of these roles:\n{}", format_roles(&val));
        default_context.push(desc);
    }
    if let Some(val) = filter.user_has_some_of {
        let desc = format!(
            "User must have at least one of these roles:\n{}",
            format_roles(&val),
        );
        default_context.push(desc);
    }
    if let Some(val) = filter.user_missing_all_of {
        let desc = format!(
            "User must be missing all of these roles:\n{}",
            format_roles(&val),
        );
        default_context.push(desc);
    }
    if let Some(val) = filter.user_missing_some_of {
        let desc = format!(
            "User must be missing at least one of these roles:\n{}",
            format_roles(&val),
        );
        default_context.push(desc);
    }
    if let Some(val) = filter.user_is_bot {
        let desc = if val {
            "User must be a bot."
        } else {
            "User must not be a bot."
        };
        default_context.push(desc.to_string());
    }

    // message context
    if let Some(val) = filter.in_channel {
        let desc = format!(
            "Message must be in one of these channels:\n{}",
            format_channels(&val)
        );
        message_context.push(desc);
    }
    if let Some(val) = filter.not_in_channel {
        let desc = format!(
            "Message must not be in any of these channels:\n{}",
            format_channels(&val)
        );
        message_context.push(desc);
    }
    if let Some(val) = filter.in_channel_or_sub_channels {
        let desc = format!(
            "Message must be in one of these channels or one of their sub-channels:\n{}",
            format_channels(&val)
        );
        message_context.push(desc);
    }
    if let Some(val) = filter.not_in_channel_or_sub_channels {
        let desc = format!(
            "Message must not be in any of these channels or any of their sub-channels:\n{}",
            format_channels(&val)
        );
        message_context.push(desc);
    }
    if let Some(val) = filter.min_attachments {
        let desc = format!("Message must have at least {val} attachments.");
        message_context.push(desc);
    }
    if let Some(val) = filter.max_attachments {
        let desc = format!("Message cannot have more than {val} attachments.");
        message_context.push(desc);
    }
    if let Some(val) = filter.min_length {
        let desc = format!(
            "Message must be at least {} characters long.",
            val.separate_with_commas()
        );
        message_context.push(desc);
    }
    if let Some(val) = filter.max_length {
        let desc = format!(
            "Message cannot be longer than {} characters.",
            val.separate_with_commas()
        );
        message_context.push(desc);
    }
    if let Some(val) = filter.matches {
        let mut desc = format!("Message must match the following regex:\n```re\n{val}\n```");
        if !premium {
            desc.push_str(
                "\n:warning: This setting is ignored because this server doesn't have premium.",
            );
        }
        message_context.push(desc);
    }
    if let Some(val) = filter.not_matches {
        let mut desc = format!("Message must not match the following regex:\n```re\n{val}\n```");
        if !premium {
            desc.push_str(
                "\n:warning: This setting is ignored because this server doesn't have premium.",
            );
        }
        message_context.push(desc);
    }

    // voter context
    if let Some(val) = filter.voter_has_all_of {
        let desc = format!(
            "Voter must have all of these roles:\n{}",
            format_roles(&val)
        );
        vote_context.push(desc);
    }
    if let Some(val) = filter.voter_has_some_of {
        let desc = format!(
            "Voter must have at least one of these roles:\n{}",
            format_roles(&val),
        );
        vote_context.push(desc);
    }
    if let Some(val) = filter.voter_missing_all_of {
        let desc = format!(
            "Voter must be missing all of these roles:\n{}",
            format_roles(&val),
        );
        vote_context.push(desc);
    }
    if let Some(val) = filter.voter_missing_some_of {
        let desc = format!(
            "Voter must be missing at least one of these roles:\n{}",
            format_roles(&val),
        );
        vote_context.push(desc);
    }
    if let Some(val) = filter.older_than {
        let desc = format!(
            "Message must be older than {}.",
            humantime::format_duration(Duration::from_secs(val as u64)),
        );
        vote_context.push(desc);
    }
    if let Some(val) = filter.newer_than {
        let desc = format!(
            "Message must be newer than {}.",
            humantime::format_duration(Duration::from_secs(val as u64)),
        );
        vote_context.push(desc);
    }

    let mut desc = String::new();
    desc.push_str(&format!(
        "instant-pass: {}\ninstant-fail: {}",
        filter.instant_pass, filter.instant_fail
    ));

    let mut emb = EmbedBuilder::new().color(constants::EMBED_DARK_BG);

    let mut has_conditions = false;

    if !default_context.is_empty() {
        has_conditions = true;
        emb = emb.field(EmbedFieldBuilder::new(
            "Default Context",
            default_context.join("\n\n"),
        ));
    }
    if !message_context.is_empty() {
        has_conditions = true;
        emb = emb.field(EmbedFieldBuilder::new(
            "Message Context",
            message_context.join("\n\n"),
        ));
    }
    if !vote_context.is_empty() {
        has_conditions = true;
        emb = emb.field(EmbedFieldBuilder::new(
            "Vote Context",
            vote_context.join("\n\n"),
        ));
    }

    if !has_conditions {
        desc.push_str("\n\nThis filter has no conditions, so it always passes.");
    }

    emb = emb
        .description(desc)
        .title(format!("Filter {}", filter.position));

    emb.build()
}
