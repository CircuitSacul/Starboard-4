use std::sync::Arc;

use anyhow::Result;
use twilight_interactions::command::CreateCommand;

use crate::client::bot::StarboardBot;
use crate::interactions::commands::chat;

macro_rules! commands_to_create {
    ($( $command: ty ),* ) => {
        vec![
            $(
                <$command>::create_command().into(),
            )*
        ]
    };
}

pub async fn post_commands(bot: Arc<StarboardBot>) -> Result<()> {
    let inter_client = bot.interaction_client().await?;

    let commands = commands_to_create!(chat::ping::Ping);

    match inter_client.set_global_commands(&commands).exec().await {
        Ok(_) => println!("Successfully registered commands"),
        Err(e) => eprintln!("Failed to register commands: {}", e),
    }

    Ok(())
}
