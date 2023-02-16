use twilight_interactions::command::{CommandModel, CreateCommand};

use crate::{
    database::models::{filter::Filter, filter_group::FilterGroup},
    errors::StarboardResult,
    get_guild_id,
    interactions::context::CommandCtx,
    utils::id_as_i64::GetI64,
};

#[derive(CommandModel, CreateCommand)]
#[command(name = "delete-filter", desc = "Delete a filter from a filter group.")]
pub struct DeleteFilter {
    /// The group to delete the filter from.
    #[command(autocomplete = true)]
    group: String,
    /// The position of the filter to delete.
    #[command(min_value = 1, max_value = 1_000)]
    position: i64,
}

impl DeleteFilter {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let guild_id = get_guild_id!(ctx).get_i64();

        let group = FilterGroup::get_by_name(&ctx.bot.pool, guild_id, &self.group).await?;
        let Some(group) = group else {
            ctx.respond_str(&format!("Filter group '{}' does not exist.", self.group), true).await?;
            return Ok(());
        };

        let ret = Filter::delete(&ctx.bot.pool, group.id, self.position as i16).await?;

        if ret.is_some() {
            ctx.respond_str(&format!("Filter at {} deleted.", self.position), false)
                .await?;
        } else {
            ctx.respond_str(
                &format!(
                    "No filter exists at {} for group '{}'.",
                    self.position, self.group
                ),
                true,
            )
            .await?;
        }

        Ok(())
    }
}
