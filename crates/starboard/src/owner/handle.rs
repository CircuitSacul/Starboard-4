use std::sync::Arc;

use twilight_model::id::{
    marker::{ChannelMarker, MessageMarker, UserMarker},
    Id,
};

use errors::StarboardResult;

use crate::{cache::models::message::CachedMessage, client::bot::StarboardBot};

use super::commands;

pub async fn handle_message(
    bot: &StarboardBot,
    channel_id: Id<ChannelMarker>,
    message_id: Id<MessageMarker>,
    author_id: Id<UserMarker>,
    message: Option<&CachedMessage>,
    is_edit: bool,
) -> StarboardResult<()> {
    // first check that this is a command being run by the bot owner
    if !bot.config.owner_ids.contains(&author_id.get()) {
        return Ok(());
    }

    let message_owner: Arc<CachedMessage>;
    let message = match message {
        Some(msg) => msg,
        None => {
            let Some(msg) = bot
                .cache
                .fog_message(bot, channel_id, message_id)
                .await?
                .into_option()
            else {
                return Ok(());
            };
            message_owner = msg;
            &message_owner
        }
    };

    // split by space
    let tokens: Vec<_> = message.content.trim().split([' ', '\n']).collect();

    // need at least two tokens
    if tokens.len() < 2 {
        return Ok(());
    }

    // first token should be a prefix
    if tokens[0].to_lowercase().trim() != "star" {
        return Ok(());
    }

    // match second token to a command, if any
    let ret = match tokens[1] {
        "sql" => commands::sql::run_sql(bot, channel_id, message_id, message, is_edit).await,
        // "embed" => commands::embed_test::test_starboard_embed(bot, event).await?,
        _ => Ok(()),
    };

    if let Err(err) = ret {
        bot.http
            .create_message(channel_id)
            .content(&err.to_string())?
            .await?;
    }

    Ok(())
}
