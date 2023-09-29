use std::sync::Arc;

use twilight_model::gateway::payload::incoming::MessageCreate;

use crate::{
    client::bot::StarboardBot,
    core::{embedder::Embedder, starboard::config::StarboardConfig},
    database::Starboard,
    unwrap_id,
};

pub async fn test_starboard_embed(bot: &StarboardBot, event: &MessageCreate) -> anyhow::Result<()> {
    let target = match &event.referenced_message {
        None => anyhow::bail!("No referenced messsage"),
        Some(target) => *target.to_owned(),
    };
    let name = match event.content.strip_prefix("star embed ") {
        None => anyhow::bail!("Invalid starboard name"),
        Some(name) => name,
    };
    let sb = Starboard::get_by_name(&bot.pool, name, unwrap_id!(event.guild_id.unwrap()))
        .await?
        .unwrap();
    let config = StarboardConfig::new(sb, Vec::new())?;

    // let e = Embedder {
    //    points: 1,
    //    config: &config,
    //    orig_message: Arc::new(Some(target.into())),
    // };
    // e.send(bot).await?;

    Ok(())
}
