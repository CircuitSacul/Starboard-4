mod create;
mod delete;
mod rename;

use twilight_interactions::command::{CommandModel, CreateCommand};

use errors::StarboardResult;

use crate::interactions::{commands::permissions::manage_channels, context::CommandCtx};

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "exclusive-groups",
    desc = "Manage exclusive groups for starboards.",
    default_permissions = "manage_channels",
    dm_permission = false
)]
pub enum ExclusiveGroups {
    #[command(name = "create")]
    Create(create::Create),
    #[command(name = "delete")]
    Delete(delete::Delete),
    #[command(name = "rename")]
    Rename(rename::Rename),
}

impl ExclusiveGroups {
    pub async fn callback(self, ctx: CommandCtx) -> StarboardResult<()> {
        match self {
            Self::Create(cmd) => cmd.callback(ctx).await,
            Self::Delete(cmd) => cmd.callback(ctx).await,
            Self::Rename(cmd) => cmd.callback(ctx).await,
        }
    }
}
