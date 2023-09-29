use twilight_interactions::command::{CommandModel, CreateCommand};

use database::{DbMessage, Starboard};
use errors::StarboardResult;

use crate::{
    core::{premium::is_premium::is_guild_premium, starboard::handle::RefreshMessage},
    get_guild_id,
    interactions::context::CommandCtx,
    utils::{id_as_i64::GetI64, into_id::IntoId, message_link::parse_message_link},
};

#[derive(CommandModel, CreateCommand)]
#[command(name = "unforce", desc = "Unforce a message.")]
pub struct UnForce {
    /// Link to the message to unforce.
    message: String,

    /// The starboard to unforce from. Leave blank to unforce from all.
    #[command(autocomplete = true)]
    starboard: Option<String>,
}

impl UnForce {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let guild_id = get_guild_id!(ctx).get_i64();

        let Some((_channel_id, message_id)) = parse_message_link(&self.message) else {
            ctx.respond_str("Invalid message link.", true).await?;
            return Ok(());
        };

        let Some(msg) = DbMessage::get_original(&ctx.bot.db, message_id).await? else {
            ctx.respond_str("That message isn't forced.", true).await?;
            return Ok(());
        };

        if msg.guild_id != guild_id {
            ctx.respond_str("That message belongs to a different server.", true)
                .await?;
            return Ok(());
        }

        match self.starboard {
            Some(name) => {
                let Some(starboard) = Starboard::get_by_name(&ctx.bot.db, &name, guild_id).await?
                else {
                    ctx.respond_str(&format!("Starboard '{name}' does not exist."), true)
                        .await?;
                    return Ok(());
                };

                let new_forced: Vec<_> = msg
                    .forced_to
                    .iter()
                    .filter(|s| **s != starboard.id)
                    .copied()
                    .collect();

                DbMessage::set_forced(&ctx.bot.db, msg.message_id, &new_forced).await?;
            }
            None => {
                DbMessage::set_forced(&ctx.bot.db, msg.message_id, &[]).await?;
            }
        }

        ctx.respond_str("Message unforced.", true).await?;
        let is_premium = is_guild_premium(&ctx.bot, guild_id, true).await?;
        RefreshMessage::new(ctx.bot, msg.message_id.into_id(), is_premium)
            .refresh(true)
            .await?;

        Ok(())
    }
}
