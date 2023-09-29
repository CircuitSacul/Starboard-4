#[derive(Debug)]
pub struct Vote {
    pub message_id: i64,
    pub starboard_id: i32,
    pub user_id: i64,

    pub target_author_id: i64,
    pub is_downvote: bool,
}

#[cfg(feature = "backend")]
impl Vote {
    pub async fn create(
        db: &crate::DbClient,
        message_id: i64,
        starboard_id: i32,
        user_id: i64,
        target_author_id: i64,
        is_downvote: bool,
    ) -> sqlx::Result<Option<()>> {
        let create = sqlx::query!(
            "INSERT INTO VOTES (message_id, starboard_id, user_id, target_author_id, is_downvote)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT DO NOTHING",
            message_id,
            starboard_id,
            user_id,
            target_author_id,
            is_downvote,
        )
        .fetch_optional(&db.pool)
        .await?;

        if create.is_some() {
            return Ok(Some(()));
        }

        sqlx::query!(
            "UPDATE votes SET is_downvote=$1 WHERE message_id=$2 AND starboard_id=$3 AND user_id=$4",
            is_downvote,
            message_id,
            starboard_id,
            user_id,
        )
        .fetch_optional(&db.pool)
        .await?;

        Ok(Some(()))
    }

    pub async fn count(
        db: &crate::DbClient,
        message_id: i64,
        starboard_id: i32,
    ) -> sqlx::Result<i32> {
        let upvotes = sqlx::query!(
            "SELECT COUNT(*) as count FROM votes WHERE message_id=$1 AND starboard_id=$2
            AND is_downvote=false",
            message_id,
            starboard_id
        )
        .fetch_one(&db.pool)
        .await?;
        let downvotes = sqlx::query!(
            "SELECT COUNT(*) as count FROM votes WHERE message_id=$1 AND starboard_id=$2
            AND is_downvote=true",
            message_id,
            starboard_id
        )
        .fetch_one(&db.pool)
        .await?;

        Ok({ upvotes.count.unwrap() - downvotes.count.unwrap() }
            .try_into()
            .unwrap())
    }

    pub async fn delete(
        db: &crate::DbClient,
        message_id: i64,
        starboard_id: i32,
        user_id: i64,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "DELETE FROM votes WHERE message_id=$1 AND starboard_id=$2 AND user_id=$3
            RETURNING *",
            message_id,
            starboard_id,
            user_id,
        )
        .fetch_optional(&db.pool)
        .await
    }
}
