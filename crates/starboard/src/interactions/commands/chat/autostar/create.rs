use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::application::interaction::application_command::InteractionChannel;

use common::constants;
use database::{
    validation::{self, ToBotStr},
    AutoStarChannel, DbGuild,
};
use errors::StarboardResult;

use crate::{
    core::premium::is_premium::is_guild_premium, get_guild_id, interactions::context::CommandCtx,
    utils::id_as_i64::GetI64,
};

#[derive(CommandModel, CreateCommand)]
#[command(name = "create", desc = "Create an autostar channel.")]
pub struct CreateAutoStarChannel {
    /// The name of the autostar channel.
    name: String,
    /// The channel to create an autostar channel in.
    #[command(channel_types = r#"
            guild_text
            guild_voice
            guild_stage_voice
            guild_announcement
            announcement_thread
            public_thread
            private_thread
            guild_forum
        "#)]
    channel: InteractionChannel,
}

impl CreateAutoStarChannel {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let guild_id = get_guild_id!(ctx).get_i64();
        let channel_id = self.channel.id.get_i64();

        DbGuild::create(&ctx.bot.db, guild_id).await?;

        let name = match validation::name::validate_name(&self.name) {
            Err(why) => {
                ctx.respond_str(&why.to_bot_str(), true).await?;
                return Ok(());
            }
            Ok(name) => name,
        };

        let count = AutoStarChannel::count_by_guild(&ctx.bot.db, guild_id).await?;
        let limit = if is_guild_premium(&ctx.bot, guild_id, true).await? {
            constants::MAX_PREM_AUTOSTAR
        } else {
            constants::MAX_AUTOSTAR
        };
        if count >= limit {
            ctx.respond_str(
                &format!(
                    "You can only have up to {} autostar channels. The premium limit is {}.",
                    limit,
                    constants::MAX_PREM_AUTOSTAR,
                ),
                true,
            )
            .await?;
            return Ok(());
        }

        let ret = AutoStarChannel::create(&ctx.bot.db, &name, channel_id, guild_id).await?;

        if ret.is_none() {
            ctx.respond_str(
                &format!("An autostar channel with the name '{name}' already exists."),
                true,
            )
            .await?;
        } else {
            ctx.bot.cache.autostar_channel_ids.insert(self.channel.id);

            ctx.respond_str(
                &format!("Created autostar channel '{name}' in <#{channel_id}>."),
                false,
            )
            .await?;
        }

        Ok(())
    }
}
