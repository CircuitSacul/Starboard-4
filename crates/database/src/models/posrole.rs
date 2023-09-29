#[derive(Debug)]
pub struct PosRole {
    pub role_id: i64,
    pub guild_id: i64,
    pub max_members: i32,
}

#[cfg(feature = "backend")]
impl PosRole {
    pub async fn create(
        db: &crate::DbClient,
        role_id: i64,
        guild_id: i64,
        max_members: i32,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "INSERT INTO posroles (role_id, guild_id, max_members) VALUES ($1, $2, $3)
            ON CONFLICT DO NOTHING RETURNING *",
            role_id,
            guild_id,
            max_members,
        )
        .fetch_optional(&db.pool)
        .await
    }

    pub async fn set_max_members(
        db: &crate::DbClient,
        role_id: i64,
        max_members: i32,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "UPDATE posroles SET max_members=$1 WHERE role_id=$2 RETURNING *",
            max_members,
            role_id,
        )
        .fetch_optional(&db.pool)
        .await
    }

    pub async fn delete(db: &crate::DbClient, role_id: i64) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "DELETE FROM posroles WHERE role_id=$1 RETURNING *",
            role_id,
        )
        .fetch_optional(&db.pool)
        .await
    }

    pub async fn list_by_guild(db: &crate::DbClient, guild_id: i64) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT * FROM posroles WHERE guild_id=$1 ORDER BY max_members ASC",
            guild_id,
        )
        .fetch_all(&db.pool)
        .await
    }

    pub async fn count(db: &crate::DbClient, guild_id: i64) -> sqlx::Result<i64> {
        Ok(
            sqlx::query!("SELECT count(*) FROM posroles WHERE guild_id=$1", guild_id)
                .fetch_one(&db.pool)
                .await?
                .count
                .unwrap(),
        )
    }
}
