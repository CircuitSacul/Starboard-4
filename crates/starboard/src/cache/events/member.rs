use std::sync::Arc;

use async_trait::async_trait;
use twilight_model::gateway::payload::incoming::{MemberRemove, MemberUpdate};

use crate::cache::{cache_struct::Cache, update::UpdateCache};

#[async_trait]
impl UpdateCache for MemberRemove {
    async fn update_cache(&self, cache: &Cache) {
        cache
            .members
            .invalidate(&(self.guild_id, self.user.id))
            .await;
    }
}

#[async_trait]
impl UpdateCache for MemberUpdate {
    async fn update_cache(&self, cache: &Cache) {
        if cache.members.contains_key(&(self.guild_id, self.user.id)) {
            cache
                .members
                .insert((self.guild_id, self.user.id), Some(Arc::new(self.into())))
                .await;
        }

        if cache.users.contains_key(&self.user.id) {
            cache
                .users
                .insert(self.user.id, Some(Arc::new((&self.user).into())))
                .await;
        }
    }
}
