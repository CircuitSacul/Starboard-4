use twilight_interactions::command::{CommandModel, CreateCommand};

use database::{
    validation::{name::validate_name, ToBotStr},
    ExclusiveGroup,
};
use errors::{PgErrorTraits, StarboardResult};

use crate::{get_guild_id, interactions::context::CommandCtx, utils::id_as_i64::GetI64};

#[derive(CommandModel, CreateCommand)]
#[command(name = "rename", desc = "Rename an exclusive group.")]
pub struct Rename {
    /// The original name for the exclusive group.
    #[command(rename = "original-name", autocomplete = true)]
    original_name: String,
    /// The new name for the exclusive group.
    #[command(rename = "new-name")]
    new_name: String,
}

impl Rename {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let guild_id = get_guild_id!(ctx).get_i64();

        let new_name = match validate_name(&self.new_name) {
            Err(why) => {
                ctx.respond_str(&why.to_bot_str(), true).await?;
                return Ok(());
            }
            Ok(name) => name,
        };

        let ret =
            ExclusiveGroup::rename(&ctx.bot.db, guild_id, &self.original_name, &new_name).await;

        let err = match ret {
            Err(why) => {
                if why.is_duplicate() {
                    format!("An exclusive group named '{new_name}' already exists.")
                } else {
                    return Err(why.into());
                }
            }
            Ok(None) => format!("Exclusive group '{}' does not exist.", self.original_name),
            Ok(Some(_)) => {
                ctx.respond_str("Done.", true).await?;
                return Ok(());
            }
        };
        ctx.respond_str(&err, true).await?;

        Ok(())
    }
}
