use futures::TryStreamExt;
use std::sync::Arc;

use common::constants;
use errors::StarboardResult;

use crate::{
    client::bot::StarboardBot, core::premium::locks::refresh_premium_locks, utils::into_id::IntoId,
};

use super::{
    is_premium::is_guild_premium,
    redeem::{redeem_premium, RedeemPremiumResult},
};

pub async fn loop_expire_premium(bot: Arc<StarboardBot>) {
    loop {
        tokio::time::sleep(constants::CHECK_EXPIRED_PREMIUM).await;

        if let Err(err) = check_expired_premium(bot.clone()).await {
            bot.handle_error(&err).await;
        }
    }
}

async fn check_expired_premium(bot: Arc<StarboardBot>) -> StarboardResult<()> {
    let expired_guilds = sqlx::query!(
        "UPDATE guilds SET premium_end=null WHERE premium_end IS NOT NULL AND premium_end < $1
        RETURNING guild_id",
        chrono::Utc::now(),
    )
    .fetch_all(&bot.db.pool)
    .await?;

    for guild in expired_guilds {
        tokio::spawn(StarboardBot::catch_future_errors(
            bot.clone(),
            process_expired_guild(bot.clone(), guild.guild_id),
        ));
    }

    Ok(())
}

async fn process_expired_guild(bot: Arc<StarboardBot>, guild_id_i64: i64) -> StarboardResult<()> {
    let mut stream = sqlx::query!(
        "SELECT user_id FROM members WHERE autoredeem_enabled=true AND guild_id=$1",
        guild_id_i64
    )
    .fetch(&bot.db.pool);

    let guild_id = guild_id_i64.into_id();

    while let Some(member) = stream.try_next().await? {
        let user_id = member.user_id.into_id();

        let member_obj = bot.cache.fog_member(&bot, guild_id, user_id).await?;
        if member_obj.is_none() {
            continue;
        }

        let ret = redeem_premium(&bot, member.user_id, guild_id_i64, 1, Some(None)).await?;

        match ret {
            RedeemPremiumResult::Ok => break, // successfully added premium
            RedeemPremiumResult::StateMismatch => break, // the server has premium now
            RedeemPremiumResult::TooFewCredits => (), // try another member
        }
    }

    refresh_premium_locks(
        &bot,
        guild_id_i64,
        is_guild_premium(&bot, guild_id_i64, false).await?,
    )
    .await?;

    Ok(())
}
