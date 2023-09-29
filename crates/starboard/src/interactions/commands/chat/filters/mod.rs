mod create_filter;
mod create_group;
mod delete_filter;
mod delete_group;
mod edit;
mod move_filter;
mod rename_group;
mod view;

use twilight_interactions::command::{CommandModel, CreateCommand};

use errors::StarboardResult;

use crate::interactions::{commands::permissions::manage_roles_channels, context::CommandCtx};

#[allow(clippy::large_enum_variant)] // Edit(edit::Edit) being the culprit
#[derive(CommandModel, CreateCommand)]
#[command(
    name = "filters",
    desc = "Manage filters.",
    default_permissions = "manage_roles_channels",
    dm_permission = false
)]
pub enum Filters {
    #[command(name = "create-group")]
    CreateGroup(create_group::CreateGroup),
    #[command(name = "delete-group")]
    DeleteGroup(delete_group::DeleteGroup),
    #[command(name = "rename-group")]
    RenameGroup(rename_group::RenameGroup),

    #[command(name = "create-filter")]
    CreateFilter(create_filter::CreateFilter),
    #[command(name = "delete-filter")]
    DeleteFilter(delete_filter::DeleteFilter),
    #[command(name = "move-filter")]
    MoveFilter(move_filter::MoveFilter),
    #[command(name = "edit")]
    Edit(edit::Edit),

    #[command(name = "view")]
    View(view::View),
}

impl Filters {
    pub async fn callback(self, ctx: CommandCtx) -> StarboardResult<()> {
        match self {
            Self::CreateGroup(cmd) => cmd.callback(ctx).await,
            Self::DeleteGroup(cmd) => cmd.callback(ctx).await,
            Self::RenameGroup(cmd) => cmd.callback(ctx).await,

            Self::CreateFilter(cmd) => cmd.callback(ctx).await,
            Self::DeleteFilter(cmd) => cmd.callback(ctx).await,
            Self::MoveFilter(cmd) => cmd.callback(ctx).await,
            Self::Edit(cmd) => cmd.callback(ctx).await,

            Self::View(cmd) => cmd.callback(ctx).await,
        }
    }
}
