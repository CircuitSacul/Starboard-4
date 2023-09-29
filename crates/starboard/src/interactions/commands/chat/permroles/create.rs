use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_mention::Mention;
use twilight_model::guild::Role;

use common::constants;
use database::PermRole;
use errors::StarboardResult;

use crate::{get_guild_id, interactions::context::CommandCtx, utils::id_as_i64::GetI64};

#[derive(CommandModel, CreateCommand)]
#[command(name = "create", desc = "Create a PermRole.")]
pub struct CreatePermRole {
    /// The role to use as a PermRole.
    role: Role,
}

impl CreatePermRole {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let guild_id = get_guild_id!(ctx);
        let guild_id_i64 = guild_id.get_i64();

        let count = PermRole::count_by_guild(&ctx.bot.db, guild_id_i64).await?;
        if count >= constants::MAX_PERMROLES {
            ctx.respond_str(
                &format!(
                    "You can only have up to {} PermRoles.",
                    constants::MAX_PERMROLES
                ),
                true,
            )
            .await?;
            return Ok(());
        }

        let pr = PermRole::create(&ctx.bot.db, self.role.id.get_i64(), guild_id_i64).await?;

        if pr.is_none() {
            ctx.respond_str("That is already a PermRole.", true).await?;
        } else {
            ctx.respond_str(
                &format!("{} is now a PermRole.", self.role.mention()),
                false,
            )
            .await?;
        }

        Ok(())
    }
}
