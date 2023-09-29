use std::fmt::Write;

use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{
    channel::message::Embed,
    id::{marker::GuildMarker, Id},
};
use twilight_util::builder::embed::EmbedFieldBuilder;

use database::Starboard;
use errors::StarboardResult;

use crate::{
    client::bot::StarboardBot,
    core::starboard::config::StarboardConfig,
    get_guild_id,
    interactions::{commands::format_settings::format_settings, context::CommandCtx},
    utils::{
        embed,
        id_as_i64::GetI64,
        views::select_paginator::{SelectPaginatorBuilder, SelectPaginatorPageBuilder},
    },
};

#[derive(CreateCommand, CommandModel)]
#[command(name = "view", desc = "View your starboards.")]
pub struct ViewStarboard {
    /// The name of the starboard to view. Leave blank to show all.
    #[command(autocomplete = true)]
    name: Option<String>,
}

impl ViewStarboard {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let guild_id = get_guild_id!(ctx);
        let bot = ctx.bot.clone();

        let starboards = Starboard::list_by_guild(&ctx.bot.db, guild_id.get_i64()).await?;
        if starboards.is_empty() {
            ctx.respond_str("This server has no starboards.", true)
                .await?;
            return Ok(());
        }

        let mut paginator = SelectPaginatorBuilder::new(ctx);

        let mut current = None;
        for (idx, sb) in starboards.into_iter().enumerate() {
            if self.name.as_ref() == Some(&sb.name) {
                current = Some(idx);
            }

            let mut label = sb.name.clone();
            if sb.premium_locked {
                label.push_str(" (premium-locked)");
            }

            let page = SelectPaginatorPageBuilder::new(label.clone())
                .add_embed(starboard_embed(&bot, guild_id, sb).await?);
            paginator = paginator.add_page(page);
        }

        if let Some(current) = current {
            paginator = paginator.current(current);
        }

        paginator.build().run().await?;

        Ok(())
    }
}

async fn starboard_embed(
    bot: &StarboardBot,
    guild_id: Id<GuildMarker>,
    starboard: Starboard,
) -> StarboardResult<Embed> {
    let config = StarboardConfig::new(starboard, &[], vec![])?;
    let pretty = format_settings(bot, guild_id, &config).await?;

    let mut desc = String::new();
    if config.starboard.premium_locked {
        desc.push_str(concat!(
            "This starboard is locked because it exceeds the non-premium limit.\n\n"
        ));
    }
    write!(
        desc,
        "This starboard is in <#{}>.",
        config.starboard.channel_id
    )
    .unwrap();

    let embed = embed::build()
        .title(format!("Starboard '{}'", &config.starboard.name))
        .description(desc)
        .field(
            EmbedFieldBuilder::new("Requirements", pretty.requirements)
                .inline()
                .build(),
        )
        .field(
            EmbedFieldBuilder::new("Behaviour", pretty.behavior)
                .inline()
                .build(),
        )
        .field(
            EmbedFieldBuilder::new("Style", pretty.style)
                .inline()
                .build(),
        )
        .field(
            EmbedFieldBuilder::new("Embed Style", pretty.embed)
                .inline()
                .build(),
        )
        .field(EmbedFieldBuilder::new("Regex Matching", pretty.regex).build())
        .field(EmbedFieldBuilder::new("Filters", pretty.filters))
        .build();

    Ok(embed)
}
