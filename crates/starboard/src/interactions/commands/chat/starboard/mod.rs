pub mod create;
pub mod delete;
pub mod edit;
pub mod filters;
pub mod rename;
pub mod view;

use twilight_interactions::command::{CommandModel, CreateCommand};

use errors::StarboardResult;

use crate::interactions::{commands::permissions::manage_channels, context::CommandCtx};

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "starboards",
    desc = "Manage starboards.",
    dm_permission = false,
    default_permissions = "manage_channels"
)]
pub enum Starboard {
    #[command(name = "create")]
    Create(create::CreateStarboard),
    #[command(name = "delete")]
    Delete(delete::DeleteStarboard),
    #[command(name = "view")]
    View(view::ViewStarboard),
    #[command(name = "rename")]
    Rename(rename::RenameStarboard),
    #[command(name = "edit")]
    Edit(edit::EditStarboard),
    #[command(name = "filters")]
    Filters(filters::Filters),
}

impl Starboard {
    pub async fn callback(self, ctx: CommandCtx) -> StarboardResult<()> {
        match self {
            Self::Create(cmd) => cmd.callback(ctx).await,
            Self::Delete(cmd) => cmd.callback(ctx).await,
            Self::View(cmd) => cmd.callback(ctx).await,
            Self::Rename(cmd) => cmd.callback(ctx).await,
            Self::Edit(cmd) => cmd.callback(ctx).await,
            Self::Filters(cmd) => cmd.callback(ctx).await,
        }
    }
}
