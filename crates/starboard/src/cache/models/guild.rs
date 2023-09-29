use std::collections::HashMap;

use twilight_model::id::{
    marker::{ChannelMarker, EmojiMarker, RoleMarker},
    Id,
};

use super::{channel::CachedChannel, role::CachedRole};

pub struct CachedGuild {
    pub name: String,
    /// all custom emojis mapped to whether they are animated
    pub emojis: HashMap<Id<EmojiMarker>, bool>,
    /// all textable channels except for threads
    pub channels: HashMap<Id<ChannelMarker>, CachedChannel>,
    pub roles: HashMap<Id<RoleMarker>, CachedRole>,
    pub active_thread_parents: HashMap<Id<ChannelMarker>, Id<ChannelMarker>>,
}
