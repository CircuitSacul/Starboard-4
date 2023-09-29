use twilight_model::application::command::CommandOptionChoice;

use database::FilterGroup;
use errors::StarboardResult;

use crate::{interactions::context::CommandCtx, utils::id_as_i64::GetI64};

use super::best_matches::best_matches_as_choices;

pub async fn filter_group_name_autocomplete(
    ctx: &CommandCtx,
    focused: &str,
) -> StarboardResult<Vec<CommandOptionChoice>> {
    let guild_id = ctx.interaction.guild_id.unwrap();
    let groups = FilterGroup::list_by_guild(&ctx.bot.db, guild_id.get_i64()).await?;
    let names: Vec<&str> = groups.iter().map(|g| g.name.as_str()).collect();

    Ok(best_matches_as_choices(focused, &names, None))
}
