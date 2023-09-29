use twilight_interactions::command::{CommandModel, CreateCommand};

use database::{Filter, FilterGroup};
use errors::StarboardResult;

use crate::{get_guild_id, interactions::context::CommandCtx, utils::id_as_i64::GetI64};

#[derive(CommandModel, CreateCommand)]
#[command(name = "move-filter", desc = "Change the position of a filter.")]
pub struct MoveFilter {
    /// The filter group containing the filter to be moved.
    #[command(autocomplete = true)]
    group: String,
    /// The original position of the filter.
    #[command(rename = "current-position", min_value = 1, max_value = 1_000)]
    current_position: i64,
    /// The new position of the filter.
    #[command(rename = "new-position", min_value = 1, max_value = 1_000)]
    new_position: i64,
}

impl MoveFilter {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let guild_id = get_guild_id!(ctx).get_i64();

        let group = FilterGroup::get_by_name(&ctx.bot.db, guild_id, &self.group).await?;
        let Some(group) = group else {
            ctx.respond_str(
                &format!("Filter group '{}' does not exist.", self.group),
                true,
            )
            .await?;
            return Ok(());
        };

        let ret = Filter::set_position(
            &ctx.bot.db,
            group.id,
            self.current_position as i16,
            self.new_position as i16,
        )
        .await?;
        if ret.is_some() {
            ctx.respond_str(
                &format!(
                    "Filter moved from {} to {}.",
                    self.current_position, self.new_position
                ),
                false,
            )
            .await?;
        } else {
            ctx.respond_str(
                &format!(
                    "There is no filter at {} for group '{}'.",
                    self.current_position, self.group
                ),
                true,
            )
            .await?;
        }

        Ok(())
    }
}
