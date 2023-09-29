use twilight_interactions::command::{CommandModel, CreateCommand};

use database::DbMessage;
use errors::StarboardResult;

use crate::{
    core::{premium::is_premium::is_guild_premium, starboard::handle::RefreshMessage},
    get_guild_id,
    interactions::context::CommandCtx,
    utils::{id_as_i64::GetI64, into_id::IntoId, message_link::parse_message_link},
};

use super::INVALID_MESSAGE_ERR;

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "trash",
    desc = "Trash a message so that it is removed from all starboards."
)]
pub struct Trash {
    /// Link to the message to trash.
    message: String,

    /// Reason for trashing the message.
    reason: Option<String>,
}

impl Trash {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let guild_id = get_guild_id!(ctx).get_i64();

        let Some((_channel_id, message_id)) = parse_message_link(&self.message) else {
            ctx.respond_str("Invalid message link.", true).await?;
            return Ok(());
        };

        let Some(orig) = DbMessage::get_original(&ctx.bot.db, message_id).await? else {
            ctx.respond_str(INVALID_MESSAGE_ERR, true).await?;
            return Ok(());
        };

        if orig.guild_id != guild_id {
            ctx.respond_str("That message belongs to a different server.", true)
                .await?;
            return Ok(());
        }

        DbMessage::set_trashed(&ctx.bot.db, orig.message_id, true, self.reason.as_deref()).await?;
        ctx.respond_str("Message trashed.", true).await?;

        let is_premium = is_guild_premium(&ctx.bot, guild_id, true).await?;
        RefreshMessage::new(ctx.bot, orig.message_id.into_id(), is_premium)
            .refresh(true)
            .await?;

        Ok(())
    }
}

#[derive(CommandModel, CreateCommand)]
#[command(name = "untrash", desc = "Untrashes a message.")]
pub struct UnTrash {
    /// Link to the message to untrash.
    message: String,
}

impl UnTrash {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let guild_id = get_guild_id!(ctx).get_i64();

        let Some((_channel_id, message_id)) = parse_message_link(&self.message) else {
            ctx.respond_str("Invalid message link.", true).await?;
            return Ok(());
        };

        let Some(orig) = DbMessage::get_original(&ctx.bot.db, message_id).await? else {
            ctx.respond_str(INVALID_MESSAGE_ERR, true).await?;
            return Ok(());
        };

        if orig.guild_id != guild_id {
            ctx.respond_str("That message belongs to a different server.", true)
                .await?;
            return Ok(());
        }

        DbMessage::set_trashed(&ctx.bot.db, orig.message_id, false, None).await?;
        ctx.respond_str("Message untrashed.", true).await?;
        let is_premium = is_guild_premium(&ctx.bot, guild_id, true).await?;
        RefreshMessage::new(ctx.bot, orig.message_id.into_id(), is_premium)
            .refresh(true)
            .await?;

        Ok(())
    }
}
