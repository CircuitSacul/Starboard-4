use chrono::{DateTime, Days, Utc};

use common::constants;
use database::{DbGuild, DbUser};
use errors::StarboardResult;

use crate::client::bot::StarboardBot;

use super::{is_premium::is_guild_premium, locks::refresh_premium_locks};

#[derive(PartialEq, Eq)]
pub enum RedeemPremiumResult {
    Ok,
    StateMismatch,
    TooFewCredits,
}

pub async fn redeem_premium(
    bot: &StarboardBot,
    user_id: i64,
    guild_id: i64,
    months: u64,
    assert_guild_status: Option<Option<DateTime<Utc>>>,
) -> StarboardResult<RedeemPremiumResult> {
    DbGuild::create(&bot.db, guild_id).await?;

    let credits = months * constants::CREDITS_PER_MONTH;

    let mut tx = bot.db.pool.begin().await?;

    // get the guild
    let guild: DbGuild = sqlx::query_as!(
        DbGuild,
        "SELECT * FROM guilds WHERE guild_id=$1 FOR UPDATE",
        guild_id
    )
    .fetch_one(&mut tx)
    .await?;
    if let Some(assert_guild_status) = assert_guild_status {
        if guild.premium_end != assert_guild_status {
            return Ok(RedeemPremiumResult::StateMismatch);
        }
    }

    // get the user
    let user: Option<DbUser> = sqlx::query_as!(
        DbUser,
        "SELECT * FROM users WHERE user_id=$1 FOR UPDATE",
        user_id
    )
    .fetch_optional(&mut tx)
    .await?;
    let Some(user) = user else {
        return Ok(RedeemPremiumResult::TooFewCredits);
    };
    if user.credits < credits as i32 {
        return Ok(RedeemPremiumResult::TooFewCredits);
    }

    // calculate and set the new premium end
    let new_end = if let Some(old_end) = guild.premium_end {
        old_end + Days::new(constants::MONTH_DAYS * months)
    } else {
        Utc::now() + Days::new(constants::MONTH_DAYS * months)
    };

    sqlx::query!(
        "UPDATE guilds SET premium_end=$1 WHERE guild_id=$2",
        new_end,
        guild_id
    )
    .fetch_all(&mut tx)
    .await?;
    sqlx::query!(
        "UPDATE users SET credits = credits - $1 WHERE user_id=$2",
        credits as i32,
        user_id,
    )
    .fetch_all(&mut tx)
    .await?;

    // commit the transaction and return
    tx.commit().await?;

    bot.cache.guild_premium.remove(&guild_id);
    refresh_premium_locks(bot, guild_id, is_guild_premium(bot, guild_id, true).await?).await?;

    Ok(RedeemPremiumResult::Ok)
}
