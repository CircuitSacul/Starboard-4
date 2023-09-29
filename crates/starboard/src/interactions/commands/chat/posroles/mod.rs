mod delete;
mod refresh;
mod set_max_members;
mod view;

use twilight_interactions::command::{CommandModel, CreateCommand};

use errors::StarboardResult;

use crate::interactions::{commands::permissions::manage_roles, context::CommandCtx};

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "posroles",
    desc = "View and manage position-based award roles.",
    dm_permission = false,
    default_permissions = "manage_roles"
)]
pub enum PosRoles {
    #[command(name = "set-max-members")]
    SetMaxMembers(set_max_members::SetMaxMembers),
    #[command(name = "delete")]
    Delete(delete::Delete),
    #[command(name = "clear-deleted")]
    ClearDeleted(delete::ClearDeleted),
    #[command(name = "view")]
    View(view::View),
    #[command(name = "refresh")]
    Refresh(refresh::Refresh),
}

impl PosRoles {
    pub async fn callback(self, ctx: CommandCtx) -> StarboardResult<()> {
        match self {
            Self::SetMaxMembers(cmd) => cmd.callback(ctx).await,
            Self::Delete(cmd) => cmd.callback(ctx).await,
            Self::ClearDeleted(cmd) => cmd.callback(ctx).await,
            Self::View(cmd) => cmd.callback(ctx).await,
            Self::Refresh(cmd) => cmd.callback(ctx).await,
        }
    }
}
