use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{
    channel::message::Embed,
    id::{marker::GuildMarker, Id},
};
use twilight_util::builder::embed::EmbedFieldBuilder;

use database::{Starboard, StarboardOverride};
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
#[command(name = "view", desc = "View your overrides.")]
pub struct ViewOverride {
    /// The name of the override to view. Leave blank to show all.
    #[command(autocomplete = true)]
    name: Option<String>,
}

impl ViewOverride {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let guild_id = get_guild_id!(ctx);
        let guild_id_i64 = guild_id.get_i64();
        let bot = ctx.bot.clone();

        let overrides = StarboardOverride::list_by_guild(&ctx.bot.db, guild_id_i64).await?;
        if overrides.is_empty() {
            ctx.respond_str("This server has no overrides.", true)
                .await?;
            return Ok(());
        }

        let mut paginator = SelectPaginatorBuilder::new(ctx);
        let mut current = 0;

        for (idx, ov) in overrides.into_iter().enumerate() {
            if self.name.as_ref() == Some(&ov.name) {
                current = idx;
            }

            let sb = Starboard::get(&bot.db, ov.starboard_id).await?.unwrap();

            let label = ov.name.clone();
            let description = format!(
                "{} overwritten settings for starboard '{}' in {} channel(s)",
                ov.overrides.as_object().unwrap().len(),
                sb.name,
                ov.channel_ids.len(),
            );

            let page = SelectPaginatorPageBuilder::new(label)
                .description(description)
                .add_embed(override_embed(&bot, guild_id, ov).await?);
            paginator = paginator.add_page(page);
        }

        paginator.current(current).build().run().await?;

        Ok(())
    }
}

async fn override_embed(
    bot: &StarboardBot,
    guild_id: Id<GuildMarker>,
    ov: StarboardOverride,
) -> StarboardResult<Embed> {
    let name = ov.name.clone();
    let sb = Starboard::get(&bot.db, ov.starboard_id).await?.unwrap();

    let channels: Vec<_> = ov.channel_ids.iter().map(|id| format!("<#{id}>")).collect();
    let channels = channels.join(", ");
    let config = StarboardConfig::new(sb, &[], vec![ov])?;
    let pretty = format_settings(bot, guild_id, &config).await?;

    let embed = embed::build()
        .title(format!("Override '{name}'"))
        .description(format!(
            concat!(
                "This override belongs to the starboard '{}'.\n\n",
                "This override applies to the following channels: {}",
            ),
            &config.starboard.name, channels,
        ))
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
