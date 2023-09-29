use thousands::Separable;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::user::User;

use errors::StarboardResult;

use crate::{
    concat_format,
    core::stats::MemberStats,
    get_guild_id,
    interactions::context::CommandCtx,
    utils::{embed, id_as_i64::GetI64},
};

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "stats",
    desc = "Show stats for you or another user in this server",
    dm_permission = false
)]
pub struct Stats {
    /// The user to show stats for.
    user: Option<User>,
}

impl Stats {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let guild_id = get_guild_id!(ctx).get_i64();

        let user_id = self
            .user
            .map(|u| u.id)
            .unwrap_or_else(|| ctx.interaction.author_id().unwrap())
            .get_i64();

        let Some(stats) = MemberStats::get(&ctx.bot.db, guild_id, user_id).await? else {
            ctx.respond_str("No stats to show.", true).await?;
            return Ok(());
        };

        let emb = {
            let xp = stats.xp.separate_with_commas();
            let recv_up = stats.received_upvotes.separate_with_commas();
            let recv_down = stats.received_downvotes.separate_with_commas();
            let give_up = stats.given_upvotes.separate_with_commas();
            let give_down = stats.given_downvotes.separate_with_commas();

            let pad = [&xp, &recv_up, &recv_down, &give_up, &give_down]
                .into_iter()
                .map(|s| s.len())
                .max()
                .unwrap();

            embed::build()
                .title("User Stats")
                .description(concat_format!(
                    "Showing Stats for <@{user_id}>\n\n";
                    "`{: >pad$}` - Total XP\n" <- stats.xp;
                    "`{: >pad$}` - Total Upvotes Received\n" <- stats.received_upvotes;
                    "`{: >pad$}` - Total Downvotes Received\n\n" <- stats.received_downvotes;
                    "`{: >pad$}` - Total Upvotes Given\n" <- stats.given_upvotes;
                    "`{: >pad$}` - Total Downvotes Given\n" <- stats.given_downvotes;
                ))
                .build()
        };

        ctx.respond(ctx.build_resp().embeds([emb]).build()).await?;

        Ok(())
    }
}
