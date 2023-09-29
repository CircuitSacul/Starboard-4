use twilight_interactions::command::{CommandModel, CreateCommand};

use database::{FilterGroup, Starboard, StarboardFilterGroup};
use errors::StarboardResult;

use crate::{get_guild_id, interactions::context::CommandCtx, utils::id_as_i64::GetI64};

#[derive(CommandModel, CreateCommand)]
#[command(name = "remove", desc = "Remove a filter group from a starboard.")]
pub struct Remove {
    /// The starboard to remove the filter group from.
    #[command(autocomplete = true)]
    starboard: String,
    /// The filter group to remove from the starboard.
    #[command(autocomplete = true, rename = "filter-group")]
    filter_group: String,
}

impl Remove {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let guild_id = get_guild_id!(ctx).get_i64();

        let Some(starboard) =
            Starboard::get_by_name(&ctx.bot.db, &self.starboard, guild_id).await?
        else {
            ctx.respond_str(
                &format!("No starboard named '{}' exists.", self.starboard),
                true,
            )
            .await?;
            return Ok(());
        };

        let Some(group) =
            FilterGroup::get_by_name(&ctx.bot.db, guild_id, &self.filter_group).await?
        else {
            ctx.respond_str(
                &format!("No filter group named '{}' exists.", self.filter_group),
                true,
            )
            .await?;
            return Ok(());
        };

        let ret = StarboardFilterGroup::delete(&ctx.bot.db, group.id, starboard.id).await?;

        if ret.is_some() {
            ctx.respond_str(
                &format!(
                    "Removed filter group '{}' from starboard '{}'.",
                    group.name, starboard.name
                ),
                false,
            )
            .await?;
        } else {
            ctx.respond_str("That filter group is not on that starboard.", true)
                .await?;
        }

        Ok(())
    }
}
