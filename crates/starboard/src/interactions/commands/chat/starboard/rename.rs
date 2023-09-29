use twilight_interactions::command::{CommandModel, CreateCommand};

use database::{
    validation::{self, ToBotStr},
    Starboard,
};
use errors::{PgErrorTraits, StarboardResult};

use crate::{get_guild_id, interactions::context::CommandCtx, utils::id_as_i64::GetI64};

#[derive(CreateCommand, CommandModel)]
#[command(name = "rename", desc = "Rename a starboard.")]
pub struct RenameStarboard {
    /// The current name of the starboard.
    #[command(rename = "current-name", autocomplete = true)]
    current_name: String,
    /// The new name for the starboard.
    #[command(rename = "new-name")]
    new_name: String,
}

impl RenameStarboard {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let guild_id = get_guild_id!(ctx);

        let new_name = match validation::name::validate_name(&self.new_name) {
            Err(why) => {
                ctx.respond_str(&why.to_bot_str(), true).await?;
                return Ok(());
            }
            Ok(name) => name,
        };

        let ret = Starboard::rename(
            &ctx.bot.db,
            &self.current_name,
            guild_id.get_i64(),
            &new_name,
        )
        .await;

        match ret {
            Err(why) => {
                if why.is_duplicate() {
                    ctx.respond_str(
                        &format!("A starboard with the name '{new_name}' already exists."),
                        true,
                    )
                    .await?
                } else {
                    return Err(why.into());
                }
            }
            Ok(None) => {
                ctx.respond_str("No starboard with that name was found.", true)
                    .await?
            }
            Ok(Some(_)) => {
                ctx.respond_str(
                    &format!(
                        "Renamed the starboard from '{}' to '{}'.",
                        self.current_name, new_name
                    ),
                    false,
                )
                .await?
            }
        };

        Ok(())
    }
}
