pub mod behavior;
pub mod embed;
pub mod requirements;
pub mod reset;
pub mod style;

use errors::StarboardResult;
use twilight_interactions::command::{CommandModel, CreateCommand};

use crate::interactions::context::CommandCtx;

#[derive(CommandModel, CreateCommand)]
#[command(name = "edit", desc = "Edit an override.")]
pub enum EditOverride {
    #[command(name = "embed")]
    Embed(embed::EditEmbedStyle),
    #[command(name = "style")]
    Style(style::EditGeneralStyle),
    #[command(name = "requirements")]
    Requirements(requirements::EditRequirements),
    #[command(name = "behavior")]
    Behaviour(behavior::EditBehavior),
    #[command(name = "reset")]
    Reset(reset::ResetOverrideSettings),
}

impl EditOverride {
    pub async fn callback(self, ctx: CommandCtx) -> StarboardResult<()> {
        match self {
            Self::Embed(cmd) => cmd.callback(ctx).await,
            Self::Style(cmd) => cmd.callback(ctx).await,
            Self::Requirements(cmd) => cmd.callback(ctx).await,
            Self::Behaviour(cmd) => cmd.callback(ctx).await,
            Self::Reset(cmd) => cmd.callback(ctx).await,
        }
    }
}
