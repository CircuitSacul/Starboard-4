#[derive(Debug)]
pub struct PermRole {
    pub role_id: i64,
    pub guild_id: i64,

    pub obtain_xproles: Option<bool>,
    pub give_votes: Option<bool>,
    pub receive_votes: Option<bool>,
}

impl PermRole {
    pub async fn create(pool: &sqlx::PgPool, role_id: i64, guild_id: i64) -> sqlx::Result<Self> {
        sqlx::query_as!(
            Self,
            r#"INSERT INTO permroles
            (role_id, guild_id)
            VALUES ($1, $2)
            RETURNING *"#,
            role_id,
            guild_id,
        )
        .fetch_one(pool)
        .await
    }

    pub async fn delete(pool: &sqlx::PgPool, role_id: i64) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "DELETE FROM permroles WHERE role_id=$1 RETURNING *",
            role_id
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn update(&self, pool: &sqlx::PgPool) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"UPDATE permroles SET obtain_xproles=$1, give_votes=$2,
            receive_votes=$3 WHERE role_id=$4 RETURNING *"#,
            self.obtain_xproles,
            self.give_votes,
            self.receive_votes,
            self.role_id,
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn get(pool: &sqlx::PgPool, role_id: i64) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM permroles WHERE role_id=$1", role_id)
            .fetch_optional(pool)
            .await
    }

    pub async fn list_by_guild(pool: &sqlx::PgPool, guild_id: i64) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM permroles WHERE guild_id=$1", guild_id)
            .fetch_all(pool)
            .await
    }
}
