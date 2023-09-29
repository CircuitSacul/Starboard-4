use twilight_interactions::command::{CommandModel, CreateCommand};

use database::pipelines;
use errors::StarboardResult;

use crate::{interactions::context::CommandCtx, utils::id_as_i64::GetI64};

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "move-starboard",
    desc = "Move a lock from one starboard to another."
)]
pub struct MoveStarboard {
    /// The starboard to move the lock from.
    #[command(rename = "from", autocomplete = true)]
    starboard_from: String,
    /// The starboard to move the lock to.
    #[command(rename = "to", autocomplete = true)]
    starboard_to: String,
}

impl MoveStarboard {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let Some(guild_id) = ctx.interaction.guild_id else {
            ctx.respond_str("Please run this command inside a server.", true)
                .await?;
            return Ok(());
        };
        let guild_id = guild_id.get_i64();

        if let Err(why) = pipelines::locks::starboard::move_lock(
            &ctx.bot.db,
            guild_id,
            &self.starboard_from,
            &self.starboard_to,
        )
        .await?
        {
            ctx.respond_str(&why, true).await?;
            return Ok(());
        }

        ctx.respond_str("Done.", true).await?;

        Ok(())
    }
}
