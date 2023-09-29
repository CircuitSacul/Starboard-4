use twilight_interactions::command::{CommandModel, CreateCommand};

use errors::StarboardResult;

use crate::{
    core::starboard::recount::{recount_votes, RecountResult},
    get_guild_id,
    interactions::context::CommandCtx,
    utils::{into_id::IntoId, message_link::parse_message_link},
};

#[derive(CommandModel, CreateCommand)]
#[command(name = "recount", desc = "Recount all the reactions on a message.")]
pub struct Recount {
    /// Link to the message to recount reactions on.
    message: String,
}

impl Recount {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let guild_id = get_guild_id!(ctx);

        let Some((channel_id, message_id)) = parse_message_link(&self.message) else {
            ctx.respond_str("Invalid message link.", true).await?;
            return Ok(());
        };

        ctx.defer(true).await?;

        let ret = recount_votes(
            ctx.bot.clone(),
            guild_id,
            channel_id.into_id(),
            message_id.into_id(),
        )
        .await?;
        let msg = match ret {
            RecountResult::UnkownMessage => "I couldn't find that message.",
            RecountResult::AlreadyRecounting => {
                "I'm already recounting the reactions on that message."
            }
            RecountResult::Cooldown(_) => "You're using this command too much.",
            RecountResult::Done => "Finished!",
        };

        ctx.respond_str(msg, true).await?;

        Ok(())
    }
}
