mod add;
mod remove;
mod set;

use errors::StarboardResult;
use twilight_interactions::command::{CommandModel, CreateCommand};

use crate::interactions::context::CommandCtx;

#[derive(CommandModel, CreateCommand)]
#[command(name = "channels", desc = "Manage the channels an override affects.")]
pub enum ManageOverrideChannels {
    #[command(name = "set")]
    Set(set::SetOverrideChannels),
    #[command(name = "remove")]
    Remove(remove::RemoveOverrideChannels),
    #[command(name = "add")]
    Add(add::AddOverrideChannels),
}

impl ManageOverrideChannels {
    pub async fn callback(self, ctx: CommandCtx) -> StarboardResult<()> {
        match self {
            Self::Set(cmd) => cmd.callback(ctx).await,
            Self::Remove(cmd) => cmd.callback(ctx).await,
            Self::Add(cmd) => cmd.callback(ctx).await,
        }
    }
}
