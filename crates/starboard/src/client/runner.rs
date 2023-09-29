use std::sync::Arc;

use futures::stream::StreamExt;
use tokio::signal::unix::{signal, SignalKind};
use twilight_gateway::{stream, CloseFrame};

use crate::{
    client::bot::StarboardBot,
    core::{
        posroles::loop_update_posroles,
        premium::{
            expire::loop_expire_premium, patreon::patreon_loop, roles::loop_update_supporter_roles,
        },
    },
    events::handle_event,
};

use super::cooldowns::Cooldowns;

async fn wait_for_shutdown() {
    let mut terminate = signal(SignalKind::terminate()).unwrap();
    let mut interrupt = signal(SignalKind::interrupt()).unwrap();
    tokio::select! {
        _ = terminate.recv() => {
            println!("Received terminate signal.");
        }
        _ = interrupt.recv() => {
            println!("Received interrupt signal.");
        }
    }
}

pub async fn run(bot: StarboardBot) {
    let bot = Arc::new(bot);
    Cooldowns::start(bot.clone());

    if bot.config.development {
        println!("Running bot in development mode.");
    }

    // start background tasks
    tokio::spawn(loop_update_posroles(bot.clone()));
    tokio::spawn(loop_expire_premium(bot.clone()));
    tokio::spawn(patreon_loop(bot.clone()));
    tokio::spawn(loop_update_supporter_roles(bot.clone()));

    // handle events
    let mut shards: Vec<_> = stream::create_range(
        0..bot.config.shards,
        bot.config.shards,
        bot.gw_config.clone(),
        |_, b| b.build(),
    )
    .collect();
    let events = stream::ShardEventStream::new(shards.iter_mut());
    let mut events = events.take_until(Box::pin(wait_for_shutdown()));

    while let Some((shard, event)) = events.next().await {
        let event = match event {
            Ok(event) => event,
            Err(why) => {
                let fatal = why.is_fatal();
                eprintln!("{}: {:#?}", shard.id(), shard.status());
                bot.handle_error(&why.into()).await;

                if fatal {
                    break;
                } else {
                    continue;
                }
            }
        };

        handle_event(shard.id(), event, bot.clone());
    }

    std::mem::drop(events);
    for mut shard in shards {
        if let Err(why) = shard.close(CloseFrame::NORMAL).await {
            bot.handle_error(&why.into()).await;
        };
        println!("Shard {} shutdown.", shard.id());
    }
}
