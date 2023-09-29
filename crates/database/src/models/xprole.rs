#[derive(Debug)]
pub struct XPRole {
    pub role_id: i64,
    pub guild_id: i64,
    pub required: i16,
}

#[cfg(feature = "backend")]
impl XPRole {
    pub async fn create(
        db: &crate::DbClient,
        role_id: i64,
        guild_id: i64,
        required: i16,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "INSERT INTO xproles (role_id, guild_id, required) VALUES ($1, $2, $3)
            ON CONFLICT DO NOTHING RETURNING *",
            role_id,
            guild_id,
            required,
        )
        .fetch_optional(&db.pool)
        .await
    }

    pub async fn delete(db: &crate::DbClient, role_id: i64) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "DELETE FROM xproles WHERE role_id=$1 RETURNING *",
            role_id
        )
        .fetch_optional(&db.pool)
        .await
    }

    pub async fn set_required(
        db: &crate::DbClient,
        role_id: i64,
        required: i16,
    ) -> sqlx::Result<Self> {
        sqlx::query_as!(
            Self,
            "UPDATE xproles SET required=$1 WHERE role_id=$2 RETURNING *",
            required,
            role_id
        )
        .fetch_one(&db.pool)
        .await
    }

    pub async fn list_by_guild(db: &crate::DbClient, guild_id: i64) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT * FROM xproles WHERE guild_id=$1 ORDER BY required DESC",
            guild_id,
        )
        .fetch_all(&db.pool)
        .await
    }

    pub async fn count(db: &crate::DbClient, guild_id: i64) -> sqlx::Result<i64> {
        Ok(
            sqlx::query!("SELECT count(*) FROM xproles WHERE guild_id=$1", guild_id)
                .fetch_one(&db.pool)
                .await?
                .count
                .unwrap(),
        )
    }
}
