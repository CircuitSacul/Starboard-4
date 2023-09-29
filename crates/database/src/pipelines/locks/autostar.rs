use errors::StarboardResult;

use crate::{AutoStarChannel, DbClient};

pub async fn move_lock(
    db: &DbClient,
    guild_id: i64,
    from: &str,
    to: &str,
) -> StarboardResult<Result<(), String>> {
    let mut tx = db.pool.begin().await?;

    let Some(from) = AutoStarChannel::get_by_name_for_update(&mut tx, from, guild_id).await? else {
        return Ok(Err(format!("Autostar channel '{from}' does not exist.")));
    };
    let Some(to) = AutoStarChannel::get_by_name_for_update(&mut tx, to, guild_id).await? else {
        return Ok(Err(format!("Autostar channel '{to}' does not exist.")));
    };

    if !from.premium_locked {
        return Ok(Err(format!(
            "Autostar channel '{}' is not locked.",
            from.name
        )));
    }
    if to.premium_locked {
        return Ok(Err(format!(
            "Autostar channel '{}' is already locked.",
            to.name
        )));
    }

    sqlx::query!(
        "UPDATE autostar_channels SET premium_locked=true WHERE id=$1",
        to.id
    )
    .execute(&mut tx)
    .await?;
    sqlx::query!(
        "UPDATE autostar_channels SET premium_locked=false WHERE id=$1",
        from.id
    )
    .execute(&mut tx)
    .await?;

    tx.commit().await?;

    Ok(Ok(()))
}
