use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::guild::Role;

use common::constants;
use database::PosRole;
use errors::StarboardResult;

use crate::{
    core::premium::is_premium::is_guild_premium, get_guild_id, interactions::context::CommandCtx,
    utils::id_as_i64::GetI64,
};

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "set-max-members",
    desc = "Create or modify a position-based award role."
)]
pub struct SetMaxMembers {
    /// The role to use as a position-based award role.
    role: Role,
    /// How many members can have this award role.
    #[command(min_value = 1, rename = "max-members")]
    max_members: i64,
}

impl SetMaxMembers {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let guild_id = get_guild_id!(ctx).get_i64();

        if !is_guild_premium(&ctx.bot, guild_id, true).await? {
            ctx.respond_str("Only premium servers can use this command.", true)
                .await?;
            return Ok(());
        }

        if self.role.id.get_i64() == guild_id || self.role.managed {
            ctx.respond_str("You can't use that role for award roles.", true)
                .await?;
            return Ok(());
        }

        let count = PosRole::count(&ctx.bot.db, guild_id).await?;
        if count >= constants::MAX_POSROLES {
            ctx.respond_str(
                &format!(
                    "You can only have up to {} position-based award roles.",
                    constants::MAX_POSROLES
                ),
                true,
            )
            .await?;
            return Ok(());
        }

        let role_id = self.role.id.get_i64();
        let posrole =
            PosRole::create(&ctx.bot.db, role_id, guild_id, self.max_members as i32).await?;

        if posrole.is_none() {
            PosRole::set_max_members(&ctx.bot.db, role_id, self.max_members as i32).await?;
            ctx.respond_str(
                &format!("Max members changed to {}.", self.max_members),
                false,
            )
            .await?;
        } else {
            ctx.respond_str("Position-based award role created.", false)
                .await?;
        }

        Ok(())
    }
}
