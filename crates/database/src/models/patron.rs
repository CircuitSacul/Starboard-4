#[derive(Debug)]
pub struct Patron {
    pub patreon_id: String,
    pub discord_id: Option<i64>,
    pub last_patreon_total_cents: i64,
}

#[cfg(feature = "backend")]
impl Patron {
    pub async fn create(db: &crate::DbClient, patreon_id: &str) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "INSERT INTO patrons (patreon_id) VALUES ($1)
            ON CONFLICT DO NOTHING RETURNING *",
            patreon_id,
        )
        .fetch_optional(&db.pool)
        .await
    }

    pub async fn get(db: &crate::DbClient, patreon_id: &str) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT * FROM patrons WHERE patreon_id=$1",
            patreon_id,
        )
        .fetch_optional(&db.pool)
        .await
    }

    pub async fn get_by_user(db: &crate::DbClient, user_id: i64) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM patrons WHERE discord_id=$1", user_id)
            .fetch_all(&db.pool)
            .await
    }

    pub async fn set_discord_id(
        db: &crate::DbClient,
        patreon_id: &str,
        discord_id: Option<i64>,
    ) -> sqlx::Result<()> {
        sqlx::query!(
            "UPDATE patrons SET discord_id=$1 WHERE patreon_id=$2",
            discord_id,
            patreon_id
        )
        .fetch_all(&db.pool)
        .await?;
        Ok(())
    }

    pub async fn set_total_cents(
        db: &crate::DbClient,
        patreon_id: &str,
        total_cents: i64,
    ) -> sqlx::Result<()> {
        sqlx::query!(
            "UPDATE patrons SET last_patreon_total_cents=$1 WHERE patreon_id=$2",
            total_cents,
            patreon_id
        )
        .fetch_all(&db.pool)
        .await?;
        Ok(())
    }
}
