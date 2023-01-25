use twilight_model::application::command::CommandOptionChoice;

use crate::{
    database::AutoStarChannel, errors::StarboardResult, interactions::context::CommandCtx,
    utils::id_as_i64::GetI64,
};

use super::best_matches::best_matches_as_choices;

pub async fn autostar_name_autocomplete(
    ctx: &CommandCtx,
    focused: &str,
) -> StarboardResult<Vec<CommandOptionChoice>> {
    let guild_id = ctx.interaction.guild_id.unwrap();
    let asc = AutoStarChannel::list_by_guild(&ctx.bot.pool, guild_id.get_i64()).await?;
    let names: Vec<&str> = asc.iter().map(|a| a.name.as_str()).collect();

    Ok(best_matches_as_choices(focused, &names, None))
}
