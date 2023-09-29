use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_mention::Mention;
use twilight_model::guild::Role;

use database::PermRole;
use errors::StarboardResult;

use crate::{
    concat_format, get_guild_id,
    interactions::{commands::deleted_roles::get_deleted_roles, context::CommandCtx},
    utils::{id_as_i64::GetI64, into_id::IntoId, views::confirm},
};

#[derive(CommandModel, CreateCommand)]
#[command(name = "delete", desc = "Delete a PermRole.")]
pub struct DeletePermRole {
    /// The PermRole to delete.
    role: Role,
}

impl DeletePermRole {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let pr = PermRole::delete(&ctx.bot.db, self.role.id.get_i64()).await?;
        if pr.is_none() {
            ctx.respond_str(&format!("{} is not a PermRole.", self.role.mention()), true)
                .await?;
        } else {
            ctx.respond_str(&format!("Deleted PermRole {}.", self.role.mention()), false)
                .await?;
        }

        Ok(())
    }
}

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "clear-deleted",
    desc = "Delete PermRoles if the Discord role has been deleted."
)]
pub struct ClearDeleted;

impl ClearDeleted {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let guild_id = get_guild_id!(ctx);
        let guild_id_i64 = guild_id.get_i64();

        let pr = PermRole::list_by_guild(&ctx.bot.db, guild_id_i64).await?;

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
                "This will delete the following permroles:\n";
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
            PermRole::delete(&ctx.bot.db, role.get_i64()).await?;
        }

        btn_ctx.edit_str("Done.", true).await?;

        Ok(())
    }
}
