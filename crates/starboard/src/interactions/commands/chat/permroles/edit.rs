use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_mention::Mention;
use twilight_model::guild::Role;

use database::PermRole;
use errors::StarboardResult;

use crate::{
    interactions::{commands::choices::tribool::Tribool, context::CommandCtx},
    utils::id_as_i64::GetI64,
};

#[derive(CommandModel, CreateCommand)]
#[command(name = "edit", desc = "Edit the global permissions for a PermRole.")]
pub struct EditPermRole {
    /// The PermRole to edit.
    role: Role,

    /// Whether a user with this role can vote on messages.
    vote: Option<Tribool>,
    /// Whether a user with this role can receive votes.
    #[command(rename = "receive-votes")]
    receive_votes: Option<Tribool>,
    /// Whether a user with this role can gain XPRoles.
    xproles: Option<Tribool>,
}

impl EditPermRole {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let pr = PermRole::get(&ctx.bot.db, self.role.id.get_i64()).await?;
        let mut pr = match pr {
            None => {
                ctx.respond_str(&format!("{} is not a PermRole.", self.role.mention()), true)
                    .await?;
                return Ok(());
            }
            Some(pr) => pr,
        };

        if let Some(val) = self.vote {
            pr.give_votes = val.as_bool();
        }
        if let Some(val) = self.receive_votes {
            pr.receive_votes = val.as_bool();
        }
        if let Some(val) = self.xproles {
            pr.obtain_xproles = val.as_bool();
        }

        pr.update(&ctx.bot.db).await?;

        ctx.respond_str(
            &format!("Updated settings for {}", self.role.mention()),
            false,
        )
        .await?;
        Ok(())
    }
}
