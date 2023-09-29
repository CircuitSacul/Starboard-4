use twilight_interactions::command::CommandModel;

use errors::StarboardResult;

use crate::interactions::{commands::chat, context::CommandCtx};

macro_rules! match_commands {
    ($ctx:expr, $($cmd_name:expr => $command:ty),* $(,)?) => {
        let cmd_inp_data = $ctx.data.clone().into();
        match &*$ctx.data.name {
            $(
                $cmd_name => <$command>::from_interaction(cmd_inp_data)?.callback($ctx).await?,
            )*
            unknown => eprintln!("Unknown command: {}", unknown),
        }
    };
}

pub async fn handle_command(ctx: CommandCtx) -> StarboardResult<()> {
    match_commands!(
        ctx,
        "ping" => chat::ping::Ping,
        "help" => chat::help::Help,
        "botstats" => chat::botstats::BotStats,
        "leaderboard" => chat::leaderboard::Leaderboard,
        "stats" => chat::stats::Stats,
        "random" => chat::random::RandomPost,
        "moststarred" => chat::moststarred::MostStarred,
        "autostar" => chat::autostar::AutoStar,
        "starboards" => chat::starboard::Starboard,
        "overrides" => chat::overrides::Overrides,
        "exclusive-groups" => chat::exclusive_groups::ExclusiveGroups,
        "permroles" => chat::permroles::PermRoles,
        "filters" => chat::filters::Filters,
        "xproles" => chat::xproles::XPRoles,
        "posroles" => chat::posroles::PosRoles,
        "utils" => chat::utils::Utils,
        "premium" => chat::premium::Premium,
        "premium-locks" => chat::premium_locks::PremiumLocks,
    );

    Ok(())
}
