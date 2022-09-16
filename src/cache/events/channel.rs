use async_trait::async_trait;
use twilight_model::gateway::payload::incoming::{ChannelCreate, ChannelDelete, ChannelUpdate};

use crate::cache::{cache::Cache, models::channel::CachedChannel, update::UpdateCache};

#[async_trait]
impl UpdateCache for ChannelCreate {
    async fn update_cache(&self, cache: &Cache) {
        let guild_id = match self.guild_id {
            None => return,
            Some(guild_id) => guild_id,
        };

        cache.guilds.alter(&guild_id, |_, mut guild| {
            let channel = guild.channels.get(&self.id);
            guild
                .channels
                .insert(self.id, CachedChannel::from_channel(channel, self));

            guild
        });
    }
}

#[async_trait]
impl UpdateCache for ChannelDelete {
    async fn update_cache(&self, cache: &Cache) {
        let guild_id = match self.guild_id {
            None => return,
            Some(guild_id) => guild_id,
        };

        cache.guilds.alter(&guild_id, |_, mut guild| {
            guild.channels.remove(&self.id);

            guild
                .active_thread_parents
                .retain(|_, &mut channel_id| channel_id != self.id);

            guild
        });
    }
}

#[async_trait]
impl UpdateCache for ChannelUpdate {
    async fn update_cache(&self, cache: &Cache) {
        let guild_id = match self.guild_id {
            None => return,
            Some(guild_id) => guild_id,
        };

        cache.guilds.alter(&guild_id, |_, mut guild| {
            let channel = guild.channels.get(&self.id);
            guild
                .channels
                .insert(self.id, CachedChannel::from_channel(channel, self));

            guild
        });
    }
}
