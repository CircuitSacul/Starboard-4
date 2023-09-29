use twilight_interactions::command::{CommandModel, CreateCommand};

use database::{
    validation::{self, ToBotStr},
    StarboardOverride,
};
use errors::{PgErrorTraits, StarboardResult};

use crate::{get_guild_id, interactions::context::CommandCtx, utils::id_as_i64::GetI64};

#[derive(CreateCommand, CommandModel)]
#[command(name = "rename", desc = "Rename an override.")]
pub struct RenameOverride {
    /// The current name of the override.
    #[command(autocomplete = true, rename = "current-name")]
    old_name: String,
    /// The new name of the override.
    #[command(rename = "new-name")]
    name: String,
}

impl RenameOverride {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let guild_id = get_guild_id!(ctx).get_i64();

        let name = match validation::name::validate_name(&self.name) {
            Ok(val) => val,
            Err(why) => {
                ctx.respond_str(&why.to_bot_str(), true).await?;
                return Ok(());
            }
        };

        let ov = StarboardOverride::rename(&ctx.bot.db, guild_id, &self.old_name, &self.name).await;

        match ov {
            Err(why) => {
                if why.is_duplicate() {
                    ctx.respond_str(
                        &format!("An override with the name '{name}' already exists."),
                        true,
                    )
                    .await?;
                } else {
                    return Err(why.into());
                }
            }
            Ok(None) => {
                ctx.respond_str(
                    &format!("No override with the name '{}' exists.", self.old_name),
                    true,
                )
                .await?;
            }
            Ok(Some(_)) => {
                ctx.respond_str(
                    &format!("Renamed override '{}' to '{}'.", self.old_name, name),
                    false,
                )
                .await?;
            }
        }

        Ok(())
    }
}
