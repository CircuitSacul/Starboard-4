use async_trait::async_trait;
use twilight_model::gateway::payload::incoming::{RoleCreate, RoleDelete, RoleUpdate};

use crate::cache::{cache_struct::Cache, update::UpdateCache};

#[async_trait]
impl UpdateCache for RoleCreate {
    async fn update_cache(&self, cache: &Cache) {
        cache.guilds.alter(&self.guild_id, |_, mut guild| {
            guild.roles.insert(self.role.id, (&self.role).into());
            guild
        })
    }
}

#[async_trait]
impl UpdateCache for RoleDelete {
    async fn update_cache(&self, cache: &Cache) {
        cache.guilds.alter(&self.guild_id, |_, mut guild| {
            guild.roles.remove(&self.role_id);
            guild
        })
    }
}

#[async_trait]
impl UpdateCache for RoleUpdate {
    async fn update_cache(&self, cache: &Cache) {
        cache.guilds.alter(&self.guild_id, |_, mut guild| {
            guild.roles.insert(self.role.id, (&self.role).into());
            guild
        })
    }
}
