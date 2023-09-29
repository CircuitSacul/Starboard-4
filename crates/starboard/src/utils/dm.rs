use twilight_http::request::channel::message::CreateMessage;
use twilight_model::id::{marker::UserMarker, Id};

use errors::StarboardResult;

use crate::client::bot::StarboardBot;

pub async fn dm(bot: &StarboardBot, user_id: Id<UserMarker>) -> StarboardResult<CreateMessage> {
    let dm_channel = bot
        .http
        .create_private_channel(user_id)
        .await?
        .model()
        .await?
        .id;
    Ok(bot.http.create_message(dm_channel))
}
