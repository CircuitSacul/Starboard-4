use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::guild::Role;

use database::PosRole;
use errors::StarboardResult;

use crate::{
    concat_format,
    core::premium::is_premium::is_guild_premium,
    get_guild_id,
    interactions::{commands::deleted_roles::get_deleted_roles, context::CommandCtx},
    utils::{id_as_i64::GetI64, into_id::IntoId, views::confirm},
};

#[derive(CommandModel, CreateCommand)]
#[command(name = "delete", desc = "Delete a position-based award role.")]
pub struct Delete {
    /// The PosRole to delete.
    posrole: Role,
}

impl Delete {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let role = PosRole::delete(&ctx.bot.db, self.posrole.id.get_i64()).await?;

        let (msg, ephemeral) = match role {
            None => ("That is not a PosRole.", true),
            Some(_) => ("PosRole deleted.", false),
        };
        ctx.respond_str(msg, ephemeral).await?;

        Ok(())
    }
}

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "clear-deleted",
    desc = "Delete PosRoles if the Discord role has been deleted."
)]
pub struct ClearDeleted;

impl ClearDeleted {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let guild_id = get_guild_id!(ctx);
        let guild_id_i64 = guild_id.get_i64();

        if !is_guild_premium(&ctx.bot, guild_id_i64, true).await? {
            ctx.respond_str("Only premium servers can use this command.", true)
                .await?;
            return Ok(());
        }

        let pr = PosRole::list_by_guild(&ctx.bot.db, guild_id_i64).await?;

        let (to_delete_pretty, to_delete) = get_deleted_roles(
            &ctx.bot,
            guild_id,
            pr.into_iter().map(|r| r.role_id.into_id()),
        );

        if to_delete.is_empty() {
            ctx.respond_str("Nothing to clear.", true).await?;
            return Ok(());
        }

        let conf = confirm::simple(
            &mut ctx,
            &concat_format!(
                "This will delete the following PosRoles:\n";
                "{to_delete_pretty}\n";
                "Do you wish to continue?";
            ),
            true,
        )
        .await?;

        let Some(mut btn_ctx) = conf else {
            return Ok(());
        };

        for role in to_delete {
            PosRole::delete(&ctx.bot.db, role.get_i64()).await?;
        }

        btn_ctx.edit_str("Done.", true).await?;

        Ok(())
    }
}
