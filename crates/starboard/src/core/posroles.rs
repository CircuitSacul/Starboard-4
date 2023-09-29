use std::sync::Arc;

use futures::TryStreamExt;
use twilight_model::id::{
    marker::{GuildMarker, RoleMarker, UserMarker},
    Id,
};

use common::constants;
use database::{DbMember, PosRole};
use errors::StarboardResult;

use crate::{
    client::bot::StarboardBot,
    utils::{id_as_i64::GetI64, into_id::IntoId},
};

use super::premium::is_premium::is_guild_premium;

pub struct GuildPRUpdateResult {
    pub removed_roles: i32,
    pub added_roles: i32,
    pub failed_removals: i32,
    pub failed_adds: i32,
}

pub async fn loop_update_posroles(bot: Arc<StarboardBot>) {
    loop {
        tokio::time::sleep(constants::UPDATE_PRS_DELAY).await;

        let guilds = sqlx::query!("SELECT DISTINCT guild_id FROM posroles")
            .fetch_all(&bot.db.pool)
            .await;

        let guilds = match guilds {
            Ok(guilds) => guilds,
            Err(err) => {
                bot.handle_error(&err.into()).await;
                continue;
            }
        };

        let mut tasks = Vec::new();
        for guild in guilds {
            let is_prem = match is_guild_premium(&bot, guild.guild_id, true).await {
                Ok(is_prem) => is_prem,
                Err(why) => {
                    bot.handle_error(&why).await;
                    continue;
                }
            };
            if !is_prem {
                continue;
            }
            let task = tokio::spawn(update_posroles_for_guild(
                bot.clone(),
                guild.guild_id.into_id(),
            ));
            tasks.push(task);
        }

        for t in tasks {
            let ret = t.await;
            let ret = match ret {
                Ok(ret) => ret,
                Err(err) => {
                    bot.handle_error(&err.into()).await;
                    continue;
                }
            };
            if let Err(err) = ret {
                bot.handle_error(&err).await;
            }
        }
    }
}

async fn set_role(
    bot: &StarboardBot,
    guild_id: Id<GuildMarker>,
    user_id: Id<UserMarker>,
    posrole_ids: &[Id<RoleMarker>],
    role_id: Option<Id<RoleMarker>>,
) -> StarboardResult<Option<(i32, i32, i32, i32)>> {
    let mut failed_adds = 0;
    let mut added_roles = 0;
    let mut failed_removals = 0;
    let mut removed_roles = 0;

    let Some(member) = bot.cache.fog_member(bot, guild_id, user_id).await? else {
        return Ok(None);
    };

    let to_remove: Vec<_> = posrole_ids
        .iter()
        .filter(|pr| member.roles.contains(*pr) && Some(**pr) != role_id)
        .collect();
    let to_add = role_id.filter(|&role_id| !member.roles.contains(&role_id));

    if let Some(role_id) = to_add {
        let ret = bot
            .http
            .add_guild_member_role(guild_id, user_id, role_id)
            .await;
        if ret.is_err() {
            failed_adds += 1;
        } else {
            added_roles += 1;
        }
    }

    for role in to_remove {
        let ret = bot
            .http
            .remove_guild_member_role(guild_id, user_id, *role)
            .await;
        if ret.is_err() {
            failed_removals += 1;
        } else {
            removed_roles += 1;
        }
    }

    Ok(Some((
        added_roles,
        failed_adds,
        removed_roles,
        failed_removals,
    )))
}

pub async fn update_posroles_for_guild(
    bot: Arc<StarboardBot>,
    guild_id: Id<GuildMarker>,
) -> StarboardResult<Option<GuildPRUpdateResult>> {
    let Some(_lock) = bot.locks.guild_pr_update.lock(guild_id.get_i64()) else {
        return Ok(None);
    };

    let mut removed_roles = 0;
    let mut added_roles = 0;
    let mut failed_removals = 0;
    let mut failed_adds = 0;

    let guild_id_i64 = guild_id.get_i64();

    let posroles = PosRole::list_by_guild(&bot.db, guild_id_i64).await?;
    let posrole_ids: Vec<Id<RoleMarker>> = posroles.iter().map(|pr| pr.role_id.into_id()).collect();

    let lb_size = posroles.iter().map(|pr| pr.max_members).sum::<i32>() * 2;
    let lb_size = lb_size as usize;
    let mut leaderboard = Vec::new();
    let mut stream = DbMember::stream_by_xp(&bot.db, guild_id_i64);

    while let Some(member) = stream.try_next().await? {
        let obj = bot
            .cache
            .fog_member(&bot, guild_id, member.user_id.into_id())
            .await?;
        if obj.is_none() {
            continue;
        }

        leaderboard.push(member);

        if leaderboard.len() > lb_size {
            break;
        }
    }

    for pr in posroles {
        let role_id: Id<RoleMarker> = pr.role_id.into_id();

        let to_drain = [pr.max_members as usize, leaderboard.len()]
            .into_iter()
            .min()
            .unwrap();
        for member in leaderboard.drain(..to_drain) {
            let user_id: Id<UserMarker> = member.user_id.into_id();

            let ret = set_role(&bot, guild_id, user_id, &posrole_ids, Some(role_id)).await?;
            if let Some((aw, af, rw, rf)) = ret {
                added_roles += aw;
                failed_adds += af;
                removed_roles += rw;
                failed_removals += rf;
            }
        }
    }

    for member in leaderboard {
        let user_id: Id<UserMarker> = member.user_id.into_id();

        let ret = set_role(&bot, guild_id, user_id, &posrole_ids, None).await?;
        if let Some((aw, af, rw, rf)) = ret {
            added_roles += aw;
            failed_adds += af;
            removed_roles += rw;
            failed_removals += rf;
        }
    }

    Ok(Some(GuildPRUpdateResult {
        added_roles,
        removed_roles,
        failed_adds,
        failed_removals,
    }))
}
