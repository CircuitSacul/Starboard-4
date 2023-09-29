use twilight_interactions::command::{CommandModel, CreateCommand};

use database::pipelines;
use errors::StarboardResult;

use crate::{interactions::context::CommandCtx, utils::id_as_i64::GetI64};

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "move-autostar",
    desc = "Move a lock from one autostar channel to another."
)]
pub struct MoveAutostar {
    /// The autostar channel to move the lock from.
    #[command(rename = "from", autocomplete = true)]
    autostar_from: String,
    /// The autostar channel to move the lock to.
    #[command(rename = "to", autocomplete = true)]
    autostar_to: String,
}

impl MoveAutostar {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let Some(guild_id) = ctx.interaction.guild_id else {
            ctx.respond_str("Please run this command inside a server.", true)
                .await?;
            return Ok(());
        };
        let guild_id_i64 = guild_id.get_i64();

        if let Err(why) = pipelines::locks::autostar::move_lock(
            &ctx.bot.db,
            guild_id_i64,
            &self.autostar_from,
            &self.autostar_to,
        )
        .await?
        {
            ctx.respond_str(&why, true).await?;
            return Ok(());
        };

        ctx.respond_str("Done.", true).await?;

        Ok(())
    }
}
