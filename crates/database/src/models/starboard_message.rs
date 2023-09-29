#[derive(Debug)]
#[cfg_attr(feature = "backend", derive(sqlx::FromRow))]
pub struct StarboardMessage {
    pub message_id: i64,
    pub starboard_id: i32,
    pub starboard_message_id: i64,
    pub last_known_point_count: i16,
}

#[cfg(feature = "backend")]
impl StarboardMessage {
    pub async fn create(
        db: &crate::DbClient,
        message_id: i64,
        starboard_message_id: i64,
        starboard_id: i32,
        last_known_point_count: i32,
    ) -> sqlx::Result<Option<Self>> {
        let point_count = last_known_point_count as i16;
        sqlx::query_as!(
            Self,
            "INSERT INTO starboard_messages
            (message_id, starboard_id, starboard_message_id, last_known_point_count)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT DO NOTHING RETURNING *",
            message_id,
            starboard_id,
            starboard_message_id,
            point_count,
        )
        .fetch_optional(&db.pool)
        .await
    }

    pub async fn delete(
        db: &crate::DbClient,
        starboard_message_id: i64,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "DELETE FROM starboard_messages WHERE starboard_message_id=$1
            RETURNING *",
            starboard_message_id,
        )
        .fetch_optional(&db.pool)
        .await
    }

    pub async fn get(
        db: &crate::DbClient,
        starboard_message_id: i64,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT * FROM starboard_messages WHERE starboard_message_id=$1",
            starboard_message_id,
        )
        .fetch_optional(&db.pool)
        .await
    }

    pub async fn get_by_starboard(
        db: &crate::DbClient,
        message_id: i64,
        starboard_id: i32,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT * FROM starboard_messages WHERE starboard_id=$1 AND message_id=$2",
            starboard_id,
            message_id,
        )
        .fetch_optional(&db.pool)
        .await
    }

    pub async fn set_last_point_count(
        db: &crate::DbClient,
        starboard_message_id: i64,
        point_count: i16,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "UPDATE starboard_messages SET last_known_point_count=$1 WHERE starboard_message_id=$2
            RETURNING *",
            point_count,
            starboard_message_id,
        )
        .fetch_optional(&db.pool)
        .await
    }
}
