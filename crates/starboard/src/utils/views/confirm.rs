use std::sync::Arc;

use twilight_model::{
    channel::message::{
        component::{ActionRow, Button, ButtonStyle},
        Component,
    },
    id::{
        marker::{MessageMarker, UserMarker},
        Id,
    },
};

use errors::StarboardResult;

use crate::{
    client::bot::StarboardBot,
    interactions::context::{CommandCtx, ComponentCtx},
};

use super::wait_for::wait_for_component;

pub fn components(danger: bool) -> Vec<Component> {
    let buttons = vec![
        Component::Button(Button {
            custom_id: Some("confirm::no".to_string()),
            disabled: false,
            emoji: None,
            label: Some("Cancel".to_string()),
            style: ButtonStyle::Secondary,
            url: None,
        }),
        Component::Button(Button {
            custom_id: Some("confirm::yes".to_string()),
            disabled: false,
            emoji: None,
            label: Some("Confirm".to_string()),
            style: if danger {
                ButtonStyle::Danger
            } else {
                ButtonStyle::Primary
            },
            url: None,
        }),
    ];

    let row = Component::ActionRow(ActionRow {
        components: buttons,
    });

    vec![row]
}

pub async fn wait_for_result(
    bot: Arc<StarboardBot>,
    message_id: Id<MessageMarker>,
    user_id: Id<UserMarker>,
) -> Option<(ComponentCtx, bool)> {
    let btn_ctx = wait_for_component(
        bot,
        &["confirm::yes", "confirm::no"],
        message_id,
        user_id,
        30,
    )
    .await?;

    let conf = match &*btn_ctx.data.custom_id {
        "confirm::yes" => true,
        "confirm::no" => false,
        _ => unreachable!(),
    };

    Some((btn_ctx, conf))
}

pub async fn simple(
    ctx: &mut CommandCtx,
    prompt: &str,
    danger: bool,
) -> StarboardResult<Option<ComponentCtx>> {
    let cmd = ctx
        .build_resp()
        .content(prompt)
        .components(components(danger))
        .build();
    let msg = ctx.respond(cmd).await?.model().await?;
    let ret = wait_for_result(
        ctx.bot.clone(),
        msg.id,
        ctx.interaction.author_id().unwrap(),
    )
    .await;

    if let Some((mut btn_ctx, conf)) = ret {
        if conf {
            return Ok(Some(btn_ctx));
        } else {
            btn_ctx.edit_str("Canceled.", true).await?;
        }
    } else {
        ctx.bot
            .http
            .update_message(msg.channel_id, msg.id)
            .content(Some("Canceled."))?
            .components(Some(&[]))?
            .await?;
    }

    Ok(None)
}
