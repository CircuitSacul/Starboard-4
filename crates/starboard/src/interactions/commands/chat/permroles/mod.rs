pub mod create;
pub mod delete;
pub mod edit;
pub mod edit_starboard;
pub mod view;

use twilight_interactions::command::{CommandModel, CreateCommand};

use errors::StarboardResult;

use crate::interactions::{commands::permissions::manage_channels, context::CommandCtx};

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "permroles",
    desc = "View and manage PermRoles.",
    dm_permission = false,
    default_permissions = "manage_channels"
)]
pub enum PermRoles {
    #[command(name = "view")]
    View(view::ViewPermRoles),
    #[command(name = "create")]
    Create(create::CreatePermRole),
    #[command(name = "delete")]
    Delete(delete::DeletePermRole),
    #[command(name = "clear-deleted")]
    ClearDeleted(delete::ClearDeleted),
    #[command(name = "edit")]
    Edit(edit::EditPermRole),
    #[command(name = "edit-starboard")]
    EditStarboard(edit_starboard::EditPermRoleStarboard),
}

impl PermRoles {
    pub async fn callback(self, ctx: CommandCtx) -> StarboardResult<()> {
        match self {
            Self::View(cmd) => cmd.callback(ctx).await,
            Self::Create(cmd) => cmd.callback(ctx).await,
            Self::Delete(cmd) => cmd.callback(ctx).await,
            Self::ClearDeleted(cmd) => cmd.callback(ctx).await,
            Self::Edit(cmd) => cmd.callback(ctx).await,
            Self::EditStarboard(cmd) => cmd.callback(ctx).await,
        }
    }
}
