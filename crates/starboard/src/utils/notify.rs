use twilight_model::{
    channel::message::component::{ActionRow, Button, ButtonStyle, Component},
    id::{marker::UserMarker, Id},
};

use errors::StarboardResult;

use crate::client::bot::StarboardBot;

use super::dm;

pub async fn notify(
    bot: &StarboardBot,
    user_id: Id<UserMarker>,
    message: &str,
) -> StarboardResult<()> {
    if bot.config.development {
        println!("Development, skipping notification:");
        println!("{message}");
        return Ok(());
    }

    let comp = Component::ActionRow(ActionRow {
        components: vec![Component::Button(Button {
            label: Some("Dismiss".to_string()),
            url: None,
            style: ButtonStyle::Secondary,
            custom_id: Some("stateless::dismiss_notification".to_string()),
            disabled: false,
            emoji: None,
        })],
    });

    let _ = dm::dm(bot, user_id)
        .await?
        .content(message)?
        .components(&[comp])?
        .await;

    Ok(())
}
