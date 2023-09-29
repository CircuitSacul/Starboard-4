use twilight_interactions::command::{CommandModel, CreateCommand};

use database::ExclusiveGroup;
use errors::StarboardResult;

use crate::{get_guild_id, interactions::context::CommandCtx, utils::id_as_i64::GetI64};

#[derive(CommandModel, CreateCommand)]
#[command(name = "delete", desc = "Delete an exclusive group.")]
pub struct Delete {
    /// The exclusive group to delete.
    #[command(autocomplete = true)]
    name: String,
}

impl Delete {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let guild_id = get_guild_id!(ctx).get_i64();

        let ret = ExclusiveGroup::delete(&ctx.bot.db, &self.name, guild_id).await?;
        if ret.is_none() {
            ctx.respond_str(
                &format!("Exclusive group '{}' does not exist.", self.name),
                true,
            )
            .await?;
        } else {
            ctx.respond_str(&format!("Deleted exclusive group '{}'.", self.name), false)
                .await?;
        }

        Ok(())
    }
}
