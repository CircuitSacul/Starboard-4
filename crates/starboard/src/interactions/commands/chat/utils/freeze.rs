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
    name = "freeze",
    desc = "Freeze a message, so that it cannot receive or lose any votes."
)]
pub struct Freeze {
    /// Link to the message to freeze.
    message: String,
}

impl Freeze {
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

        if orig.guild_id != get_guild_id!(ctx) {
            ctx.respond_str("That message belongs to a different server.", true)
                .await?;
            return Ok(());
        }

        DbMessage::set_freeze(&ctx.bot.db, orig.message_id, true)
            .await?
            .unwrap();
        ctx.respond_str("Message frozen.", true).await?;

        let is_premium = is_guild_premium(&ctx.bot, guild_id, true).await?;
        let mut refresh = RefreshMessage::new(ctx.bot, orig.message_id.into_id(), is_premium);
        refresh.refresh(true).await?;

        Ok(())
    }
}

#[derive(CommandModel, CreateCommand)]
#[command(name = "unfreeze", desc = "Unfreeze a message.")]
pub struct UnFreeze {
    /// Link to the message to unfreeze.
    message: String,
}

impl UnFreeze {
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

        if orig.guild_id != get_guild_id!(ctx) {
            ctx.respond_str("That message belongs to a different server.", true)
                .await?;
            return Ok(());
        }

        DbMessage::set_freeze(&ctx.bot.db, orig.message_id, false)
            .await?
            .unwrap();
        ctx.respond_str("Message unfrozen.", true).await?;

        let is_premium = is_guild_premium(&ctx.bot, guild_id, true).await?;
        let mut refresh = RefreshMessage::new(ctx.bot, orig.message_id.into_id(), is_premium);
        refresh.refresh(true).await?;

        Ok(())
    }
}
