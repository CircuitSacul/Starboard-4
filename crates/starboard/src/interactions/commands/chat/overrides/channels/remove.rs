use twilight_interactions::command::{CommandModel, CreateCommand};

use database::StarboardOverride;
use errors::StarboardResult;

use crate::{get_guild_id, interactions::context::CommandCtx, parsing, utils::id_as_i64::GetI64};

#[derive(CommandModel, CreateCommand)]
#[command(name = "remove", desc = "Remove channels from an override.")]
pub struct RemoveOverrideChannels {
    /// The override to remove channels from.
    #[command(autocomplete = true, rename = "override")]
    name: String,
    /// The channels to remove.
    channels: String,
}

impl RemoveOverrideChannels {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let guild_id = get_guild_id!(ctx);
        let guild_id_i64 = guild_id.get_i64();

        let ov = StarboardOverride::get(&ctx.bot.db, guild_id_i64, &self.name).await?;
        if let Some(ov) = ov {
            let to_remove =
                parsing::mentions::textable_channel_ids(&ctx.bot, guild_id, &self.channels).await?;
            let channel_ids: Vec<_> = ov
                .channel_ids
                .iter()
                .copied()
                .filter(|id| !to_remove.contains(id))
                .collect();

            let ret = StarboardOverride::set_channels(
                &ctx.bot.db,
                guild_id_i64,
                &self.name,
                &channel_ids,
            )
            .await?;

            if ret.is_some() {
                ctx.respond_str(
                    &format!("Updated the channels for override '{}'.", self.name),
                    false,
                )
                .await?;
                return Ok(());
            }
        }

        ctx.respond_str(
            &format!("No override with the name '{}' exists.", self.name),
            true,
        )
        .await?;
        Ok(())
    }
}
