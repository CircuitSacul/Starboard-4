use common::constants;
use errors::StarboardResult;

use crate::{client::bot::StarboardBot, utils::into_id::IntoId};

pub async fn refresh_premium_locks(
    bot: &StarboardBot,
    guild_id: i64,
    premium: bool,
) -> StarboardResult<()> {
    // unlock everything first
    sqlx::query!(
        "UPDATE starboards SET premium_locked=false WHERE guild_id=$1",
        guild_id
    )
    .fetch_all(&bot.db.pool)
    .await?;
    let unlocked_asc_channel_ids = sqlx::query!(
        "UPDATE autostar_channels SET premium_locked=false WHERE guild_id=$1
        RETURNING channel_id",
        guild_id
    )
    .fetch_all(&bot.db.pool)
    .await?;
    for row in unlocked_asc_channel_ids {
        bot.cache
            .autostar_channel_ids
            .insert(row.channel_id.into_id());
    }

    // if premium, just return
    if premium {
        return Ok(());
    }

    // lock starboards
    let count = sqlx::query!(
        "SELECT count(*) as count FROM starboards WHERE guild_id=$1 AND premium_locked=false",
        guild_id
    )
    .fetch_one(&bot.db.pool)
    .await?
    .count
    .unwrap();
    let to_lock = count - constants::MAX_STARBOARDS;
    if to_lock > 0 {
        let sb_ids = sqlx::query!(
            "SELECT id FROM starboards WHERE guild_id=$1 LIMIT $2",
            guild_id,
            to_lock
        )
        .fetch_all(&bot.db.pool)
        .await?;
        sqlx::query!(
            "UPDATE starboards SET premium_locked=true WHERE id=any($1)",
            &sb_ids.into_iter().map(|r| r.id).collect::<Vec<_>>()
        )
        .fetch_all(&bot.db.pool)
        .await?;
    }

    // lock autostar channels
    let count = sqlx::query!(
        "SELECT count(*) as count FROM autostar_channels WHERE guild_id=$1 AND 
        premium_locked=false",
        guild_id
    )
    .fetch_one(&bot.db.pool)
    .await?
    .count
    .unwrap();
    let to_lock = count - constants::MAX_AUTOSTAR;
    if to_lock > 0 {
        let asc_ids = sqlx::query!(
            "SELECT id FROM autostar_channels WHERE guild_id=$1 LIMIT $2",
            guild_id,
            to_lock
        )
        .fetch_all(&bot.db.pool)
        .await?;
        sqlx::query!(
            "UPDATE autostar_channels SET premium_locked=true WHERE id=any($1)",
            &asc_ids.into_iter().map(|r| r.id).collect::<Vec<_>>()
        )
        .fetch_all(&bot.db.pool)
        .await?;
    }

    Ok(())
}
