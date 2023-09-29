mod channels;
mod create;
mod delete;
mod edit;
mod rename;
mod view;

use twilight_interactions::command::{CommandModel, CreateCommand};

use errors::StarboardResult;

use crate::interactions::{commands::permissions::manage_channels, context::CommandCtx};

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "overrides",
    desc = "Manage overrides.",
    dm_permission = false,
    default_permissions = "manage_channels"
)]
pub enum Overrides {
    #[command(name = "create")]
    Create(create::CreateOverride),
    #[command(name = "delete")]
    Delete(delete::DeleteOverride),
    #[command(name = "rename")]
    Rename(rename::RenameOverride),
    #[command(name = "channels")]
    Channels(channels::ManageOverrideChannels),
    #[command(name = "edit")]
    Edit(edit::EditOverride),
    #[command(name = "view")]
    View(view::ViewOverride),
}

impl Overrides {
    pub async fn callback(self, ctx: CommandCtx) -> StarboardResult<()> {
        match self {
            Self::Create(cmd) => cmd.callback(ctx).await,
            Self::Delete(cmd) => cmd.callback(ctx).await,
            Self::Rename(cmd) => cmd.callback(ctx).await,
            Self::Channels(cmd) => cmd.callback(ctx).await,
            Self::Edit(cmd) => cmd.callback(ctx).await,
            Self::View(cmd) => cmd.callback(ctx).await,
        }
    }
}
