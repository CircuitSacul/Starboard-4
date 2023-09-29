use futures::TryStreamExt;
use twilight_interactions::command::{CommandModel, CreateCommand};

use database::DbMember;
use errors::StarboardResult;

use crate::{
    get_guild_id,
    interactions::context::CommandCtx,
    utils::{embed, id_as_i64::GetI64, into_id::IntoId, views::paginator},
};

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "leaderboard",
    desc = "Show the servers leaderboard.",
    dm_permission = false
)]
pub struct Leaderboard {
    /// Whether to include users who've left. False by default.
    #[command(rename = "include-gone")]
    include_gone: Option<bool>,
}

impl Leaderboard {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let guild_id = get_guild_id!(ctx);
        let guild_id_i64 = guild_id.get_i64();

        let include_gone = self.include_gone == Some(true);

        let lb = if include_gone {
            DbMember::list_by_xp(&ctx.bot.db, guild_id_i64, 99).await?
        } else {
            ctx.defer(false).await?;

            let mut lb = Vec::new();
            let mut stream = DbMember::stream_by_xp(&ctx.bot.db, guild_id_i64);

            while let Some(member) = stream.try_next().await? {
                let obj = ctx
                    .bot
                    .cache
                    .fog_member(&ctx.bot, guild_id, member.user_id.into_id())
                    .await?;
                if obj.is_none() {
                    continue;
                }

                lb.push(member);

                if lb.len() >= 99 {
                    break;
                }
            }

            lb
        };

        let mut idx = 0;
        let pages = lb.chunks(9).map(|chunk| {
            chunk
                .iter()
                .map(|DbMember { user_id, xp, .. }| {
                    idx += 1;
                    format!("`#{idx}` <@{user_id}> - {xp} XP\n")
                })
                .collect::<String>()
        });
        let pages: Vec<_> = pages
            .map(|p| {
                (
                    None,
                    Some(vec![embed::build()
                        .title(if include_gone {
                            "Leaderboard (Including Gone)"
                        } else {
                            "Leaderboard"
                        })
                        .description(p)
                        .build()]),
                )
            })
            .collect();

        if pages.is_empty() {
            ctx.respond_str("Nothing to show.", true).await?;
            return Ok(());
        }

        let author_id = ctx.interaction.author_id().unwrap();
        paginator::simple(&mut ctx, pages, author_id, false).await?;

        Ok(())
    }
}
