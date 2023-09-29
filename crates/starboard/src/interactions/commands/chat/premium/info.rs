use std::{borrow::Cow, fmt::Write};

use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::channel::message::MessageFlags;
use twilight_util::builder::embed::EmbedFieldBuilder;

use common::constants;
use database::{DbGuild, DbMember, DbUser};
use errors::StarboardResult;

use crate::{
    concat_format,
    interactions::context::CommandCtx,
    utils::{embed, id_as_i64::GetI64, into_id::IntoId},
};

#[derive(CommandModel, CreateCommand)]
#[command(name = "info", desc = "Get premium info.")]
pub struct Info;

impl Info {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let user_id = ctx.interaction.author_id().unwrap().get_i64();

        let user = DbUser::get(&ctx.bot.db, user_id).await?;
        let credits = match &user {
            Some(user) => user.credits,
            None => 0,
        };

        let mut emb = embed::build()
            .title("Starboard Premium")
            .description(concat_format!(
                "Starboard uses a credit system for premium. For each USD you donate (currently ";
                "only Patreon is supported), you receive one premium credit. Three premium ";
                "credits can be redeemed for one month of premium in any server of your choice.";
                "\n\nThis means that premium for one server is $3/month, two servers is $6/month, ";
                "and so on. To get premium, visit my [Patreon]({})." <- constants::PATREON_URL;
            ))
            .field(EmbedFieldBuilder::new(
                "Status",
                format!("You currently have {credits} credits."),
            ));

        'out: {
            if user.is_none() {
                break 'out;
            }

            let ar = DbMember::list_autoredeem_by_user(&ctx.bot.db, user_id).await?;
            if ar.is_empty() {
                break 'out;
            }

            let mut value = "Autoredeem is enabled for the following servers:\n".to_string();
            for guild_id in ar {
                ctx.bot.cache.guilds.with(&guild_id.into_id(), |_, guild| {
                    if let Some(guild) = &guild {
                        value.push_str(&guild.name);
                        value.push('\n');
                    } else {
                        writeln!(value, "Deleted Guild {guild_id}").unwrap();
                    }
                });
            }

            value.push_str(concat!(
                "\nAutoredeem will automatically take credits from your account when the server ",
                "runs out of premium. This will only occur if Starboard is still in that server ",
                "and you are still in that server.\n\n Disable it at any time by using ",
                "`/premium autoredeem disable`."
            ));

            emb = emb.field(EmbedFieldBuilder::new("Autoredeem", value));
        }

        if let Some(guild_id) = ctx.interaction.guild_id {
            let guild = DbGuild::get(&ctx.bot.db, guild_id.get_i64()).await?;

            let value = match guild.and_then(|g| g.premium_end) {
                None => Cow::Borrowed("This server does not have premium."),
                Some(end) => Cow::Owned(format!(
                    "This server has premium until <t:{}:F>.",
                    end.timestamp()
                )),
            };
            emb = emb.field(EmbedFieldBuilder::new("Server Premium", value));
        };

        ctx.respond(
            ctx.build_resp()
                .embeds([emb.build()])
                .flags(MessageFlags::EPHEMERAL)
                .build(),
        )
        .await?;

        Ok(())
    }
}
