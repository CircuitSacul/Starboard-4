use twilight_model::id::{
    marker::{GuildMarker, UserMarker},
    Id,
};

use database::{DbClient, DbMember, Starboard};
use errors::StarboardResult;

use crate::{client::bot::StarboardBot, utils::id_as_i64::GetI64};

use super::{premium::is_premium::is_guild_premium, xproles::refresh_xpr};

#[derive(Default)]
pub struct MemberStats {
    pub xp: f32,
    pub given_upvotes: i64,
    pub given_downvotes: i64,
    pub received_upvotes: i64,
    pub received_downvotes: i64,
}

impl MemberStats {
    pub async fn get(db: &DbClient, guild_id: i64, user_id: i64) -> StarboardResult<Option<Self>> {
        let mut stats = Self::default();

        let starboards = Starboard::list_by_guild(db, guild_id).await?;
        if starboards.is_empty() {
            return Ok(None);
        }

        for sb in starboards {
            if sb.settings.private {
                continue;
            }

            let given_upvotes = Self::given_upvotes(db, user_id, sb.id).await?;
            let given_downvotes = Self::given_downvotes(db, user_id, sb.id).await?;
            let received_upvotes = Self::received_upvotes(db, user_id, sb.id).await?;
            let received_downvotes = Self::received_downvotes(db, user_id, sb.id).await?;

            stats.given_upvotes += given_upvotes;
            stats.given_downvotes += given_downvotes;
            stats.received_upvotes += received_upvotes;
            stats.received_downvotes += received_downvotes;

            stats.xp += (received_upvotes - received_downvotes) as f32 * sb.settings.xp_multiplier;
        }

        Ok(Some(stats))
    }

    async fn given_downvotes(
        db: &DbClient,
        user_id: i64,
        starboard_id: i32,
    ) -> StarboardResult<i64> {
        Ok(sqlx::query!(
            "SELECT count(*) FROM votes WHERE starboard_id=$1 AND user_id=$2
            AND is_downvote=true",
            starboard_id,
            user_id,
        )
        .fetch_one(&db.pool)
        .await?
        .count
        .unwrap())
    }

    async fn given_upvotes(db: &DbClient, user_id: i64, starboard_id: i32) -> StarboardResult<i64> {
        Ok(sqlx::query!(
            "SELECT count(*) FROM votes WHERE starboard_id=$1 AND user_id=$2
            AND is_downvote=false",
            starboard_id,
            user_id,
        )
        .fetch_one(&db.pool)
        .await?
        .count
        .unwrap())
    }

    async fn received_downvotes(
        db: &DbClient,
        user_id: i64,
        starboard_id: i32,
    ) -> StarboardResult<i64> {
        Ok(sqlx::query!(
            "SELECT count(*) FROM votes WHERE starboard_id=$1
            AND target_author_id=$2 AND is_downvote=true",
            starboard_id,
            user_id,
        )
        .fetch_one(&db.pool)
        .await?
        .count
        .unwrap())
    }

    async fn received_upvotes(
        db: &DbClient,
        user_id: i64,
        starboard_id: i32,
    ) -> StarboardResult<i64> {
        Ok(sqlx::query!(
            "SELECT count(*) FROM votes WHERE starboard_id=$1
            AND target_author_id=$2 AND is_downvote=false",
            starboard_id,
            user_id,
        )
        .fetch_one(&db.pool)
        .await?
        .count
        .unwrap())
    }
}

pub async fn refresh_xp(
    bot: &StarboardBot,
    guild_id: Id<GuildMarker>,
    user_id: Id<UserMarker>,
) -> StarboardResult<()> {
    if bot
        .cooldowns
        .xp_refresh
        .trigger(&(user_id, guild_id))
        .is_some()
    {
        return Ok(());
    }

    let Some(stats) = MemberStats::get(&bot.db, guild_id.get_i64(), user_id.get_i64()).await?
    else {
        return Ok(());
    };

    DbMember::set_xp(&bot.db, user_id.get_i64(), guild_id.get_i64(), stats.xp).await?;

    if is_guild_premium(bot, guild_id.get_i64(), true).await? {
        refresh_xpr(bot, guild_id, user_id).await?;
    }

    Ok(())
}
