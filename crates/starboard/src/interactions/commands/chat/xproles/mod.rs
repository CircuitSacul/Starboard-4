mod delete;
mod setxp;
mod view;

use twilight_interactions::command::{CommandModel, CreateCommand};

use errors::StarboardResult;

use crate::interactions::{commands::permissions::manage_roles, context::CommandCtx};

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "xproles",
    desc = "View and manage XP-based award roles.",
    dm_permission = false,
    default_permissions = "manage_roles"
)]
pub enum XPRoles {
    #[command(name = "setxp")]
    SetXP(setxp::SetXP),
    #[command(name = "delete")]
    Delete(delete::Delete),
    #[command(name = "clear-deleted")]
    ClearDeleted(delete::ClearDeleted),
    #[command(name = "view")]
    View(view::View),
}

impl XPRoles {
    pub async fn callback(self, ctx: CommandCtx) -> StarboardResult<()> {
        match self {
            Self::SetXP(cmd) => cmd.callback(ctx).await,
            Self::Delete(cmd) => cmd.callback(ctx).await,
            Self::ClearDeleted(cmd) => cmd.callback(ctx).await,
            Self::View(cmd) => cmd.callback(ctx).await,
        }
    }
}
