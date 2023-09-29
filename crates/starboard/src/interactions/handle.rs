use std::sync::Arc;

use twilight_model::application::interaction::{Interaction, InteractionData, InteractionType};

use errors::StarboardResult;

use crate::client::bot::StarboardBot;

use super::{
    autocomplete::handle::handle_autocomplete, commands::handle::handle_command,
    components::handle::handle_component, context::Ctx,
};

pub async fn handle_interaction(
    interaction: Interaction,
    bot: Arc<StarboardBot>,
) -> StarboardResult<()> {
    let Some(data) = &interaction.data else {
        return Ok(());
    };

    match data {
        InteractionData::ApplicationCommand(data) => {
            let data = *data.clone();
            let ctx = Ctx::new(bot, interaction, data);

            match ctx.interaction.kind {
                InteractionType::ApplicationCommandAutocomplete => handle_autocomplete(ctx).await?,
                InteractionType::ApplicationCommand => handle_command(ctx).await?,
                _ => (),
            }
        }
        InteractionData::MessageComponent(data) => {
            let data = data.to_owned();
            let ctx = Ctx::new(bot, interaction, data);

            handle_component(ctx).await?;
        }
        _ => {}
    }

    Ok(())
}
