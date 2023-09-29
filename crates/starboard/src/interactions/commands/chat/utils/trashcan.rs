use twilight_interactions::command::{CommandModel, CreateCommand};

use database::DbMessage;
use errors::StarboardResult;

use crate::{
    get_guild_id,
    interactions::context::CommandCtx,
    utils::{id_as_i64::GetI64, message_link::fmt_message_link, views::paginator},
};

#[derive(CommandModel, CreateCommand)]
#[command(name = "trashcan", desc = "View all trashed messages.")]
pub struct TrashCan;

impl TrashCan {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let guild_id = get_guild_id!(ctx).get_i64();

        let trashed = DbMessage::list_trashed(&ctx.bot.db, guild_id).await?;

        if trashed.is_empty() {
            ctx.respond_str("There are no trashed messages.", true)
                .await?;
            return Ok(());
        }

        let pages = trashed.chunks(50).map(|chunk| {
            chunk
                .iter()
                .map(|msg| {
                    let reason = msg.trash_reason.as_deref().unwrap_or("No reason given.");
                    let link = fmt_message_link(guild_id, msg.channel_id, msg.message_id);

                    format!("[{reason}]({link})\n")
                })
                .collect::<String>()
        });

        let author_id = ctx.interaction.author_id().unwrap();
        paginator::simple(
            &mut ctx,
            pages.map(|page| (Some(page), None)).collect(),
            author_id,
            true,
        )
        .await?;

        Ok(())
    }
}
