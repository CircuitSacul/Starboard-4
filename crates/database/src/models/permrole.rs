#[derive(Debug)]
pub struct PermRole {
    pub role_id: i64,
    pub guild_id: i64,

    pub obtain_xproles: Option<bool>,
    pub give_votes: Option<bool>,
    pub receive_votes: Option<bool>,
}

#[cfg(feature = "backend")]
impl PermRole {
    pub async fn create(
        db: &crate::DbClient,
        role_id: i64,
        guild_id: i64,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "INSERT INTO permroles (role_id, guild_id) VALUES ($1, $2)
            ON CONFLICT DO NOTHING RETURNING *",
            role_id,
            guild_id,
        )
        .fetch_optional(&db.pool)
        .await
    }

    pub async fn delete(db: &crate::DbClient, role_id: i64) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "DELETE FROM permroles WHERE role_id=$1 RETURNING *",
            role_id
        )
        .fetch_optional(&db.pool)
        .await
    }

    pub async fn update(&self, db: &crate::DbClient) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"UPDATE permroles SET obtain_xproles=$1, give_votes=$2,
            receive_votes=$3 WHERE role_id=$4 RETURNING *"#,
            self.obtain_xproles,
            self.give_votes,
            self.receive_votes,
            self.role_id,
        )
        .fetch_optional(&db.pool)
        .await
    }

    pub async fn get(db: &crate::DbClient, role_id: i64) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM permroles WHERE role_id=$1", role_id)
            .fetch_optional(&db.pool)
            .await
    }

    pub async fn count_by_guild(db: &crate::DbClient, guild_id: i64) -> sqlx::Result<i64> {
        sqlx::query!(
            "SELECT COUNT(*) as count FROM permroles WHERE guild_id=$1",
            guild_id
        )
        .fetch_one(&db.pool)
        .await
        .map(|r| r.count.unwrap())
    }

    pub async fn list_by_guild(db: &crate::DbClient, guild_id: i64) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM permroles WHERE guild_id=$1", guild_id)
            .fetch_all(&db.pool)
            .await
    }
}
