use twilight_model::id::{
    marker::{GuildMarker, UserMarker},
    Id,
};

use database::{PermRole, PermRoleStarboard};
use errors::StarboardResult;

use crate::{
    client::bot::StarboardBot,
    utils::{id_as_i64::GetI64, into_id::IntoId, sort_permroles::SortVecPermRole},
};

pub struct Permissions {
    pub give_votes: bool,
    pub receive_votes: bool,
    pub obtain_xproles: bool,
}

impl Default for Permissions {
    fn default() -> Self {
        Self::new()
    }
}

impl Permissions {
    pub fn new() -> Self {
        Self {
            give_votes: true,
            receive_votes: true,
            obtain_xproles: true,
        }
    }

    pub async fn get_permissions(
        bot: &StarboardBot,
        user_id: Id<UserMarker>,
        guild_id: Id<GuildMarker>,
        starboard_id: Option<i32>,
    ) -> StarboardResult<Self> {
        let guild_id_i64 = guild_id.get_i64();
        let mut perms = Self::new();

        // get permroles
        let permroles = PermRole::list_by_guild(&bot.db, guild_id_i64).await?;

        if permroles.is_empty() {
            return Ok(perms);
        }

        // filter out non-applicable permroles
        let member = bot.cache.fog_member(bot, guild_id, user_id).await?;
        let roles = member.as_ref().map(|m| &m.roles);

        let mut permroles = if let Some(roles) = roles {
            permroles
                .into_iter()
                .filter(|r| r.role_id == guild_id_i64 || roles.contains(&r.role_id.into_id()))
                .collect::<Vec<_>>()
        } else {
            let guild_id: i64 = guild_id_i64;
            permroles
                .into_iter()
                .filter(|r| r.role_id == guild_id)
                .collect::<Vec<_>>()
        };

        // sort permroles by their order in the guild
        permroles.sort_permroles(&bot.cache);

        for pr in permroles {
            if let Some(val) = pr.give_votes {
                perms.give_votes = val;
            }
            if let Some(val) = pr.receive_votes {
                perms.receive_votes = val;
            }
            if let Some(val) = pr.obtain_xproles {
                perms.obtain_xproles = val;
            }

            if let Some(sb_id) = starboard_id {
                let pr_sb = PermRoleStarboard::get(&bot.db, pr.role_id, sb_id).await?;
                let Some(pr_sb) = pr_sb else {
                    continue;
                };

                if let Some(val) = pr_sb.give_votes {
                    perms.give_votes = val;
                }
                if let Some(val) = pr_sb.receive_votes {
                    perms.receive_votes = val;
                }
            }
        }

        Ok(perms)
    }
}
