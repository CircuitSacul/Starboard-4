#[derive(Debug)]
pub struct DbUser {
    pub user_id: i64,
    pub is_bot: bool,
    pub credits: i32,
    pub donated_cents: i64,
    /// 0=none, 1=active, 2=declined, 3=former
    pub patreon_status: i16,
}

#[cfg(feature = "backend")]
impl DbUser {
    pub async fn create(
        db: &crate::DbClient,
        user_id: i64,
        is_bot: bool,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "INSERT INTO users (user_id, is_bot) VALUES ($1, $2)
            ON CONFLICT DO NOTHING RETURNING *",
            user_id,
            is_bot,
        )
        .fetch_optional(&db.pool)
        .await
    }

    pub async fn get(db: &crate::DbClient, user_id: i64) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM users WHERE user_id=$1", user_id)
            .fetch_optional(&db.pool)
            .await
    }

    pub async fn add_credits(db: &crate::DbClient, user_id: i64, credits: i32) -> sqlx::Result<()> {
        sqlx::query!(
            "UPDATE users SET credits = credits + $1 WHERE user_id=$2",
            credits,
            user_id
        )
        .fetch_all(&db.pool)
        .await?;
        Ok(())
    }

    pub async fn set_patreon_status(
        db: &crate::DbClient,
        user_id: i64,
        patreon_status: i16,
    ) -> sqlx::Result<()> {
        sqlx::query!(
            "UPDATE users SET patreon_status=$1 WHERE user_id=$2",
            patreon_status,
            user_id
        )
        .fetch_all(&db.pool)
        .await?;
        Ok(())
    }
}
