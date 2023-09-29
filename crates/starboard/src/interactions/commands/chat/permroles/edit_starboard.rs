use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_mention::Mention;
use twilight_model::guild::Role;

use database::{PermRoleStarboard, Starboard};
use errors::{PgErrorTraits, StarboardResult};

use crate::{
    get_guild_id,
    interactions::{commands::choices::tribool::Tribool, context::CommandCtx},
    utils::id_as_i64::GetI64,
};

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "edit-starboard",
    desc = "Edit the settings for a PermRole in a starboard."
)]
pub struct EditPermRoleStarboard {
    /// The PermRole to edit.
    role: Role,
    /// The starboard to edit the PermRole for.
    #[command(autocomplete = true)]
    starboard: String,

    /// Whether a user can vote on messages.
    vote: Option<Tribool>,
    /// Whether a user's messages can be voted on.
    #[command(rename = "receive-votes")]
    receive_vote: Option<Tribool>,
}

impl EditPermRoleStarboard {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let guild_id = get_guild_id!(ctx).get_i64();

        let sb = Starboard::get_by_name(&ctx.bot.db, &self.starboard, guild_id).await?;
        let sb = match sb {
            None => {
                ctx.respond_str(
                    &format!("Starboard '{}' does not exist.", self.starboard),
                    true,
                )
                .await?;
                return Ok(());
            }
            Some(sb) => sb,
        };

        let ret = PermRoleStarboard::create(&ctx.bot.db, self.role.id.get_i64(), sb.id).await;
        let mut pr_sb = match ret {
            Ok(Some(val)) => val,
            Ok(None) => PermRoleStarboard::get(&ctx.bot.db, self.role.id.get_i64(), sb.id)
                .await?
                .unwrap(),
            Err(why) => {
                if why.is_fk_violation() {
                    ctx.respond_str(&format!("{} is not a PermRole.", self.role.mention()), true)
                        .await?;
                    return Ok(());
                }
                return Err(why.into());
            }
        };

        if let Some(val) = self.vote {
            pr_sb.give_votes = val.as_bool();
        }
        if let Some(val) = self.receive_vote {
            pr_sb.receive_votes = val.as_bool();
        }

        pr_sb.update(&ctx.bot.db).await?;
        ctx.respond_str(
            &format!(
                "Updated the settings for {} in '{}'.",
                self.role.mention(),
                sb.name
            ),
            false,
        )
        .await?;

        Ok(())
    }
}
