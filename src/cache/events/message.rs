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

        let message = CachedMessage::from(&self.0);
        cache
            .messages
            .insert(self.id, Some(Arc::new(message)), 1)
            .await;
    }
}

#[async_trait]
impl UpdateCache for MessageDelete {
    async fn update_cache(&self, cache: &Cache) {
        cache.messages.insert(self.id, None, 1).await;
    }
}

#[async_trait]
impl UpdateCache for MessageDeleteBulk {
    async fn update_cache(&self, cache: &Cache) {
        for id in &self.ids {
            cache.messages.insert(*id, None, 1).await;
        }
    }
}

#[async_trait]
impl UpdateCache for MessageUpdate {
    async fn update_cache(&self, cache: &Cache) {
        let cached = {
            let cached = cache.messages.get(&self.id);

            match cached {
                None => return,
                Some(msg) => msg.value().clone(),
            }
        };

        let Some(cached) = cached else {
            cache.messages.remove(&self.id).await;
            return;
        };

        let attachments = match &self.attachments {
            Some(attachments) => attachments.clone(),
            None => cached.attachments.clone(),
        };
        let embeds = match &self.embeds {
            Some(embeds) => embeds.clone(),
            None => cached.embeds.clone(),
        };
        // here we assume that, if the messages content was edited,
        // it doesn't have any "specialty" and thus we don't need
        // to try to parse the system content from the messages kind.
        // For example, a join message will never be edited.
        let content = match &self.content {
            Some(content) => content.clone(),
            None => cached.content.clone(),
        };

        let message = CachedMessage {
            author_id: cached.author_id,
            attachments,
            embeds,
            content,
            stickers: cached.stickers.clone(),
            referenced_message: cached.referenced_message,
        };

        cache
            .messages
            .insert(self.id, Some(Arc::new(message)), 1)
            .await;
    }
}
