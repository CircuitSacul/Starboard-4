use std::{sync::Arc, time::Duration};

use twilight_model::{
    application::interaction::{Interaction, InteractionData},
    id::{
        marker::{MessageMarker, UserMarker},
        Id,
    },
};

use crate::{client::bot::StarboardBot, interactions::context::ComponentCtx};

pub async fn wait_for_component(
    bot: Arc<StarboardBot>,
    button_ids: &'static [&'static str],
    message_id: Id<MessageMarker>,
    user_id: Id<UserMarker>,
    timeout: u64,
) -> Option<ComponentCtx> {
    let check = move |int: &Interaction| {
        let data = match &int.data {
            None => return false,
            Some(data) => data,
        };

        let data = match data {
            InteractionData::MessageComponent(data) => data,
            _ => return false,
        };

        let msg = match &int.message {
            None => return false,
            Some(msg) => msg,
        };

        let int_user_id = match int.author_id() {
            None => return false,
            Some(user_id) => user_id,
        };

        {
            msg.id == message_id && int_user_id == user_id && button_ids.contains(&&*data.custom_id)
        }
    };

    let event = tokio::time::timeout(
        Duration::from_secs(timeout),
        bot.standby.wait_for_component(message_id, check),
    )
    .await
    .ok()?
    .ok()?;

    let data = {
        let data = &event.data.as_ref().unwrap();
        let data = match data {
            InteractionData::MessageComponent(data) => data,
            _ => unreachable!(),
        };
        data.clone()
    };

    Some(ComponentCtx::new(bot, event, data))
}
