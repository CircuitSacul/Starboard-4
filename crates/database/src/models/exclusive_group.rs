pub struct ExclusiveGroup {
    pub id: i32,
    pub guild_id: i64,
    pub name: String,
}

#[cfg(feature = "backend")]
impl ExclusiveGroup {
    pub async fn create(
        db: &crate::DbClient,
        name: &str,
        guild_id: i64,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "INSERT INTO exclusive_groups (name, guild_id) VALUES ($1, $2)
            ON CONFLICT DO NOTHING RETURNING *",
            name,
            guild_id
        )
        .fetch_optional(&db.pool)
        .await
    }

    pub async fn delete(
        db: &crate::DbClient,
        name: &str,
        guild_id: i64,
    ) -> sqlx::Result<Option<Self>> {
        let ret = sqlx::query_as!(
            Self,
            "DELETE FROM exclusive_groups WHERE guild_id=$1 AND name=$2 RETURNING *",
            guild_id,
            name
        )
        .fetch_optional(&db.pool)
        .await?;

        if let Some(group) = &ret {
            sqlx::query!(
                "UPDATE overrides SET overrides = (overrides::jsonb - 'exclusive_group')::json
                WHERE guild_id=$1 AND (overrides::jsonb->'exclusive_group')::int=$2",
                guild_id,
                group.id,
            )
            .fetch_all(&db.pool)
            .await?;
        }

        Ok(ret)
    }

    pub async fn rename(
        db: &crate::DbClient,
        guild_id: i64,
        old_name: &str,
        new_name: &str,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "UPDATE exclusive_groups SET name=$1 WHERE guild_id=$2 AND name=$3 RETURNING *",
            new_name,
            guild_id,
            old_name
        )
        .fetch_optional(&db.pool)
        .await
    }

    pub async fn get(db: &crate::DbClient, id: i32) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM exclusive_groups WHERE id=$1", id)
            .fetch_optional(&db.pool)
            .await
    }

    pub async fn get_by_name(
        db: &crate::DbClient,
        guild_id: i64,
        name: &str,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT * FROM exclusive_groups WHERE guild_id=$1 AND name=$2",
            guild_id,
            name
        )
        .fetch_optional(&db.pool)
        .await
    }

    pub async fn list_by_guild(db: &crate::DbClient, guild_id: i64) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT * FROM exclusive_groups WHERE guild_id=$1",
            guild_id
        )
        .fetch_all(&db.pool)
        .await
    }

    pub async fn count_by_guild(db: &crate::DbClient, guild_id: i64) -> sqlx::Result<i64> {
        sqlx::query!(
            "SELECT count(*) as count FROM exclusive_groups WHERE guild_id=$1",
            guild_id
        )
        .fetch_one(&db.pool)
        .await
        .map(|r| r.count.unwrap())
    }
}
