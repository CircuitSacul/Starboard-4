use std::str::FromStr;

use lazy_static::lazy_static;
use regex::Regex;
use twilight_http::request::channel::reaction::RequestReactionType;
use twilight_mention::Mention;
use twilight_model::{
    channel::message::ReactionType,
    id::{
        marker::{EmojiMarker, GuildMarker},
        Id,
    },
};

use crate::client::bot::StarboardBot;

/// Get rid of the Variation-Selector-16 codepoint that is sometimes present in user
/// input. https://emojipedia.org/variation-selector-16/
pub fn clean_emoji(unicode: &str) -> &str {
    unicode.strip_suffix('\u{fe0f}').unwrap_or(unicode)
}

#[derive(Clone)]
pub struct SimpleEmoji {
    raw: String,
    as_id: Option<Id<EmojiMarker>>,
}

impl PartialEq for SimpleEmoji {
    fn eq(&self, other: &Self) -> bool {
        clean_emoji(&other.raw) == clean_emoji(&self.raw)
    }
}

impl PartialEq<String> for SimpleEmoji {
    fn eq(&self, other: &String) -> bool {
        clean_emoji(&self.raw) == clean_emoji(other)
    }
}

impl SimpleEmoji {
    pub fn from_user_input(
        input: &str,
        bot: &StarboardBot,
        guild_id: Id<GuildMarker>,
    ) -> Vec<Self> {
        lazy_static! {
            static ref RE: Regex = Regex::new(concat!(
                r"(<a?:\w+:(?P<emoji_id>\d+)>",
                r"|(?P<flag>[\u{1f1e6}-\u{1f1ff}]{2,})",
                r"|(?P<unicode>.(\u{200d}.)*\u{fe0f}?))",
            ))
            .unwrap();
        };

        let mut emojis = Vec::new();
        for caps in RE.captures_iter(input) {
            if let Some(emoji_id) = caps.name("emoji_id") {
                let id: Id<EmojiMarker> = emoji_id.as_str().parse().unwrap();
                if !bot.cache.guild_emoji_exists(guild_id, id) {
                    continue;
                }

                emojis.push(Self {
                    raw: emoji_id.as_str().to_owned(),
                    as_id: Some(id),
                });
            } else if let Some(emoji) = caps.name("unicode") {
                if emojis::get(emoji.as_str()).is_some() {
                    emojis.push(Self {
                        raw: emoji.as_str().to_owned(),
                        as_id: None,
                    });
                }
            } else if let Some(flag) = caps.name("flag") {
                // a flag is two consecutive regional indicators. The good news is that
                // regional indicators are single codepoints which makes it easy to iterate
                let mut letters = flag.as_str().chars();
                // safety: the regex only matches for 2 or more consecutive regional indicators
                let mut previous = letters.next().unwrap();

                loop {
                    let Some(letter) = letters.next() else {
                        emojis.push(Self {
                            raw: previous.into(),
                            as_id: None,
                        });
                        break;
                    };

                    let flag = String::from_iter([previous, letter]);

                    if emojis::get(&flag).is_some() {
                        emojis.push(Self {
                            raw: flag,
                            as_id: None,
                        });

                        if let Some(next) = letters.next() {
                            previous = next;
                        } else {
                            break;
                        }
                    } else {
                        emojis.push(Self {
                            raw: previous.into(),
                            as_id: None,
                        });
                        previous = letter;
                    }
                }
            }
        }

        emojis
    }
}

pub trait EmojiCommon: Sized {
    type FromOut;
    type Stored;

    fn into_readable(self, bot: &StarboardBot, guild_id: Id<GuildMarker>) -> String;
    fn into_stored(self) -> Self::Stored;
    fn from_stored(stored: Self::Stored) -> Self;
}

impl SimpleEmoji {
    pub fn reactable(&self) -> RequestReactionType {
        if let Some(emoji_id) = self.as_id {
            RequestReactionType::Custom {
                name: None,
                id: emoji_id,
            }
        } else {
            RequestReactionType::Unicode { name: &self.raw }
        }
    }
}

impl EmojiCommon for SimpleEmoji {
    type FromOut = Option<Self>;
    type Stored = String;

    fn into_readable(self, bot: &StarboardBot, guild_id: Id<GuildMarker>) -> String {
        if let Some(emoji_id) = self.as_id {
            match bot.cache.is_emoji_animated(guild_id, emoji_id) {
                None => self.raw,
                Some(true) => format!("<a:name:{emoji_id}>"),
                Some(false) => emoji_id.mention().to_string(),
            }
        } else {
            self.raw
        }
    }

    fn from_stored(raw: Self::Stored) -> Self {
        let as_id = match Id::<EmojiMarker>::from_str(&raw) {
            Ok(value) => Some(value),
            Err(_) => None,
        };

        Self { raw, as_id }
    }

    fn into_stored(self) -> Self::Stored {
        self.raw
    }
}

impl EmojiCommon for Vec<SimpleEmoji> {
    type FromOut = Self;
    type Stored = Vec<String>;

    fn into_readable(self, bot: &StarboardBot, guild_id: Id<GuildMarker>) -> String {
        let mut arr = Vec::new();
        for emoji in self {
            arr.push(emoji.into_readable(bot, guild_id));
        }
        if arr.is_empty() {
            "no emojis".to_string()
        } else {
            arr.join(", ")
        }
    }

    fn from_stored(stored: Self::Stored) -> Self {
        let mut arr = Vec::new();
        for piece in stored {
            arr.push(SimpleEmoji::from_stored(piece));
        }
        arr
    }

    fn into_stored(self) -> Self::Stored {
        let mut arr = Vec::new();
        for emoji in self {
            arr.push(emoji.into_stored());
        }
        arr
    }
}

impl From<ReactionType> for SimpleEmoji {
    fn from(reaction: ReactionType) -> Self {
        match reaction {
            ReactionType::Custom { id, .. } => SimpleEmoji {
                raw: id.to_string(),
                as_id: Some(id),
            },
            ReactionType::Unicode { name } => SimpleEmoji {
                raw: name,
                as_id: None,
            },
        }
    }
}
