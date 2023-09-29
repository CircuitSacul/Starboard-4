use twilight_interactions::command::{CommandModel, CreateCommand};

use database::AutoStarChannel;
use errors::StarboardResult;

use crate::{
    core::premium::{is_premium::is_guild_premium, locks::refresh_premium_locks},
    get_guild_id,
    interactions::context::CommandCtx,
    utils::{id_as_i64::GetI64, views::confirm},
};

#[derive(CreateCommand, CommandModel)]
#[command(name = "delete", desc = "Delete an autostar channel.")]
pub struct DeleteAutoStarChannel {
    /// The name of the autostar channel to delete.
    #[command(autocomplete = true)]
    name: String,
}

impl DeleteAutoStarChannel {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let guild_id = get_guild_id!(ctx);

        let mut btn_ctx = match confirm::simple(
            &mut ctx,
            &format!(
                "Are you sure you want to delete the autostar channel '{}'?",
                self.name
            ),
            true,
        )
        .await?
        {
            None => return Ok(()),
            Some(btn_ctx) => btn_ctx,
        };

        let ret = AutoStarChannel::delete(&ctx.bot.db, &self.name, guild_id.get_i64()).await?;
        refresh_premium_locks(
            &ctx.bot,
            guild_id.get_i64(),
            is_guild_premium(&ctx.bot, guild_id.get_i64(), true).await?,
        )
        .await?;
        if ret.is_none() {
            btn_ctx
                .edit_str("No autostar channel with that name was found.", true)
                .await?;
        } else {
            btn_ctx
                .edit_str(&format!("Deleted autostar channel '{}'.", self.name), true)
                .await?;
        }
        Ok(())
    }
}
