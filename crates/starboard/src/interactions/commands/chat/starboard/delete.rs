use twilight_interactions::command::{CommandModel, CreateCommand};

use database::Starboard;
use errors::StarboardResult;

use crate::{
    core::premium::{is_premium::is_guild_premium, locks::refresh_premium_locks},
    get_guild_id,
    interactions::context::CommandCtx,
    utils::{id_as_i64::GetI64, views::confirm},
};

#[derive(CreateCommand, CommandModel)]
#[command(name = "delete", desc = "Delete a starboard.")]
pub struct DeleteStarboard {
    /// The name of the starboard to delete.
    #[command(autocomplete = true)]
    name: String,
}

impl DeleteStarboard {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let guild_id = get_guild_id!(ctx);
        let guild_id_i64 = guild_id.get_i64();

        let mut btn_ctx = match confirm::simple(
            &mut ctx,
            &format!(
                "Are you sure you want to delete the starboard '{}'?",
                self.name
            ),
            true,
        )
        .await?
        {
            None => return Ok(()),
            Some(btn_ctx) => btn_ctx,
        };

        let ret = Starboard::delete(&ctx.bot.db, &self.name, guild_id_i64).await?;
        refresh_premium_locks(
            &ctx.bot,
            guild_id.get_i64(),
            is_guild_premium(&ctx.bot, guild_id.get_i64(), true).await?,
        )
        .await?;
        if ret.is_none() {
            btn_ctx
                .edit_str("No starboard with that name was found.", true)
                .await?;
        } else {
            ctx.bot.cache.guild_vote_emojis.remove(&guild_id_i64);
            btn_ctx
                .edit_str(&format!("Deleted starboard '{}'.", self.name), true)
                .await?;
        }
        Ok(())
    }
}
