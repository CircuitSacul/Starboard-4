use std::sync::Arc;

use async_trait::async_trait;
use twilight_model::gateway::payload::incoming::{
    MessageCreate, MessageDelete, MessageDeleteBulk, MessageUpdate,
};

use crate::cache::{cache::Cache, models::message::CachedMessage, update::UpdateCache};

#[async_trait]
impl UpdateCache for MessageCreate {
    async fn update_cache(&self, cache: &Cache) {
        // We only need to cache created messages if the channel is an autostar channel.
        if !cache.autostar_channel_ids.contains(&self.channel_id) {
            return;
        }

        let message = CachedMessage {
            id: self.id,
            attachments: self.attachments.clone(),
            embeds: self.embeds.clone(),
        };

        cache.messages.insert(message.id, Arc::new(message)).await;
    }
}

#[async_trait]
impl UpdateCache for MessageDelete {
    async fn update_cache(&self, cache: &Cache) {
        cache.messages.invalidate(&self.id).await;
    }
}

#[async_trait]
impl UpdateCache for MessageDeleteBulk {
    async fn update_cache(&self, cache: &Cache) {
        for id in &self.ids {
            cache.messages.invalidate(id).await;
        }
    }
}

#[async_trait]
impl UpdateCache for MessageUpdate {
    async fn update_cache(&self, cache: &Cache) {
        let cached = cache.messages.get(&self.id);

        let cached = match cached {
            None => return,
            Some(msg) => msg,
        };

        let attachments = match &self.attachments {
            Some(attachments) => attachments.clone(),
            None => cached.attachments.clone(),
        };
        let embeds = match &self.embeds {
            Some(embeds) => embeds.clone(),
            None => cached.embeds.clone(),
        };

        let message = CachedMessage {
            id: self.id,
            attachments,
            embeds,
        };

        cache.messages.insert(self.id, Arc::new(message)).await;
    }
}