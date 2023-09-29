use twilight_interactions::command::{CommandModel, CreateCommand};

use database::{DbMessage, Starboard};
use errors::StarboardResult;

use crate::{
    core::{
        premium::is_premium::is_guild_premium,
        starboard::{handle::RefreshMessage, message::get_or_create_original},
    },
    get_guild_id,
    interactions::context::CommandCtx,
    utils::{id_as_i64::GetI64, into_id::IntoId, message_link::parse_message_link},
};

#[derive(CommandModel, CreateCommand)]
#[command(name = "force", desc = "Force a message to one or all starboards.")]
pub struct Force {
    /// Link to the message to force.
    message: String,

    /// The starboard to force to. Leave blank to force to all.
    #[command(autocomplete = true)]
    starboard: Option<String>,
}

impl Force {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let Some((channel_id, message_id)) = parse_message_link(&self.message) else {
            ctx.respond_str("Invalid message link.", true).await?;
            return Ok(());
        };

        let guild_id = get_guild_id!(ctx);

        // validate that the channel is in the guild
        if !ctx
            .bot
            .cache
            .guild_has_channel(&ctx.bot, guild_id, channel_id.into_id())
            .await?
        {
            ctx.respond_str("That message belongs to a different server.", true)
                .await?;
            return Ok(());
        }

        let forced: Vec<_> = match self.starboard {
            None => Starboard::list_by_guild(&ctx.bot.db, guild_id.get_i64())
                .await?
                .into_iter()
                .map(|s| s.id)
                .collect(),
            Some(name) => {
                let Some(sb) =
                    Starboard::get_by_name(&ctx.bot.db, &name, guild_id.get_i64()).await?
                else {
                    ctx.respond_str(&format!("Starboard '{name}' does not exist."), true)
                        .await?;
                    return Ok(());
                };
                vec![sb.id]
            }
        };

        if forced.is_empty() {
            // if the length != 1, that means it's trying to force to all starboards. So, if the
            // length is 0, that means there are no starboards.

            ctx.respond_str(
                "This server has no starboards, so you can't force messages.",
                true,
            )
            .await?;
            return Ok(());
        }

        let ret = get_or_create_original(
            &ctx.bot,
            guild_id,
            channel_id.into_id(),
            message_id.into_id(),
        )
        .await?;
        let (Some(orig), _) = ret else {
            ctx.respond_str(
                concat!(
                    "I don't have the necessary permissions to see that message. Make ",
                    "sure I have the 'view channel' and 'read message history' ",
                    "permissions in that channel."
                ),
                true,
            )
            .await?;
            return Ok(());
        };

        let mut forced = forced;
        for already_forced in orig.forced_to {
            if !forced.contains(&already_forced) {
                forced.push(already_forced);
            }
        }

        let is_premium = is_guild_premium(&ctx.bot, guild_id.get_i64(), true).await?;
        DbMessage::set_forced(&ctx.bot.db, orig.message_id, &forced).await?;
        RefreshMessage::new(ctx.bot.clone(), orig.message_id.into_id(), is_premium)
            .refresh(true)
            .await?;
        ctx.respond_str("Message forced.", true).await?;

        Ok(())
    }
}
