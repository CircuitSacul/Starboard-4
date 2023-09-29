use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::guild::Role;

use common::constants;
use database::XPRole;
use errors::StarboardResult;

use crate::{
    core::premium::is_premium::is_guild_premium, get_guild_id, interactions::context::CommandCtx,
    utils::id_as_i64::GetI64,
};

#[derive(CommandModel, CreateCommand)]
#[command(name = "setxp", desc = "Create or modify an XP-based award role.")]
pub struct SetXP {
    /// The role to use as an XP-based award role.
    role: Role,
    /// How much XP is required to obtain this award role.
    #[command(min_value = 1, max_value = 32_767, rename = "required-xp")]
    required_xp: i64,
}

impl SetXP {
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

        let count = XPRole::count(&ctx.bot.db, guild_id).await?;
        if count >= constants::MAX_XPROLES {
            ctx.respond_str(
                &format!(
                    "You can only have up to {} XP-based award roles.",
                    constants::MAX_XPROLES
                ),
                true,
            )
            .await?;
            return Ok(());
        }

        let role_id = self.role.id.get_i64();
        let xprole =
            XPRole::create(&ctx.bot.db, role_id, guild_id, self.required_xp as i16).await?;

        if xprole.is_none() {
            XPRole::set_required(&ctx.bot.db, role_id, self.required_xp as i16).await?;
            ctx.respond_str(
                &format!("Required XP changed to {}.", self.required_xp,),
                false,
            )
            .await?;
        } else {
            ctx.respond_str("XP-based award role created.", false)
                .await?;
        }

        Ok(())
    }
}
