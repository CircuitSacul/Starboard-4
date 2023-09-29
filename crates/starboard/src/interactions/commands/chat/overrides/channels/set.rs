use twilight_interactions::command::{CommandModel, CreateCommand};

use database::StarboardOverride;
use errors::StarboardResult;

use crate::{get_guild_id, interactions::context::CommandCtx, parsing, utils::id_as_i64::GetI64};

#[derive(CommandModel, CreateCommand)]
#[command(name = "set", desc = "Set the channels that an override affects.")]
pub struct SetOverrideChannels {
    /// The override to set the channels for.
    #[command(autocomplete = true, rename = "override")]
    name: String,
    /// A list of channels that the override should affect. Use "none" to
    /// remove all.
    channels: String,
}

impl SetOverrideChannels {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let guild_id = get_guild_id!(ctx);
        let guild_id_i64 = guild_id.get_i64();

        let channel_ids: Vec<_> =
            parsing::mentions::textable_channel_ids(&ctx.bot, guild_id, &self.channels)
                .await?
                .into_iter()
                .collect();
        if let Err(why) = StarboardOverride::validate_channels(&channel_ids) {
            ctx.respond_str(&why, true).await?;
            return Ok(());
        }
        let ov =
            StarboardOverride::set_channels(&ctx.bot.db, guild_id_i64, &self.name, &channel_ids)
                .await?;

        if ov.is_none() {
            ctx.respond_str(
                &format!("No override with the name '{}' exists.", self.name),
                true,
            )
            .await?;
        } else {
            ctx.respond_str(
                &format!("Set the channels for override '{}'.", self.name),
                false,
            )
            .await?;
        }
        Ok(())
    }
}
