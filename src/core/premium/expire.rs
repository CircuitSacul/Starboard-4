use futures::TryStreamExt;
use std::sync::Arc;

use crate::{
    client::bot::StarboardBot, constants, core::premium::locks::refresh_premium_locks,
    errors::StarboardResult,
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
    .fetch_all(&bot.pool)
    .await?;

    for guild in expired_guilds {
        tokio::spawn(StarboardBot::catch_future_errors(
            bot.clone(),
            process_expired_guild(bot.clone(), guild.guild_id),
        ));
    }

    Ok(())
}

async fn process_expired_guild(bot: Arc<StarboardBot>, guild_id: i64) -> StarboardResult<()> {
    let mut stream = sqlx::query!(
        "SELECT user_id FROM members WHERE autoredeem_enabled=true AND guild_id=$1",
        guild_id
    )
    .fetch(&bot.pool);

    while let Some(member) = stream.try_next().await? {
        let ret = redeem_premium(&bot, member.user_id, guild_id, 1, Some(None)).await?;

        if ret == RedeemPremiumResult::Ok {
            break;
        }
    }

    refresh_premium_locks(&bot, guild_id, is_guild_premium(&bot, guild_id).await?).await?;

    Ok(())
}
