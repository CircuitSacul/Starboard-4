mod move_autostar;
mod move_starboard;
mod refresh;

use twilight_interactions::command::{CommandModel, CreateCommand};

use errors::StarboardResult;

use crate::interactions::{commands::permissions::manage_channels, context::CommandCtx};

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "premium-locks",
    desc = "Manage premium locks.",
    default_permissions = "manage_channels",
    dm_permission = false
)]
pub enum PremiumLocks {
    #[command(name = "refresh")]
    Refresh(refresh::Refresh),
    #[command(name = "move-autostar")]
    MoveAutostar(move_autostar::MoveAutostar),
    #[command(name = "move-starboard")]
    MoveStarboard(move_starboard::MoveStarboard),
}

impl PremiumLocks {
    pub async fn callback(self, ctx: CommandCtx) -> StarboardResult<()> {
        match self {
            Self::Refresh(cmd) => cmd.callback(ctx).await,
            Self::MoveAutostar(cmd) => cmd.callback(ctx).await,
            Self::MoveStarboard(cmd) => cmd.callback(ctx).await,
        }
    }
}
