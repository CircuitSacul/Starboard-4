use twilight_model::id::{
    marker::{ChannelMarker, GuildMarker, MessageMarker},
    Id,
};

use database::{DbMember, DbMessage, DbUser};
use errors::StarboardResult;

use crate::{client::bot::StarboardBot, utils::id_as_i64::GetI64};

pub async fn get_or_create_original(
    bot: &StarboardBot,
    guild_id: Id<GuildMarker>,
    channel_id: Id<ChannelMarker>,
    message_id: Id<MessageMarker>,
) -> StarboardResult<(Option<DbMessage>, Option<bool>)> {
    let guild_id_i64 = guild_id.get_i64();
    let channel_id_i64 = channel_id.get_i64();
    let message_id_i64 = message_id.get_i64();

    if let Some(orig) = DbMessage::get_original(&bot.db, message_id_i64).await? {
        return Ok((Some(orig), None));
    }

    // author data
    let (author_is_bot, author_id) = {
        let orig_msg_obj = bot.cache.fog_message(bot, channel_id, message_id).await?;
        let orig_msg_obj = match orig_msg_obj.into_option() {
            None => return Ok((None, None)),
            Some(obj) => obj,
        };

        let user = bot.cache.fog_user(bot, orig_msg_obj.author_id).await?;
        let is_bot = user.map(|u| u.is_bot).unwrap_or(false);
        (is_bot, orig_msg_obj.author_id.get_i64())
    };

    DbUser::create(&bot.db, author_id, author_is_bot).await?;
    DbMember::create(&bot.db, author_id, guild_id.get_i64()).await?;

    let is_nsfw = bot
        .cache
        .fog_channel_nsfw(bot, guild_id, channel_id)
        .await?
        .unwrap();

    // message
    let orig = DbMessage::create(
        &bot.db,
        message_id_i64,
        guild_id_i64,
        channel_id_i64,
        author_id,
        is_nsfw,
    )
    .await?;

    let orig = match orig {
        Some(orig) => orig,
        None => DbMessage::get(&bot.db, message_id_i64).await?.unwrap(),
    };

    Ok((Some(orig), Some(author_is_bot)))
}
