use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::channel::message::MessageFlags;
use twilight_util::builder::embed::EmbedFieldBuilder;

use database::{DbMessage, Starboard, StarboardMessage, StarboardOverride, Vote};
use errors::StarboardResult;

use crate::{
    concat_format,
    core::starboard::config::StarboardConfig,
    get_guild_id,
    interactions::context::CommandCtx,
    utils::{
        embed,
        id_as_i64::GetI64,
        into_id::IntoId,
        message_link::{fmt_message_link, parse_message_link},
    },
};

use super::INVALID_MESSAGE_ERR;

#[derive(CommandModel, CreateCommand)]
#[command(name = "info", desc = "Get info for a message.")]
pub struct Info {
    /// Link to the message to get info for.
    message: String,
}

impl Info {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let guild_id = get_guild_id!(ctx);
        let guild_id_i64 = guild_id.get_i64();

        let Some((_channel_id, message_id)) = parse_message_link(&self.message) else {
            ctx.respond_str("Invalid message link.", true).await?;
            return Ok(());
        };

        let Some(sql_msg) = DbMessage::get_original(&ctx.bot.db, message_id).await? else {
            ctx.respond_str(INVALID_MESSAGE_ERR, true).await?;
            return Ok(());
        };

        if sql_msg.guild_id != guild_id {
            ctx.respond_str("That message belongs to a different server.", true)
                .await?;
            return Ok(());
        }

        let link = fmt_message_link(guild_id, sql_msg.channel_id, sql_msg.message_id);
        let mut emb = embed::build().description(concat_format!(
            "original: `{}` [jump]({})" <- sql_msg.message_id, link;
            "\nchannel: `{0}` <#{0}>" <- sql_msg.channel_id;
            "\nauthor: `{0}` <@{0}>" <- sql_msg.author_id;
            "\n\ntrashed: {:?}" <- sql_msg.trashed;
            "\nfrozen: {:?}" <- sql_msg.frozen;
        ));

        for starboard in Starboard::list_by_guild(&ctx.bot.db, guild_id_i64).await? {
            let points = Vote::count(&ctx.bot.db, sql_msg.message_id, starboard.id).await?;

            let channel_ids = ctx
                .bot
                .cache
                .qualified_channel_ids(&ctx.bot, guild_id, sql_msg.channel_id.into_id())
                .await?;
            let channel_ids = channel_ids
                .into_iter()
                .map(|id| id.get_i64())
                .collect::<Vec<_>>();
            let overrides = StarboardOverride::list_by_starboard_and_channels(
                &ctx.bot.db,
                starboard.id,
                &channel_ids,
            )
            .await?;

            let config = StarboardConfig::new(starboard, &channel_ids, overrides)?;

            let sb_msg = StarboardMessage::get_by_starboard(
                &ctx.bot.db,
                sql_msg.message_id,
                config.starboard.id,
            )
            .await?;

            let link = sb_msg
                .map(|m| {
                    let channel_id = if ctx
                        .bot
                        .cache
                        .is_channel_forum(guild_id, config.starboard.channel_id.into_id())
                    {
                        m.starboard_message_id
                    } else {
                        config.starboard.channel_id
                    };

                    fmt_message_link(guild_id, channel_id, m.starboard_message_id)
                })
                .map(|link| format!("[jump]({link})"))
                .unwrap_or_else(|| "Not on starboard.".to_string());
            emb = emb.field(
                EmbedFieldBuilder::new(
                    config.starboard.name,
                    concat_format!(
                        "{}\n" <- link;
                        "points: {}/{}\n" <- points,
                            config.resolved.required
                                .map(|v| v.to_string())
                                .unwrap_or_else(|| "unset".to_string());
                        "forced: {}" <- sql_msg.forced_to.contains(&config.starboard.id);
                    ),
                )
                .build(),
            );
        }

        ctx.respond(
            ctx.build_resp()
                .embeds([emb.build()])
                .flags(MessageFlags::EPHEMERAL)
                .build(),
        )
        .await?;

        Ok(())
    }
}
