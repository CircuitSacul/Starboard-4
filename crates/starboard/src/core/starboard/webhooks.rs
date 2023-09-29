use std::sync::Arc;

use twilight_model::channel::Webhook;

use database::Starboard;
use errors::StarboardResult;

use crate::{
    client::bot::StarboardBot,
    utils::{id_as_i64::GetI64, into_id::IntoId},
};

pub async fn create_webhook(bot: &StarboardBot, starboard: &Starboard) -> &'static str {
    if get_valid_webhook(bot, starboard, true, false).await.is_ok() {
        concat!(
            "I created a webhook in the starboard channel. You can change the username ",
            "and avatar by going to channel settings > integrations > webhooks."
        )
    } else {
        concat!(
            "I tried to create a webhook, but couldn't. Please make sure I have the ",
            "necessary permissions."
        )
    }
}

pub async fn get_valid_webhook(
    bot: &StarboardBot,
    starboard: &Starboard,
    allow_create: bool,
    allow_cache: bool,
) -> StarboardResult<Option<Arc<Webhook>>> {
    if let Some(wh) = starboard.webhook_id {
        if let Some(wh) = bot
            .cache
            .fog_webhook(bot, wh.into_id(), allow_cache)
            .await?
        {
            return Ok(Some(wh));
        }

        Starboard::set_webhook(&bot.db, starboard.id, None).await?;
    }

    if !allow_create {
        return Ok(None);
    }

    let webhook_channel = bot
        .cache
        .fog_parent_channel_id(
            bot,
            starboard.guild_id.into_id(),
            starboard.channel_id.into_id(),
        )
        .await?;
    let Some(webhook_channel) = webhook_channel else {
        return Ok(None);
    };

    let name = format!("Webhook for '{}'", starboard.name);
    let wh = bot.http.create_webhook(webhook_channel, &name)?;

    let Ok(wh) = wh.await else {
        return Ok(None);
    };
    let wh = Arc::new(wh.model().await?);

    bot.cache.webhooks.insert(wh.id, wh.clone());

    Starboard::set_webhook(&bot.db, starboard.id, Some(wh.id.get_i64())).await?;

    Ok(Some(wh))
}
