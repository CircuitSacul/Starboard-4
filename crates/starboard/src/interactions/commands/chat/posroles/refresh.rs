use twilight_interactions::command::{CommandModel, CreateCommand};

use errors::StarboardResult;

use crate::{
    concat_format,
    core::{posroles::update_posroles_for_guild, premium::is_premium::is_guild_premium},
    get_guild_id,
    interactions::context::CommandCtx,
    utils::id_as_i64::GetI64,
};

#[derive(CommandModel, CreateCommand)]
#[command(name = "refresh", desc = "Refresh the PosRoles for the server.")]
pub struct Refresh;

impl Refresh {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let guild_id = get_guild_id!(ctx);

        if !is_guild_premium(&ctx.bot, guild_id.get_i64(), true).await? {
            ctx.respond_str("Only premium servers can use this command.", true)
                .await?;
            return Ok(());
        }

        ctx.defer(true).await?;

        let ret = update_posroles_for_guild(ctx.bot.clone(), guild_id).await?;

        if let Some(ret) = ret {
            ctx.respond_str(
                &concat_format!(
                    "Finished updating.\n";
                    "Added {} roles, {} failed.\n" <- ret.added_roles, ret.failed_adds;
                    "Removed {} roles, {} failed." <- ret.removed_roles, ret.failed_removals;
                ),
                true,
            )
            .await?;
        } else {
            ctx.respond_str("PosRoles are already being updated.", true)
                .await?;
        }

        Ok(())
    }
}
