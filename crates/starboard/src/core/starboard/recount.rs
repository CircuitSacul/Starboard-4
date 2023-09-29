use std::{sync::Arc, time::Duration};

use twilight_model::id::{
    marker::{ChannelMarker, GuildMarker, MessageMarker},
    Id,
};

use database::{DbMember, DbMessage, DbUser, Vote};
use errors::StarboardResult;

use crate::{
    client::bot::StarboardBot,
    core::{emoji::SimpleEmoji, premium::is_premium::is_guild_premium},
    utils::{id_as_i64::GetI64, into_id::IntoId},
};

use super::{
    config::StarboardConfig,
    handle::RefreshMessage,
    message::get_or_create_original,
    vote_status::{VoteContext, VoteStatus},
};

pub enum RecountResult {
    Cooldown(Duration),
    AlreadyRecounting,
    UnkownMessage,
    Done,
}

pub async fn recount_votes(
    bot: Arc<StarboardBot>,
    guild_id: Id<GuildMarker>,
    channel_id: Id<ChannelMarker>,
    message_id: Id<MessageMarker>,
) -> StarboardResult<RecountResult> {
    let Some(_guard) = bot.locks.vote_recount.lock(message_id) else {
        return Ok(RecountResult::AlreadyRecounting);
    };
    if let Some(retry) = bot.cooldowns.vote_recount.trigger(&guild_id) {
        return Ok(RecountResult::Cooldown(retry));
    }

    let orig = get_or_create_original(&bot, guild_id, channel_id, message_id).await?;
    let (Some(orig), author_is_bot) = orig else {
        return Ok(RecountResult::UnkownMessage);
    };
    let author_is_bot = match author_is_bot {
        Some(val) => val,
        None => DbUser::get(&bot.db, orig.author_id).await?.unwrap().is_bot,
    };

    let configs =
        StarboardConfig::list_for_channel(&bot, guild_id, orig.channel_id.into_id()).await?;

    let guild_id_i64 = guild_id.get_i64();

    let orig_obj = bot
        .http
        .message(channel_id, message_id)
        .await?
        .model()
        .await?;
    for reaction in orig_obj.reactions {
        let emoji = SimpleEmoji::from(reaction.emoji);

        let is_vote = StarboardConfig::is_guild_vote_emoji(&bot, guild_id_i64, &emoji).await?;

        if is_vote {
            recount_votes_reaction(
                &bot,
                (channel_id, message_id),
                &orig,
                &configs,
                guild_id,
                author_is_bot,
                emoji,
            )
            .await?;
        }
    }

    let is_premium = is_guild_premium(&bot, guild_id_i64, true).await?;
    let mut refresh = RefreshMessage::new(bot.clone(), message_id, is_premium);
    refresh.set_sql_message(orig);
    refresh.refresh(false).await?;

    Ok(RecountResult::Done)
}

async fn recount_votes_reaction(
    bot: &StarboardBot,
    refreshing: (Id<ChannelMarker>, Id<MessageMarker>),
    orig: &DbMessage,
    configs: &[StarboardConfig],
    guild_id: Id<GuildMarker>,
    author_is_bot: bool,
    emoji: SimpleEmoji,
) -> StarboardResult<()> {
    let mut last_user = None;
    let reactable = emoji.reactable();
    loop {
        let mut reactions = bot.http.reactions(refreshing.0, refreshing.1, &reactable);
        if let Some(last_user) = last_user {
            reactions = reactions.after(last_user);
        }
        let reactions = reactions.await?.model().await?;

        if reactions.is_empty() {
            break;
        }
        last_user = Some(reactions.last().unwrap().id);

        for user in reactions {
            if user.bot {
                continue;
            }

            let vote = VoteContext {
                emoji: &emoji,
                reactor_id: user.id,
                message_id: orig.message_id.into_id(),
                message_author_id: orig.author_id.into_id(),
                channel_id: orig.channel_id.into_id(),
                message_author_is_bot: author_is_bot,
                message_has_image: None,
                message_is_frozen: orig.frozen,
            };
            let status = VoteStatus::get_vote_status(bot, vote, configs).await?;

            let VoteStatus::Valid((upvotes, downvotes)) = status else {
                continue;
            };

            let user_id = user.id.get_i64();
            DbUser::create(&bot.db, user_id, user.bot).await?;
            DbMember::create(&bot.db, user_id, guild_id.get_i64()).await?;

            for config in &upvotes {
                Vote::create(
                    &bot.db,
                    orig.message_id,
                    config.starboard.id,
                    user_id,
                    orig.author_id,
                    false,
                )
                .await?;
            }
            for config in &downvotes {
                Vote::create(
                    &bot.db,
                    orig.message_id,
                    config.starboard.id,
                    user_id,
                    orig.author_id,
                    true,
                )
                .await?;
            }
        }
    }
    Ok(())
}
