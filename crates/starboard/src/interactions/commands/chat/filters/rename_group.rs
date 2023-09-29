use twilight_interactions::command::{CommandModel, CreateCommand};

use database::FilterGroup;
use errors::{PgErrorTraits, StarboardResult};

use crate::{get_guild_id, interactions::context::CommandCtx, utils::id_as_i64::GetI64};

#[derive(CommandModel, CreateCommand)]
#[command(name = "rename-group", desc = "Rename a filter group.")]
pub struct RenameGroup {
    /// The current name of the group.
    #[command(autocomplete = true, rename = "current-name")]
    current_name: String,
    /// The new name of the group.
    #[command(rename = "new-name")]
    new_name: String,
}

impl RenameGroup {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let guild_id = get_guild_id!(ctx).get_i64();

        let group = FilterGroup::get_by_name(&ctx.bot.db, guild_id, &self.current_name).await?;
        let Some(group) = group else {
            ctx.respond_str(
                &format!("Filter group '{}' does not exist.", self.current_name),
                true,
            )
            .await?;
            return Ok(());
        };

        let ret = FilterGroup::rename(&ctx.bot.db, group.id, &self.new_name).await;

        match ret {
            Ok(_) => {
                ctx.respond_str("Renamed filter group.", false).await?;
            }
            Err(why) => {
                if why.is_duplicate() {
                    ctx.respond_str(
                        &format!("A filter group named '{}' already exists.", self.new_name),
                        true,
                    )
                    .await?;
                } else {
                    return Err(why.into());
                }
            }
        }

        Ok(())
    }
}
