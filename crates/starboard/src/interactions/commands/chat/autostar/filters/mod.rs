mod add;
mod remove;

use errors::StarboardResult;

use crate::interactions::context::CommandCtx;
use twilight_interactions::command::{CommandModel, CreateCommand};

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "filters",
    desc = "Manage filter groups for an autostar channel."
)]
pub enum Filters {
    #[command(name = "add")]
    Add(add::Add),
    #[command(name = "remove")]
    Remove(remove::Remove),
}

impl Filters {
    pub async fn callback(self, ctx: CommandCtx) -> StarboardResult<()> {
        match self {
            Self::Add(cmd) => cmd.callback(ctx).await,
            Self::Remove(cmd) => cmd.callback(ctx).await,
        }
    }
}
