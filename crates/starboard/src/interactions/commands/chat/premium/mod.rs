mod autoredeem;
mod info;
mod redeem;

use twilight_interactions::command::{CommandModel, CreateCommand};

use errors::StarboardResult;

use crate::interactions::context::CommandCtx;

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "premium",
    desc = "Premium-releated commands. See /premium-locks for locks."
)]
pub enum Premium {
    #[command(name = "info")]
    Info(info::Info),
    #[command(name = "autoredeem")]
    Autoredeem(autoredeem::Autoredeem),
    #[command(name = "redeem")]
    Redeem(redeem::Redeem),
}

impl Premium {
    pub async fn callback(self, ctx: CommandCtx) -> StarboardResult<()> {
        match self {
            Self::Info(cmd) => cmd.callback(ctx).await,
            Self::Autoredeem(cmd) => cmd.callback(ctx).await,
            Self::Redeem(cmd) => cmd.callback(ctx).await,
        }
    }
}
