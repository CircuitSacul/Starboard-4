use crate::{concat_format, constants, database::OverrideValues};

#[derive(Debug)]
pub struct StarboardOverride {
    // serial
    pub id: i32,
    pub guild_id: i64,
    pub name: String,

    pub starboard_id: i32,
    pub channel_ids: Vec<i64>,

    pub overrides: serde_json::Value,
}

impl StarboardOverride {
    pub async fn create(
        pool: &sqlx::PgPool,
        guild_id: i64,
        name: &String,
        starboard_id: i32,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "INSERT INTO overrides (guild_id, name, starboard_id) VALUES ($1, $2, $3)
            ON CONFLICT DO NOTHING RETURNING *",
            guild_id,
            name,
            starboard_id,
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn delete(
        pool: &sqlx::PgPool,
        guild_id: i64,
        name: &String,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "DELETE FROM overrides WHERE guild_id=$1 AND name=$2 RETURNING *",
            guild_id,
            name,
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn rename(
        pool: &sqlx::PgPool,
        guild_id: i64,
        old_name: &str,
        new_name: &str,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "UPDATE overrides SET name=$1 WHERE name=$2 AND guild_id=$3 RETURNING *",
            new_name,
            old_name,
            guild_id,
        )
        .fetch_optional(pool)
        .await
    }

    pub fn validate_channels(channel_ids: &[i64]) -> Result<(), String> {
        if channel_ids.len() > constants::MAX_CHANNELS_PER_OVERRIDE {
            Err(concat_format!(
                "You can only have up to {}" <- constants::MAX_CHANNELS_PER_OVERRIDE;
                " channels per override.\n";
                "Tip: You can override the settings for an entire category if you include its ID.";

            ))
        } else {
            Ok(())
        }
    }

    pub async fn set_channels(
        pool: &sqlx::PgPool,
        guild_id: i64,
        name: &str,
        channel_ids: &[i64],
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "UPDATE overrides SET channel_ids=$1 WHERE name=$2 AND guild_id=$3 RETURNING *",
            channel_ids,
            name,
            guild_id,
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn update_settings(
        pool: &sqlx::PgPool,
        id: i32,
        settings: OverrideValues,
    ) -> sqlx::Result<Option<Self>> {
        let settings = serde_json::to_value(&settings).unwrap();
        Self::update_settings_raw(pool, id, settings).await
    }

    pub async fn update_settings_raw(
        pool: &sqlx::PgPool,
        id: i32,
        settings: serde_json::Value,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "UPDATE overrides SET overrides=$1 WHERE id=$2 RETURNING *",
            settings,
            id
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn get(pool: &sqlx::PgPool, guild_id: i64, name: &str) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT * FROM overrides WHERE guild_id=$1 AND name=$2",
            guild_id,
            name
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn list_by_guild(pool: &sqlx::PgPool, guild_id: i64) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM overrides WHERE guild_id=$1", guild_id)
            .fetch_all(pool)
            .await
    }

    pub async fn list_by_starboard_and_channels(
        pool: &sqlx::PgPool,
        starboard_id: i32,
        channel_ids: &[i64],
    ) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT * FROM overrides WHERE starboard_id=$1 AND channel_ids && $2::bigint[]",
            starboard_id,
            channel_ids,
        )
        .fetch_all(pool)
        .await
    }

    pub async fn count_by_starboard(pool: &sqlx::PgPool, starboard_id: i32) -> sqlx::Result<i64> {
        sqlx::query!(
            "SELECT COUNT(*) as count FROM overrides WHERE starboard_id=$1",
            starboard_id
        )
        .fetch_one(pool)
        .await
        .map(|r| r.count.unwrap())
    }

    pub async fn list_by_starboard(
        pool: &sqlx::PgPool,
        starboard_id: i32,
    ) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT * FROM overrides WHERE starboard_id=$1",
            starboard_id,
        )
        .fetch_all(pool)
        .await
    }

    pub fn get_overrides(&self) -> serde_json::Result<OverrideValues> {
        serde_json::from_value(self.overrides.clone())
    }
}
