#[cfg(feature = "backend")]
use crate::{
    call_with_starboard_settings, helpers::query::build_update::build_update,
    starboard_from_record, starboard_from_row, DbClient,
};

#[derive(Debug, Clone)]
pub struct Starboard {
    pub id: i32,
    pub name: String,
    pub channel_id: i64,
    pub guild_id: i64,

    pub webhook_id: Option<i64>,
    pub premium_locked: bool,

    pub settings: crate::StarboardSettings,
}

#[cfg(feature = "backend")]
impl Starboard {
    pub async fn create(
        db: &DbClient,
        name: &String,
        channel_id: i64,
        guild_id: i64,
    ) -> sqlx::Result<Option<Self>> {
        let starboard = sqlx::query!(
            "INSERT INTO STARBOARDS (name, channel_id, guild_id) VALUES ($1, $2, $3)
            ON CONFLICT DO NOTHING RETURNING *",
            name,
            channel_id,
            guild_id,
        )
        .fetch_optional(&db.pool)
        .await?;

        if let Some(row) = starboard {
            Ok(Some(starboard_from_record!(row)))
        } else {
            Ok(None)
        }
    }

    pub async fn delete(db: &DbClient, name: &String, guild_id: i64) -> sqlx::Result<Option<Self>> {
        sqlx::query!(
            "DELETE FROM starboards WHERE name=$1 AND guild_id=$2 RETURNING *",
            name,
            guild_id,
        )
        .fetch_optional(&db.pool)
        .await
        .map(|row| row.map(|row| starboard_from_record!(row)))
    }

    pub async fn update_settings(self, db: &DbClient) -> sqlx::Result<Option<Self>> {
        let mut builder = sqlx::QueryBuilder::<sqlx::Postgres>::new("UPDATE starboards SET ");

        call_with_starboard_settings!(build_update, self.settings, builder);

        builder
            .push(" WHERE name=")
            .push_bind(self.name)
            .push(" AND guild_id=")
            .push_bind(self.guild_id)
            .push(" RETURNING *");

        builder
            .build()
            .fetch_optional(&db.pool)
            .await
            .map(|r| r.map(|r| starboard_from_row!(r)))
    }

    pub async fn set_webhook(
        db: &DbClient,
        starboard_id: i32,
        webhook_id: Option<i64>,
    ) -> sqlx::Result<()> {
        sqlx::query!(
            "UPDATE starboards SET webhook_id=$1 WHERE id=$2",
            webhook_id,
            starboard_id
        )
        .execute(&db.pool)
        .await
        .map(|_| ())
    }

    pub async fn disable_webhooks(db: &DbClient, starboard_id: i32) -> sqlx::Result<()> {
        sqlx::query!(
            "UPDATE starboards SET use_webhook=false WHERE id=$1",
            starboard_id
        )
        .execute(&db.pool)
        .await
        .map(|_| ())
    }

    pub async fn rename(
        db: &DbClient,
        name: &String,
        guild_id: i64,
        new_name: &String,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query!(
            "UPDATE starboards SET name=$1 WHERE name=$2 AND guild_id=$3
            RETURNING *",
            new_name,
            name,
            guild_id,
        )
        .fetch_optional(&db.pool)
        .await
        .map(|r| r.map(|r| starboard_from_record!(r)))
    }

    pub async fn get(db: &DbClient, id: i32) -> sqlx::Result<Option<Self>> {
        let result = sqlx::query!("SELECT * FROM starboards WHERE id=$1", id)
            .fetch_optional(&db.pool)
            .await?;

        Ok(result.map(|sb| starboard_from_record!(sb)))
    }

    pub async fn get_by_name(
        db: &DbClient,
        name: &str,
        guild_id: i64,
    ) -> sqlx::Result<Option<Self>> {
        let result = sqlx::query!(
            "SELECT * FROM starboards WHERE name=$1 AND guild_id=$2",
            name,
            guild_id
        )
        .fetch_optional(&db.pool)
        .await?;

        Ok(result.map(|sb| starboard_from_record!(sb)))
    }

    pub async fn get_by_name_for_update(
        e: impl sqlx::PgExecutor<'_>,
        name: &str,
        guild_id: i64,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query!(
            "SELECT * FROM starboards WHERE guild_id=$1 AND name=$2 FOR UPDATE",
            guild_id,
            name
        )
        .fetch_optional(e)
        .await
        .map(|r| r.map(|r| starboard_from_record!(r)))
    }

    pub async fn count_by_guild(db: &DbClient, guild_id: i64) -> sqlx::Result<i64> {
        sqlx::query!(
            "SELECT COUNT(*) as count FROM starboards WHERE guild_id=$1",
            guild_id
        )
        .fetch_one(&db.pool)
        .await
        .map(|r| r.count.unwrap())
    }

    pub async fn list_by_guild(db: &DbClient, guild_id: i64) -> sqlx::Result<Vec<Self>> {
        sqlx::query!("SELECT * FROM starboards WHERE guild_id=$1", guild_id,)
            .fetch_all(&db.pool)
            .await
            .map(|rows| {
                rows.into_iter()
                    .map(|row| starboard_from_record!(row))
                    .collect()
            })
    }
}
