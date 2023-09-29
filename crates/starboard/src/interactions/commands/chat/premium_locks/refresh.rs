use twilight_interactions::command::{CommandModel, CreateCommand};

use errors::StarboardResult;

use crate::{
    core::premium::{is_premium::is_guild_premium, locks::refresh_premium_locks},
    interactions::context::CommandCtx,
    utils::id_as_i64::GetI64,
};

#[derive(CommandModel, CreateCommand)]
#[command(name = "refresh", desc = "Refreshes premium locks.")]
pub struct Refresh;

impl Refresh {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let Some(guild_id) = ctx.interaction.guild_id else {
            ctx.respond_str("Please run this command inside a server.", true)
                .await?;
            return Ok(());
        };
        let guild_id = guild_id.get_i64();

        refresh_premium_locks(
            &ctx.bot,
            guild_id,
            is_guild_premium(&ctx.bot, guild_id, true).await?,
        )
        .await?;

        ctx.respond_str("Refreshed locks.", true).await?;

        Ok(())
    }
}
