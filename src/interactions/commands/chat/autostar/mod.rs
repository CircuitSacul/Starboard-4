pub mod create;
pub mod delete;
pub mod edit;
pub mod rename;
pub mod view;

use twilight_interactions::command::{CommandModel, CreateCommand};

use crate::interactions::commands::context::CommandCtx;
use crate::interactions::commands::permissions::manage_channels;

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "autostar",
    desc = "Manage autostar channels.",
    dm_permission = false,
    default_permissions = "manage_channels"
)]
pub enum AutoStar {
    #[command(name = "create")]
    Create(create::CreateAutoStarChannel),
    #[command(name = "delete")]
    Delete(delete::DeleteAutoStarChannel),
    #[command(name = "view")]
    View(view::ViewAutoStarChannels),
    #[command(name = "edit")]
    Edit(edit::EditAutoStar),
    #[command(name = "rename")]
    Rename(rename::RenameAutoStarChannel),
}

impl AutoStar {
    pub async fn callback(self, ctx: CommandCtx) -> anyhow::Result<()> {
        match self {
            Self::Create(cmd) => cmd.callback(ctx).await,
            Self::Delete(cmd) => cmd.callback(ctx).await,
            Self::View(cmd) => cmd.callback(ctx).await,
            Self::Edit(cmd) => cmd.callback(ctx).await,
            Self::Rename(cmd) => cmd.callback(ctx).await,
        }
    }
}
