//! Parsing and validation for different types of mentions.

use std::{collections::HashSet, str::FromStr};

use lazy_static::lazy_static;
use twilight_model::id::{marker::GuildMarker, Id};

use errors::StarboardResult;

use crate::{client::bot::StarboardBot, utils::id_as_i64::GetI64};

fn parse_numbers<IdT>(inp: &str) -> impl Iterator<Item = IdT> + '_
where
    IdT: FromStr,
    <IdT as FromStr>::Err: std::fmt::Debug,
{
    lazy_static! {
        static ref RE: regex::Regex = regex::Regex::new(r"\d+").unwrap();
    }

    RE.find_iter(inp).map(|val| val.as_str().parse().unwrap())
}

/// Parse channel ids from user input.
pub async fn textable_channel_ids(
    bot: &StarboardBot,
    guild_id: Id<GuildMarker>,
    inp: &str,
) -> StarboardResult<HashSet<i64>> {
    let mut ret = HashSet::new();

    for channel_id in parse_numbers(inp) {
        if !bot
            .cache
            .guild_has_channel(bot, guild_id, channel_id)
            .await?
        {
            continue;
        }

        ret.insert(channel_id.get_i64());
    }

    Ok(ret)
}

/// Parse role ids from user input
pub fn parse_role_ids(bot: &StarboardBot, guild_id: Id<GuildMarker>, inp: &str) -> HashSet<i64> {
    let mut ret = HashSet::new();

    for role_id in parse_numbers(inp) {
        if !bot.cache.guilds.with(&guild_id, |_, g| {
            g.as_ref().map_or(false, |g| g.roles.contains_key(&role_id))
        }) {
            continue;
        }

        ret.insert(role_id.get_i64());
    }

    ret
}
