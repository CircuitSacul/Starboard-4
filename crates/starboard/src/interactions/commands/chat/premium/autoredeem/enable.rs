use twilight_interactions::command::{CommandModel, CreateCommand};

use database::DbMember;
use errors::StarboardResult;

use crate::{interactions::context::CommandCtx, utils::id_as_i64::GetI64};

#[derive(CommandModel, CreateCommand)]
#[command(name = "enable", desc = "Enable autoredeem for the current server.")]
pub struct Enable;

impl Enable {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let Some(guild_id) = ctx.interaction.guild_id else {
            ctx.respond_str("Please run this command inside a server.", true)
                .await?;
            return Ok(());
        };
        let guild_id = guild_id.get_i64();
        let user_id = ctx.interaction.author_id().unwrap().get_i64();

        DbMember::create(&ctx.bot.db, user_id, guild_id).await?;
        DbMember::set_autoredeem_enabled(&ctx.bot.db, user_id, guild_id, true).await?;

        ctx.respond_str("Autoredeem enabled.", true).await?;

        Ok(())
    }
}
