use errors::StarboardResult;

use crate::interactions::context::ComponentCtx;

pub async fn handle_dismiss(ctx: &ComponentCtx) -> StarboardResult<()> {
    assert!(ctx.interaction.is_dm());
    let message = ctx.interaction.message.as_ref().unwrap();

    ctx.bot
        .http
        .delete_message(message.channel_id, message.id)
        .await?;

    Ok(())
}
