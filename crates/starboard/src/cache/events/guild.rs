use std::collections::HashMap;

use async_trait::async_trait;
use twilight_model::gateway::payload::incoming::{GuildCreate, GuildDelete, GuildEmojisUpdate};

use crate::cache::{
    cache_struct::Cache,
    models::{channel::CachedChannel, guild::CachedGuild},
    update::UpdateCache,
};

#[async_trait]
impl UpdateCache for GuildCreate {
    async fn update_cache(&self, cache: &Cache) {
        let channels = self
            .channels
            .iter()
            .map(|c| (c.id, CachedChannel::from_channel(None, c)))
            .collect();

        let guild = CachedGuild {
            name: self.name.clone(),
            emojis: self.emojis.iter().map(|e| (e.id, e.animated)).collect(),
            channels,
            active_thread_parents: self
                .threads
                .iter()
                .map(|t| (t.id, t.parent_id.unwrap()))
                .collect(),
            roles: self
                .roles
                .iter()
                .map(|r| (r.id, r.into()))
                .collect::<HashMap<_, _>>(),
        };
        cache.guilds.insert(self.id, guild);
    }
}

#[async_trait]
impl UpdateCache for GuildDelete {
    async fn update_cache(&self, cache: &Cache) {
        cache.guilds.remove(&self.id);
    }
}

#[async_trait]
impl UpdateCache for GuildEmojisUpdate {
    async fn update_cache(&self, cache: &Cache) {
        cache.guilds.alter(&self.guild_id, |_, mut guild| {
            for emoji in &self.emojis {
                guild.emojis.insert(emoji.id, emoji.animated);
            }
            guild
        });
    }
}
