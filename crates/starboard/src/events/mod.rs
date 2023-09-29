use std::{sync::Arc, time::Duration};

use twilight_gateway::{Event, ShardId};

use errors::StarboardResult;

use crate::{
    cache::models::message::CachedMessage,
    client::bot::StarboardBot,
    core,
    interactions::{commands::register::post_commands, handle::handle_interaction},
    utils::into_id::IntoId,
};

pub fn handle_event(shard_id: ShardId, event: Event, bot: Arc<StarboardBot>) {
    tokio::spawn(internal_handle_event(shard_id, event, bot));
}

async fn internal_handle_event(shard_id: ShardId, event: Event, bot: Arc<StarboardBot>) {
    bot.cache.update(&event).await;
    bot.standby.process(&event);

    let ret = tokio::spawn(match_events(shard_id, event, bot.clone())).await;

    match ret {
        Ok(ret) => match ret {
            Ok(_) => {}
            Err(why) => bot.handle_error(&why).await,
        },
        Err(why) => bot.handle_error(&why.into()).await,
    }
}

async fn match_events(
    shard_id: ShardId,
    event: Event,
    bot: Arc<StarboardBot>,
) -> StarboardResult<()> {
    match event {
        Event::InteractionCreate(int) => handle_interaction(int.0, bot).await?,
        Event::Ready(info) => {
            println!("Shard {} connected.", shard_id);
            if bot.application.read().await.is_none() {
                bot.application.write().await.replace(info.application);
                post_commands(bot).await;
            }
        }
        Event::ThreadCreate(event) => {
            let Some(guild_id) = event.guild_id else {
                return Ok(());
            };
            let Some(parent_id) = event.parent_id else {
                return Ok(());
            };

            // the initial message doesn't exist right away, so we need
            // to wait a few seconds. Otherwise, the message won't
            // receive its reaction.
            tokio::time::sleep(Duration::from_secs(2)).await;

            if bot.cache.is_channel_forum(guild_id, parent_id) {
                core::autostar::handle(
                    &bot,
                    guild_id,
                    parent_id,
                    event.id,
                    event.id.get().into_id(),
                    None,
                )
                .await?;
            }
        }
        Event::MessageCreate(event) => {
            if event.content == format!("<@{}>", bot.config.bot_id) {
                let _ = bot
                    .http
                    .create_message(event.channel_id)
                    .content(concat!(
                        "See `/help` for more information on Starboard. If you don't see ",
                        "slash commands, try reinviting me using the \"Add to Server\" ",
                        "button on my profile.",
                    ))?
                    .reply(event.id)
                    .await;
            }

            let channel_id = event.channel_id;
            let message_id = event.id;
            let author_id = event.author.id;
            let guild_id = event.guild_id;
            let msg: Arc<CachedMessage> = Arc::new(event.0.into());

            if let Some(guild_id) = guild_id {
                core::autostar::handle(
                    &bot,
                    guild_id,
                    channel_id,
                    channel_id,
                    message_id,
                    Some(msg.clone()),
                )
                .await?;
            }

            crate::owner::handle::handle_message(
                &bot,
                channel_id,
                message_id,
                author_id,
                Some(&msg),
                false,
            )
            .await?;
        }
        Event::MessageUpdate(event) => {
            if let Some(author) = &event.author {
                crate::owner::handle::handle_message(
                    &bot,
                    event.channel_id,
                    event.id,
                    author.id,
                    None,
                    true,
                )
                .await?;
            }

            core::starboard::link_events::handle_message_update(bot, event).await?;
        }
        Event::ReactionAdd(event) => {
            core::starboard::reaction_events::handle_reaction_add(bot, event).await?;
        }
        Event::ReactionRemove(event) => {
            core::starboard::reaction_events::handle_reaction_remove(bot, event).await?;
        }
        Event::MessageDelete(event) => {
            core::starboard::link_events::handle_message_delete(bot, event.id).await?;
        }
        Event::ThreadDelete(event) => {
            core::starboard::link_events::handle_message_delete(bot, event.id.get().into_id())
                .await?;
        }
        _ => {}
    }

    Ok(())
}
