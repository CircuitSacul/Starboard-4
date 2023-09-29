use errors::StarboardResult;

use crate::{DbClient, Starboard};

pub async fn move_lock(
    db: &DbClient,
    guild_id: i64,
    from: &str,
    to: &str,
) -> StarboardResult<Result<(), String>> {
    let mut tx = db.pool.begin().await?;

    let Some(from) = Starboard::get_by_name_for_update(&mut tx, from, guild_id).await? else {
        return Ok(Err(format!("Starboard '{from}' does not exist.")));
    };
    let Some(to) = Starboard::get_by_name_for_update(&mut tx, to, guild_id).await? else {
        return Ok(Err(format!("Starboard '{to}' does not exist.")));
    };

    if !from.premium_locked {
        return Ok(Err(format!("Starboard '{}' is not locked.", from.name)));
    }
    if to.premium_locked {
        return Ok(Err(format!("Starboard '{}' is already locked.", to.name)));
    }

    sqlx::query!(
        "UPDATE starboards SET premium_locked=true WHERE id=$1",
        to.id
    )
    .execute(&mut tx)
    .await?;
    sqlx::query!(
        "UPDATE starboards SET premium_locked=false WHERE id=$1",
        from.id
    )
    .execute(&mut tx)
    .await?;

    tx.commit().await?;

    Ok(Ok(()))
}
