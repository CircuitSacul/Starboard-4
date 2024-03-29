use crate::database::StarboardMessage;

#[derive(Debug)]
pub struct DbMessage {
    pub message_id: i64,
    pub guild_id: i64,
    pub channel_id: i64,
    pub author_id: i64,

    pub is_nsfw: bool,

    pub forced_to: Vec<i32>,
    pub trashed: bool,
    pub trash_reason: Option<String>,
    pub frozen: bool,
}

impl DbMessage {
    pub async fn create(
        pool: &sqlx::PgPool,
        message_id: i64,
        guild_id: i64,
        channel_id: i64,
        author_id: i64,
        is_nsfw: bool,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"INSERT INTO messages (message_id, guild_id, channel_id, author_id, is_nsfw)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT DO NOTHING RETURNING *"#,
            message_id,
            guild_id,
            channel_id,
            author_id,
            is_nsfw,
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn set_freeze(
        pool: &sqlx::PgPool,
        message_id: i64,
        frozen: bool,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "UPDATE messages SET frozen=$1 WHERE message_id=$2 RETURNING *",
            frozen,
            message_id,
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn set_forced(
        pool: &sqlx::PgPool,
        message_id: i64,
        forced: &[i32],
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "UPDATE messages SET forced_to=$1 WHERE message_id=$2 RETURNING *",
            forced,
            message_id,
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn set_trashed(
        pool: &sqlx::PgPool,
        message_id: i64,
        trashed: bool,
        reason: Option<&str>,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "UPDATE messages SET trashed=$1, trash_reason=$2 WHERE message_id=$3 RETURNING *",
            trashed,
            reason,
            message_id,
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn list_trashed(pool: &sqlx::PgPool, guild_id: i64) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT * FROM messages WHERE guild_id=$1 AND trashed=true ORDER BY message_id",
            guild_id
        )
        .fetch_all(pool)
        .await
    }

    pub async fn get_original(pool: &sqlx::PgPool, message_id: i64) -> sqlx::Result<Option<Self>> {
        let orig = if let Some(sb_msg) = StarboardMessage::get(pool, message_id).await? {
            sb_msg.message_id
        } else {
            message_id
        };

        sqlx::query_as!(Self, "SELECT * FROM messages WHERE message_id=$1", orig)
            .fetch_optional(pool)
            .await
    }

    pub async fn get(pool: &sqlx::PgPool, message_id: i64) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT * FROM messages WHERE message_id=$1",
            message_id,
        )
        .fetch_optional(pool)
        .await
    }
}
