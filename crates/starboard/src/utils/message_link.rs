use std::fmt::Display;

use lazy_static::lazy_static;
use regex::Regex;

pub fn parse_message_link(link: &str) -> Option<(i64, i64)> {
    if let Some((channel_id, message_id)) = link.split_once('-') {
        let channel_id = match channel_id.parse::<i64>() {
            Ok(channel_id) => channel_id,
            Err(_) => return None,
        };
        let message_id = match message_id.parse::<i64>() {
            Ok(message_id) => message_id,
            Err(_) => return None,
        };

        return Some((channel_id, message_id));
    }

    lazy_static! {
        static ref RE: Regex = Regex::new(r"/channels/(\d+)/(\d+)/(\d+)").unwrap();
    }

    let ret = RE.captures(link)?;

    let channel_id: i64 = ret.get(2).unwrap().as_str().parse().unwrap();
    let message_id: i64 = ret.get(3).unwrap().as_str().parse().unwrap();

    Some((channel_id, message_id))
}

pub fn fmt_message_link(
    guild_id: impl Display,
    channel_id: impl Display,
    message_id: impl Display,
) -> String {
    format!("https://discord.com/channels/{guild_id}/{channel_id}/{message_id}")
}
