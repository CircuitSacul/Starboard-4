use std::sync::Arc;

use twilight_interactions::command::CreateCommand;

use crate::{client::bot::StarboardBot, interactions::commands::chat};

macro_rules! commands_to_create {
    ($( $command: ty ),* $(,)? ) => {
        vec![
            $(
                <$command>::create_command().into(),
            )*
        ]
    };
}

pub async fn post_commands(bot: Arc<StarboardBot>) {
    let inter_client = bot.interaction_client().await;

    let commands = commands_to_create!(
        chat::ping::Ping,
        chat::help::Help,
        chat::botstats::BotStats,
        chat::leaderboard::Leaderboard,
        chat::stats::Stats,
        chat::random::RandomPost,
        chat::moststarred::MostStarred,
        chat::autostar::AutoStar,
        chat::starboard::Starboard,
        chat::overrides::Overrides,
        chat::exclusive_groups::ExclusiveGroups,
        chat::permroles::PermRoles,
        chat::filters::Filters,
        chat::xproles::XPRoles,
        chat::posroles::PosRoles,
        chat::utils::Utils,
        chat::premium::Premium,
        chat::premium_locks::PremiumLocks,
    );

    match inter_client.set_global_commands(&commands).await {
        Ok(_) => println!("Successfully registered commands"),
        Err(e) => eprintln!("Failed to register commands: {e}"),
    }
}
