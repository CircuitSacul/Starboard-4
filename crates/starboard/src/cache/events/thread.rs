use std::collections::{HashMap, HashSet};

use async_trait::async_trait;
use twilight_model::gateway::payload::incoming::{
    ThreadCreate, ThreadDelete, ThreadListSync, ThreadUpdate,
};

use crate::{
    cache::{cache_struct::Cache, update::UpdateCache},
    utils::into_id::IntoId,
};

#[async_trait]
impl UpdateCache for ThreadCreate {
    async fn update_cache(&self, cache: &Cache) {
        let Some(parent_id) = self.parent_id else {
            return;
        };
        let Some(guild_id) = self.guild_id else {
            return;
        };

        cache.guilds.alter(&guild_id, |_, mut guild| {
            guild.active_thread_parents.insert(self.id, parent_id);
            guild
        });
    }
}

#[async_trait]
impl UpdateCache for ThreadDelete {
    async fn update_cache(&self, cache: &Cache) {
        cache.guilds.alter(&self.guild_id, |_, mut guild| {
            guild.active_thread_parents.remove(&self.id);
            guild
        });
        cache.messages.invalidate(&self.id.get().into_id()).await;
    }
}

#[async_trait]
impl UpdateCache for ThreadUpdate {
    async fn update_cache(&self, cache: &Cache) {
        let Some(guild_id) = self.guild_id else {
            return;
        };
        let Some(thread) = &self.thread_metadata else {
            return;
        };
        let Some(parent_id) = self.parent_id else {
            return;
        };

        cache.guilds.alter(&guild_id, |_, mut guild| {
            if thread.archived {
                guild.active_thread_parents.remove(&self.id);
            } else {
                guild.active_thread_parents.insert(self.id, parent_id);
            }

            guild
        });
    }
}

#[async_trait]
impl UpdateCache for ThreadListSync {
    async fn update_cache(&self, cache: &Cache) {
        cache.guilds.alter(&self.guild_id, |_, mut guild| {
            if self.channel_ids.is_empty() {
                guild.active_thread_parents = self
                    .threads
                    .iter()
                    .map(|t| (t.id, t.parent_id.unwrap()))
                    .collect();
            } else {
                // ThreadListSync only syncs threads for the channels it sends,
                // so any threads belonging to other channels should stay.
                let channel_ids: HashSet<_> = self.channel_ids.iter().collect();
                let mut threads: HashMap<_, _> = guild
                    .active_thread_parents
                    .into_iter()
                    .filter(|(_, parent_id)| !channel_ids.contains(parent_id))
                    .collect();

                for thread in &self.threads {
                    if let Some(parent_id) = thread.parent_id {
                        threads.insert(thread.id, parent_id);
                    }
                }

                guild.active_thread_parents = threads;
            }

            guild
        });
    }
}
