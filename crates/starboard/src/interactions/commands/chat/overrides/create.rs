use twilight_interactions::command::{CommandModel, CreateCommand};

use common::constants;
use database::{
    validation::{self, ToBotStr},
    Starboard, StarboardOverride,
};
use errors::StarboardResult;

use crate::{get_guild_id, interactions::context::CommandCtx, utils::id_as_i64::GetI64};

#[derive(CommandModel, CreateCommand)]
#[command(name = "create", desc = "Create an override.")]
pub struct CreateOverride {
    /// The name of the override.
    name: String,
    /// The starboard this override belongs too.
    #[command(autocomplete = true)]
    starboard: String,
}

impl CreateOverride {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let guild_id = get_guild_id!(ctx).get_i64();

        let name = match validation::name::validate_name(&self.name) {
            Err(why) => {
                ctx.respond_str(&why.to_bot_str(), true).await?;
                return Ok(());
            }
            Ok(name) => name,
        };

        let starboard = Starboard::get_by_name(&ctx.bot.db, &self.starboard, guild_id).await?;
        let starboard = match starboard {
            None => {
                ctx.respond_str(&format!("'{}' is not a starboard.", self.starboard), true)
                    .await?;
                return Ok(());
            }
            Some(val) => val,
        };

        let count = StarboardOverride::count_by_starboard(&ctx.bot.db, starboard.id).await?;
        if count >= constants::MAX_OVERRIDES_PER_STARBOARD {
            ctx.respond_str(
                &format!(
                    "You can only have up to {} overrides per starboard.",
                    constants::MAX_OVERRIDES_PER_STARBOARD
                ),
                true,
            )
            .await?;
            return Ok(());
        }

        let ov = StarboardOverride::create(&ctx.bot.db, guild_id, &name, starboard.id).await?;

        if ov.is_none() {
            ctx.respond_str(
                &format!("An override with the name '{name}' already exists."),
                true,
            )
            .await?;
        } else {
            ctx.respond_str(
                &format!(
                    "Created override '{}' in starboard '{}'.",
                    name, self.starboard
                ),
                false,
            )
            .await?;
        }

        Ok(())
    }
}
