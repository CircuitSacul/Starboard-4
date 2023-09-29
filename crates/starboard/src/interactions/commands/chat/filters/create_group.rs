use twilight_interactions::command::{CommandModel, CreateCommand};

use common::constants;
use database::{
    validation::{name::validate_name, ToBotStr},
    DbGuild, FilterGroup,
};
use errors::StarboardResult;

use crate::{get_guild_id, interactions::context::CommandCtx, utils::id_as_i64::GetI64};

#[derive(CommandModel, CreateCommand)]
#[command(name = "create-group", desc = "Create a filter group.")]
pub struct CreateGroup {
    /// The name of the filter group.
    name: String,
}

impl CreateGroup {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let guild_id = get_guild_id!(ctx).get_i64();

        let count = FilterGroup::list_by_guild(&ctx.bot.db, guild_id)
            .await?
            .len();
        if count >= constants::MAX_FILTER_GROUPS {
            ctx.respond_str(
                &format!(
                    "You can only have up to {} filter groups.",
                    constants::MAX_FILTER_GROUPS
                ),
                true,
            )
            .await?;
            return Ok(());
        }

        DbGuild::create(&ctx.bot.db, guild_id).await?;
        let name = match validate_name(&self.name) {
            Ok(val) => val,
            Err(why) => {
                ctx.respond_str(&why.to_bot_str(), true).await?;
                return Ok(());
            }
        };
        let group = FilterGroup::create(&ctx.bot.db, guild_id, &name).await?;
        if group.is_none() {
            ctx.respond_str(
                &format!("A filter group named '{name}' already exists."),
                true,
            )
            .await?;
        } else {
            ctx.respond_str(&format!("Created filter group '{name}'."), false)
                .await?;
        }

        Ok(())
    }
}
