use twilight_model::application::command::CommandOptionChoice;

use database::DbMember;
use errors::StarboardResult;

use crate::{
    interactions::context::CommandCtx,
    utils::{id_as_i64::GetI64, into_id::IntoId},
};

use super::best_matches::best_matches_as_choices;

pub async fn autoredeem_autocomplete(
    ctx: &CommandCtx,
    focused: &str,
) -> StarboardResult<Vec<CommandOptionChoice>> {
    let user_id = ctx.interaction.author_id().unwrap().get_i64();

    let guild_ids = DbMember::list_autoredeem_by_user(&ctx.bot.db, user_id).await?;

    let get_guild_name = |id| {
        ctx.bot
            .cache
            .guilds
            .with(&id, |_, g| g.as_ref().map(|g| format!("{} {id}", g.name)))
            .unwrap_or_else(|| format!("Unkown Server {id}"))
    };
    let guild_names = guild_ids
        .iter()
        .map(|id| get_guild_name(id.into_id()))
        .collect::<Vec<_>>();
    let guild_names_ref = guild_names
        .iter()
        .map(|name| name.as_str())
        .collect::<Vec<_>>();

    Ok(best_matches_as_choices(
        focused,
        &guild_names_ref,
        Some(|n: &str| n.split(' ').last().unwrap().to_string()),
    ))
}
