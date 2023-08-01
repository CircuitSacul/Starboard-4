use chrono::{DateTime, Utc};

use crate::DbClient;

#[derive(Debug)]
pub struct DbGuild {
    pub guild_id: i64,
    pub premium_end: Option<DateTime<Utc>>,
}

#[cfg(feature = "backend")]
impl DbGuild {
    pub async fn create(db: &DbClient, guild_id: i64) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "INSERT INTO guilds (guild_id) VALUES ($1) ON CONFLICT DO NOTHING RETURNING *",
            guild_id
        )
        .fetch_optional(&db.pool)
        .await
    }

    pub async fn get(db: &DbClient, guild_id: i64) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM guilds WHERE guild_id=$1", guild_id)
            .fetch_optional(&db.pool)
            .await
    }
}
