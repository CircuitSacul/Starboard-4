pub struct StarboardFilterGroup {
    pub filter_group_id: i32,
    pub starboard_id: i32,
}

impl StarboardFilterGroup {
    pub async fn create(
        pool: &sqlx::PgPool,
        filter_group_id: i32,
        starboard_id: i32,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "INSERT INTO starboard_filter_groups (filter_group_id, starboard_id) VALUES ($1, $2)
            ON CONFLICT DO NOTHING RETURNING *",
            filter_group_id,
            starboard_id
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn delete(
        pool: &sqlx::PgPool,
        filter_group_id: i32,
        starboard_id: i32,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "DELETE FROM starboard_filter_groups WHERE filter_group_id=$1 AND starboard_id=$2
            RETURNING *",
            filter_group_id,
            starboard_id
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn list_by_starboard(
        pool: &sqlx::PgPool,
        starboard_id: i32,
    ) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT * FROM starboard_filter_groups WHERE starboard_id=$1",
            starboard_id
        )
        .fetch_all(pool)
        .await
    }
}
