#[cfg(feature = "backend")]
use sqlx::FromRow;

#[cfg(feature = "backend")]
use crate::helpers::query::build_update::build_update;

#[cfg_attr(feature = "backend", derive(FromRow))]
pub struct Filter {
    pub id: i32,

    pub filter_group_id: i32,
    pub position: i16,

    pub instant_pass: bool,
    pub instant_fail: bool,

    // default context
    pub user_has_all_of: Option<Vec<i64>>,
    pub user_has_some_of: Option<Vec<i64>>,
    pub user_missing_all_of: Option<Vec<i64>>,
    pub user_missing_some_of: Option<Vec<i64>>,
    pub user_is_bot: Option<bool>,

    // message context
    pub in_channel: Option<Vec<i64>>,
    pub not_in_channel: Option<Vec<i64>>,
    pub in_channel_or_sub_channels: Option<Vec<i64>>,
    pub not_in_channel_or_sub_channels: Option<Vec<i64>>,
    pub min_attachments: Option<i16>,
    pub max_attachments: Option<i16>,
    pub min_length: Option<i32>,
    pub max_length: Option<i32>,
    pub matches: Option<String>,
    pub not_matches: Option<String>,

    // vote context
    pub voter_has_all_of: Option<Vec<i64>>,
    pub voter_has_some_of: Option<Vec<i64>>,
    pub voter_missing_all_of: Option<Vec<i64>>,
    pub voter_missing_some_of: Option<Vec<i64>>,
    pub older_than: Option<i64>,
    pub newer_than: Option<i64>,
}

#[cfg(feature = "backend")]
impl Filter {
    pub async fn create(
        db: &crate::DbClient,
        filter_group_id: i32,
        position: i16,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "INSERT INTO filters (filter_group_id, position) VALUES ($1, $2)
            ON CONFLICT DO NOTHING RETURNING *",
            filter_group_id,
            position,
        )
        .fetch_optional(&db.pool)
        .await
    }

    pub async fn delete(
        db: &crate::DbClient,
        filter_group_id: i32,
        position: i16,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "DELETE FROM filters WHERE filter_group_id=$1 AND position=$2 RETURNING *",
            filter_group_id,
            position,
        )
        .fetch_optional(&db.pool)
        .await
    }

    pub async fn update_settings(self, db: &crate::DbClient) -> sqlx::Result<Option<Self>> {
        let mut builder = sqlx::QueryBuilder::<sqlx::Postgres>::new("UPDATE filters SET ");

        crate::call_with_filters_settings!(build_update, self, builder);

        builder
            .push(" WHERE id=")
            .push_bind(self.id)
            .push(" RETURNING *");

        let ret = builder.build().fetch_optional(&db.pool).await?;

        if let Some(row) = ret {
            Ok(Some(Filter::from_row(&row)?))
        } else {
            Ok(None)
        }
    }

    pub async fn get_last_position(
        db: &crate::DbClient,
        filter_group_id: i32,
    ) -> sqlx::Result<i16> {
        sqlx::query!(
            "SELECT MAX(position) as position FROM filters WHERE filter_group_id=$1",
            filter_group_id
        )
        .fetch_one(&db.pool)
        .await
        .map(|r| r.position.unwrap_or(0))
    }

    pub async fn get_by_position(
        db: &crate::DbClient,
        filter_group_id: i32,
        position: i16,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT * FROM filters WHERE filter_group_id=$1 AND position=$2",
            filter_group_id,
            position
        )
        .fetch_optional(&db.pool)
        .await
    }

    pub async fn shift(
        executor: impl sqlx::PgExecutor<'_>,
        filter_group_id: i32,
        start: i16,
        end: Option<i16>,
        distance: i16,
    ) -> sqlx::Result<()> {
        sqlx::query!(
            "UPDATE filters SET position = position + $1
            WHERE position >= $2 AND ($3::SMALLINT IS NULL OR position <= $3)
            AND filter_group_id=$4",
            distance,
            start,
            end,
            filter_group_id,
        )
        .execute(executor)
        .await?;

        Ok(())
    }

    pub async fn set_position(
        db: &crate::DbClient,
        filter_group_id: i32,
        current: i16,
        new: i16,
    ) -> sqlx::Result<Option<()>> {
        if current == new {
            return Ok(Some(()));
        }

        let mut tx = db.pool.begin().await?;

        // select for update, locking these rows
        sqlx::query!(
            "SELECT FROM filters WHERE filter_group_id=$1 FOR UPDATE",
            filter_group_id
        )
        .execute(&mut tx)
        .await?;

        // fetch the item that we're moving
        let to_move = Self::get_by_position(db, filter_group_id, current).await?;
        let Some(to_move) = to_move else {
            return Ok(None);
        };

        // shift other items out of the way if needed
        let (start, end, dir) = if current > new {
            (new, current, 1)
        } else {
            (current, new, -1)
        };
        Filter::shift(&mut tx, filter_group_id, start, Some(end), dir).await?;

        // update the position
        let ret = sqlx::query!(
            "UPDATE filters SET position=$1 WHERE id=$2 AND filter_group_id=$3",
            new,
            to_move.id,
            filter_group_id
        )
        .execute(&mut tx)
        .await?;

        // commit & return
        tx.commit().await?;

        if ret.rows_affected() == 0 {
            Ok(None)
        } else {
            Ok(Some(()))
        }
    }

    pub async fn list_by_filter(
        db: &crate::DbClient,
        filter_group_id: i32,
    ) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT * FROM filters WHERE filter_group_id=$1 ORDER BY position ASC",
            filter_group_id
        )
        .fetch_all(&db.pool)
        .await
    }
}
