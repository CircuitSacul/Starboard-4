use twilight_interactions::command::{CommandModel, CreateCommand};

use database::FilterGroup;
use errors::StarboardResult;

use crate::{
    get_guild_id,
    interactions::context::CommandCtx,
    utils::{id_as_i64::GetI64, views::confirm},
};

#[derive(CommandModel, CreateCommand)]
#[command(name = "delete-group", desc = "Delete a filter group.")]
pub struct DeleteGroup {
    /// The filter group to delete.
    #[command(autocomplete = true)]
    name: String,
}

impl DeleteGroup {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let guild_id = get_guild_id!(ctx).get_i64();

        let Some(mut btn_ctx) = confirm::simple(
            &mut ctx,
            &format!(
                "This will delete the filter group '{}'. Continue?",
                self.name
            ),
            true,
        )
        .await?
        else {
            return Ok(());
        };

        let filter = FilterGroup::delete(&ctx.bot.db, guild_id, &self.name).await?;

        if filter.is_some() {
            btn_ctx
                .edit_str(&format!("Deleted filter group '{}'.", self.name), true)
                .await?;
        } else {
            btn_ctx
                .edit_str(
                    &format!("Filter group '{}' does not exist.", self.name),
                    true,
                )
                .await?;
        }

        Ok(())
    }
}
