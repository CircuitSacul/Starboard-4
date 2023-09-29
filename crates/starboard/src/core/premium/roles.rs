use std::sync::Arc;

use twilight_model::id::{marker::UserMarker, Id};

use common::constants;
use database::DbUser;
use errors::StarboardResult;

use crate::{
    client::bot::StarboardBot,
    utils::{id_as_i64::GetI64, into_id::IntoId},
};

pub async fn loop_update_supporter_roles(bot: Arc<StarboardBot>) {
    loop {
        tokio::time::sleep(constants::UPDATE_SUPPORTER_ROLES_DELAY).await;

        let clone = bot.clone();
        let ret = tokio::spawn(async move {
            let users = sqlx::query_as!(
                DbUser,
                "SELECT * FROM users WHERE patreon_status!=0 OR donated_cents!=0"
            )
            .fetch_all(&clone.db.pool)
            .await?;

            for user in users {
                update_supporter_roles(&clone, user.user_id.into_id()).await?;
            }

            Ok(())
        })
        .await;

        match ret {
            Ok(Ok(())) => (),
            Ok(Err(err)) => bot.handle_error(&err).await,
            Err(err) => bot.handle_error(&err.into()).await,
        }
    }
}

pub async fn update_supporter_roles(
    bot: &StarboardBot,
    user_id: Id<UserMarker>,
) -> StarboardResult<()> {
    let Some(guild_id) = bot.config.main_guild else {
        return Ok(());
    };
    let supporter_role = bot.config.supporter_role.map(|r| r.into_id());
    let patron_role = bot.config.patron_role.map(|r| r.into_id());
    let guild_id = guild_id.into_id();
    let Some(user) = DbUser::get(&bot.db, user_id.get_i64()).await? else {
        return Ok(());
    };

    let Some(member) = bot.cache.fog_member(bot, guild_id, user_id).await? else {
        return Ok(());
    };
    let has_supporter = supporter_role.map_or(false, |r| member.roles.contains(&r));
    let has_patron = patron_role.map_or(false, |r| member.roles.contains(&r));

    let is_active_patron = user.patreon_status == 1;
    let is_supporter = user.patreon_status != 0 || user.donated_cents != 0 || is_active_patron;

    if let Some(role) = bot.config.patron_role {
        let role_id = role.into_id();
        if is_active_patron {
            if !has_patron {
                bot.http
                    .add_guild_member_role(guild_id, user_id, role_id)
                    .await?;
            }
        } else if has_patron {
            bot.http
                .remove_guild_member_role(guild_id, user_id, role_id)
                .await?;
        }
    }

    if let Some(role) = bot.config.supporter_role {
        let role_id = role.into_id();
        if is_supporter {
            if !has_supporter {
                bot.http
                    .add_guild_member_role(guild_id, user_id, role_id)
                    .await?;
            }
        } else if has_supporter {
            bot.http
                .remove_guild_member_role(guild_id, user_id, role_id)
                .await?;
        }
    }

    Ok(())
}
