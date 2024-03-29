#[derive(Debug)]
pub struct PermRoleStarboard {
    pub permrole_id: i64,
    pub starboard_id: i32,

    pub give_votes: Option<bool>,
    pub receive_votes: Option<bool>,
}

impl PermRoleStarboard {
    pub async fn create(
        pool: &sqlx::PgPool,
        permrole_id: i64,
        starboard_id: i32,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "INSERT INTO permrole_starboards (permrole_id, starboard_id) VALUES ($1, $2)
            ON CONFLICT DO NOTHING RETURNING *",
            permrole_id,
            starboard_id,
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn delete(
        pool: &sqlx::PgPool,
        permrole_id: i64,
        starboard_id: i32,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "DELETE FROM permrole_starboards WHERE permrole_id=$1 AND starboard_id=$2 RETURNING *",
            permrole_id,
            starboard_id,
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn update(&self, pool: &sqlx::PgPool) -> sqlx::Result<Option<Self>> {
        if self.give_votes.is_none() && self.receive_votes.is_none() {
            return Self::delete(pool, self.permrole_id, self.starboard_id).await;
        }

        sqlx::query_as!(
            Self,
            r#"UPDATE permrole_starboards SET give_votes=$1, receive_votes=$2 WHERE permrole_id=$3
            AND starboard_id=$4 RETURNING *"#,
            self.give_votes,
            self.receive_votes,
            self.permrole_id,
            self.starboard_id,
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn get(
        pool: &sqlx::PgPool,
        permrole_id: i64,
        starboard_id: i32,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT * FROM permrole_starboards WHERE permrole_id=$1 AND starboard_id=$2",
            permrole_id,
            starboard_id
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn list_by_permrole(
        pool: &sqlx::PgPool,
        permrole_id: i64,
    ) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT * FROM permrole_starboards WHERE permrole_id=$1",
            permrole_id,
        )
        .fetch_all(pool)
        .await
    }
}
