#[cfg(feature = "backend")]
use futures::stream::BoxStream;

#[derive(Debug)]
pub struct DbMember {
    pub user_id: i64,
    pub guild_id: i64,
    pub xp: f32,
    pub autoredeem_enabled: bool,
}

#[cfg(feature = "backend")]
impl DbMember {
    pub async fn create(
        db: &crate::DbClient,
        user_id: i64,
        guild_id: i64,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "INSERT INTO members (user_id, guild_id) VALUES ($1, $2)
            ON CONFLICT DO NOTHING RETURNING *",
            user_id,
            guild_id
        )
        .fetch_optional(&db.pool)
        .await
    }

    pub async fn set_xp(
        db: &crate::DbClient,
        user_id: i64,
        guild_id: i64,
        xp: f32,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "UPDATE members SET xp=$1 WHERE user_id=$2 AND guild_id=$3 RETURNING *",
            xp,
            user_id,
            guild_id
        )
        .fetch_optional(&db.pool)
        .await
    }

    pub async fn set_autoredeem_enabled(
        db: &crate::DbClient,
        user_id: i64,
        guild_id: i64,
        autoredeem_enabled: bool,
    ) -> sqlx::Result<()> {
        sqlx::query!(
            "UPDATE members SET autoredeem_enabled=$1 WHERE user_id=$2 AND guild_id=$3",
            autoredeem_enabled,
            user_id,
            guild_id,
        )
        .fetch_all(&db.pool)
        .await?;

        Ok(())
    }

    pub fn stream_by_xp(db: &crate::DbClient, guild_id: i64) -> BoxStream<'_, sqlx::Result<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT * FROM members WHERE guild_id=$1 AND xp > 0 ORDER BY xp DESC",
            guild_id
        )
        .fetch(&db.pool)
    }

    pub async fn list_by_xp(
        db: &crate::DbClient,
        guild_id: i64,
        limit: i64,
    ) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT * FROM members WHERE guild_id=$1 AND xp > 0 ORDER BY xp DESC LIMIT $2",
            guild_id,
            limit,
        )
        .fetch_all(&db.pool)
        .await
    }

    pub async fn list_autoredeem_by_user(
        db: &crate::DbClient,
        user_id: i64,
    ) -> sqlx::Result<Vec<i64>> {
        let rows = sqlx::query!(
            "SELECT guild_id FROM members WHERE user_id=$1 AND autoredeem_enabled=true",
            user_id
        )
        .fetch_all(&db.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.guild_id).collect())
    }

    pub async fn get(
        db: &crate::DbClient,
        guild_id: i64,
        user_id: i64,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT * FROM members WHERE guild_id=$1 AND user_id=$2",
            guild_id,
            user_id,
        )
        .fetch_optional(&db.pool)
        .await
    }
}
