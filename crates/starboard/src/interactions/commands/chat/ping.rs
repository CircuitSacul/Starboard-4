use twilight_interactions::command::{CommandModel, CreateCommand};

use errors::StarboardResult;

use crate::interactions::context::CommandCtx;

#[derive(CreateCommand, CommandModel)]
#[command(name = "ping", desc = "Pong!")]
pub struct Ping;

impl Ping {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        ctx.respond_str("Pong! I'm here.", false).await?;

        Ok(())
    }
}
