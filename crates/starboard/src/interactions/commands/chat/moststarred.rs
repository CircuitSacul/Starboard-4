use std::sync::Arc;

use futures::{stream::BoxStream, TryStreamExt};
use sqlx::{Postgres, QueryBuilder};
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{
    application::interaction::application_command::InteractionChannel,
    channel::message::{
        component::{ActionRow, Button, ButtonStyle},
        Component,
    },
    id::{marker::MessageMarker, Id},
    user::User,
};

use database::{DbMessage, Starboard, StarboardMessage};
use errors::StarboardResult;

use crate::{
    core::embedder::{builder::BuiltStarboardEmbed, Embedder},
    get_guild_id,
    interactions::context::{CommandCtx, ComponentCtx},
    utils::{id_as_i64::GetI64, views::wait_for::wait_for_component},
};

use super::random::{get_config, get_embedder, get_post_query};

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "moststarred",
    desc = "Shows the most starred messages in this server.",
    dm_permission = false
)]
pub struct MostStarred {
    /// The starboard to show the most starred messages for.
    #[command(autocomplete = true)]
    starboard: String,

    /// Only show messages with at least this many points.
    #[command(rename = "min-points", max_value = 32767, min_value = -32767)]
    min_points: Option<i64>,
    /// Only show messages with at most this many points.
    #[command(rename = "max-points", max_value = 32767, min_value = -32767)]
    max_points: Option<i64>,
    /// Only show messages that were sent in this channel.
    channel: Option<InteractionChannel>,
    /// Only show messages sent by this user.
    author: Option<User>,
    /// Whether to allow messages from NSFW starboards.
    #[command(rename = "allow-nsfw")]
    allow_nsfw: Option<bool>,
}

impl MostStarred {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let guild_id = get_guild_id!(ctx);
        let guild_id_i64 = guild_id.get_i64();

        let Some(sb) = Starboard::get_by_name(&ctx.bot.db, &self.starboard, guild_id_i64).await?
        else {
            ctx.respond_str(
                &format!("Starboard '{}' does not exist.", self.starboard),
                true,
            )
            .await?;
            return Ok(());
        };

        if sb.settings.private {
            ctx.respond_str("That starboard is private.", true).await?;
            return Ok(());
        }

        let allow_nsfw = self.allow_nsfw.unwrap_or(false);

        if allow_nsfw {
            let nsfw = ctx
                .bot
                .cache
                .fog_channel_nsfw(
                    &ctx.bot,
                    guild_id,
                    ctx.interaction.channel.as_ref().unwrap().id,
                )
                .await?;
            if !nsfw.unwrap() {
                ctx.respond_str(
                    "This channel isn't NSFW, so you can't allow NSFW messages.",
                    true,
                )
                .await?;
                return Ok(());
            }
        }

        let mut query = get_post_query(
            sb.id,
            allow_nsfw,
            self.channel.map(|ch| ch.id.get_i64()),
            self.author.map(|user| user.id.get_i64()),
            self.min_points.map(|v| v as i16),
            self.max_points.map(|v| v as i16),
        );
        query.push(" ORDER BY last_known_point_count DESC");

        scrolling_paginator(ctx, query, sb).await?;

        Ok(())
    }
}

fn components(
    current_page: usize,
    last_page: Option<usize>,
    done: bool,
    gtm_btn: Option<Button>,
) -> Vec<Component> {
    let buttons = vec![
        Component::Button(Button {
            custom_id: Some("moststarred_scroller::back".to_string()),
            disabled: done || current_page == 1,
            emoji: None,
            label: Some("Back".to_string()),
            style: ButtonStyle::Secondary,
            url: None,
        }),
        Component::Button(Button {
            custom_id: Some("moststarred_scroller::page_number".to_string()),
            disabled: true,
            emoji: None,
            label: Some(current_page.to_string()),
            style: ButtonStyle::Secondary,
            url: None,
        }),
        Component::Button(Button {
            custom_id: Some("moststarred_scroller::next".to_string()),
            disabled: done || Some(current_page) == last_page,
            emoji: None,
            label: Some("Next".to_string()),
            style: ButtonStyle::Secondary,
            url: None,
        }),
    ];

    let mut action_rows = vec![Component::ActionRow(ActionRow {
        components: buttons,
    })];

    if let Some(btn) = gtm_btn {
        action_rows.push(Component::ActionRow(ActionRow {
            components: vec![Component::Button(btn)],
        }));
    }

    action_rows
}

async fn scrolling_paginator(
    mut ctx: CommandCtx,
    mut query: QueryBuilder<'_, Postgres>,
    starboard: Starboard,
) -> StarboardResult<()> {
    let user_id = ctx.interaction.author_id().unwrap();

    let bot = ctx.bot.clone();
    let mut messages: BoxStream<'_, sqlx::Result<StarboardMessage>> =
        query.build_query_as().fetch(&bot.db.pool);

    let mut btn_ctx: Option<ComponentCtx> = None;
    let mut message_id: Option<Id<MessageMarker>> = None;
    let mut current_page: usize = 1;
    let mut last_page: Option<usize> = None;
    let mut gtm_btn: Option<Button>;

    let mut cache: Vec<Embedder> = Vec::new();

    loop {
        if current_page > cache.len() {
            if let Some(next_sb_message) = messages.try_next().await? {
                let orig_msg = DbMessage::get(&ctx.bot.db, next_sb_message.message_id)
                    .await?
                    .unwrap();
                let config = get_config(&ctx.bot, starboard.clone(), orig_msg.channel_id).await?;
                let config = Arc::new(config);
                let embedder =
                    get_embedder(ctx.bot.clone(), config, orig_msg, next_sb_message).await?;

                let Some(embedder) = embedder else {
                    continue;
                };
                cache.push(embedder);
            } else {
                if current_page == 1 {
                    ctx.respond_str("Nothing to show.", true).await?;
                    return Ok(());
                }

                current_page -= 1;
                last_page = Some(current_page);
            };
        }

        // built message
        let embedder = &cache[current_page - 1];

        let built = embedder.build(false, false).await?;
        let BuiltStarboardEmbed::Full(built) = built else {
            unreachable!("didn't get full embed");
        };

        // respond
        gtm_btn = BuiltStarboardEmbed::build_go_to_message_button(embedder);
        let data = ctx
            .build_resp()
            .components(components(current_page, last_page, false, gtm_btn.clone()))
            .embeds(built.embeds)
            .content(built.top_content)
            .build();

        if let Some(btn_ctx) = &mut btn_ctx {
            btn_ctx.edit(data).await?;
        } else {
            let resp = ctx.respond(data).await?.model().await?;
            message_id = Some(resp.id);
        }

        // wait for component interaction
        btn_ctx = wait_for_component(
            ctx.bot.clone(),
            &["moststarred_scroller::next", "moststarred_scroller::back"],
            message_id.unwrap(),
            user_id,
            60 * 5,
        )
        .await;

        if let Some(btn_ctx) = &btn_ctx {
            match &*btn_ctx.data.custom_id {
                "moststarred_scroller::next" => current_page += 1,
                "moststarred_scroller::back" => current_page -= 1,
                _ => unreachable!(),
            }
        } else {
            break;
        }
    }

    if let Some(btn_ctx) = &mut btn_ctx {
        btn_ctx
            .edit(
                btn_ctx
                    .build_resp()
                    .components(components(current_page, last_page, true, gtm_btn))
                    .build(),
            )
            .await?;
    } else if message_id.is_some() {
        let i = ctx.bot.interaction_client().await;
        i.update_response(&ctx.interaction.token)
            .components(Some(&components(current_page, last_page, true, gtm_btn)))?
            .await?;
    } else {
        ctx.respond_str("Nothing to show.", true).await?;
    }

    Ok(())
}
