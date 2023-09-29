pub mod force;
pub mod freeze;
pub mod info;
pub mod recount;
pub mod refresh;
pub mod trash;
pub mod trashcan;
pub mod unforce;

use twilight_interactions::command::{CommandModel, CreateCommand};

use errors::StarboardResult;

use crate::interactions::{commands::permissions::manage_messages, context::CommandCtx};

const INVALID_MESSAGE_ERR: &str = concat!(
    "I couldn't find that message. There are a few possible reasons why:",
    "\n- I don't have access to the channel the message is in.",
    "\n- The message doesn't exist.",
    "\n- The message doesn't have any upvotes, so it isn't in the database.",
);

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "utils",
    desc = "Utility commands.",
    dm_permission = false,
    default_permissions = "manage_messages"
)]
pub enum Utils {
    #[command(name = "info")]
    Info(info::Info),

    #[command(name = "freeze")]
    Freeze(freeze::Freeze),
    #[command(name = "unfreeze")]
    UnFreeze(freeze::UnFreeze),

    #[command(name = "force")]
    Force(force::Force),
    #[command(name = "unforce")]
    UnForce(unforce::UnForce),

    #[command(name = "trash")]
    Trash(trash::Trash),
    #[command(name = "untrash")]
    UnTrash(trash::UnTrash),
    #[command(name = "trashcan")]
    TrashCan(trashcan::TrashCan),

    #[command(name = "refresh")]
    Refresh(refresh::Refresh),
    #[command(name = "recount")]
    Recount(recount::Recount),
}

impl Utils {
    pub async fn callback(self, ctx: CommandCtx) -> StarboardResult<()> {
        match self {
            Self::Info(cmd) => cmd.callback(ctx).await,

            Self::Freeze(cmd) => cmd.callback(ctx).await,
            Self::UnFreeze(cmd) => cmd.callback(ctx).await,

            Self::Force(cmd) => cmd.callback(ctx).await,
            Self::UnForce(cmd) => cmd.callback(ctx).await,

            Self::Trash(cmd) => cmd.callback(ctx).await,
            Self::UnTrash(cmd) => cmd.callback(ctx).await,
            Self::TrashCan(cmd) => cmd.callback(ctx).await,

            Self::Refresh(cmd) => cmd.callback(ctx).await,
            Self::Recount(cmd) => cmd.callback(ctx).await,
        }
    }
}
