use twilight_interactions::command::{CommandModel, CreateCommand};

use database::{AutoStarChannel, AutostarChannelFilterGroup, FilterGroup};
use errors::StarboardResult;

use crate::{get_guild_id, interactions::context::CommandCtx, utils::id_as_i64::GetI64};

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "remove",
    desc = "Remove a filter group from an autostar channel."
)]
pub struct Remove {
    /// The autostar channel to remove the filter group from.
    #[command(autocomplete = true, rename = "autostar-channel")]
    autostar_channel: String,
    /// The filter group to remove from the autostar channel.
    #[command(autocomplete = true, rename = "filter-group")]
    filter_group: String,
}

impl Remove {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let guild_id = get_guild_id!(ctx).get_i64();

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

        let Some(asc) =
            AutoStarChannel::get_by_name(&ctx.bot.db, &self.autostar_channel, guild_id).await?
        else {
            ctx.respond_str(
                &format!(
                    "No autostar channel named '{}' exists.",
                    self.autostar_channel
                ),
                true,
            )
            .await?;
            return Ok(());
        };

        let ret = AutostarChannelFilterGroup::delete(&ctx.bot.db, group.id, asc.id).await?;

        if ret.is_some() {
            ctx.respond_str(
                &format!(
                    "Removed the filter group '{}' from autostar channel '{}'.",
                    group.name, asc.name
                ),
                false,
            )
            .await?;
        } else {
            ctx.respond_str(
                "That filter group is not applied to that autostar channel.",
                true,
            )
            .await?;
        }

        Ok(())
    }
}
