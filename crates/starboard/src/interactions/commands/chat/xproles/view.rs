use std::fmt::Write;

use thousands::Separable;
use twilight_interactions::command::{CommandModel, CreateCommand};

use database::XPRole;
use errors::StarboardResult;

use crate::{
    core::premium::is_premium::is_guild_premium,
    get_guild_id,
    interactions::context::CommandCtx,
    utils::{embed, id_as_i64::GetI64, views::paginator},
};

#[derive(CommandModel, CreateCommand)]
#[command(name = "view", desc = "View all of your XP-based award roles.")]
pub struct View;

impl View {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let guild_id = get_guild_id!(ctx).get_i64();

        if !is_guild_premium(&ctx.bot, guild_id, true).await? {
            ctx.respond_str("Only premium servers can use this command.", true)
                .await?;
            return Ok(());
        }

        let xproles = XPRole::list_by_guild(&ctx.bot.db, guild_id).await?;
        if xproles.is_empty() {
            ctx.respond_str("There are no XPRoles.", true).await?;
            return Ok(());
        }

        let mut embeds = Vec::new();
        for chunk in xproles.chunks(10) {
            let mut desc = String::new();

            for xpr in chunk {
                writeln!(
                    desc,
                    "<@&{}> - `{}` XP",
                    xpr.role_id,
                    xpr.required.separate_with_commas()
                )
                .unwrap();
            }

            let emb = embed::build()
                .title("XP-based Award Roles")
                .description(desc)
                .build();

            embeds.push(emb);
        }

        let author_id = ctx.interaction.author_id().unwrap();
        paginator::simple(
            &mut ctx,
            embeds
                .into_iter()
                .map(|emb| (None, Some(vec![emb])))
                .collect(),
            author_id,
            false,
        )
        .await?;

        Ok(())
    }
}
