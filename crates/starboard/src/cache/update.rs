use async_trait::async_trait;

use super::cache_struct::Cache;

#[async_trait]
pub(super) trait UpdateCache {
    async fn update_cache(&self, cache: &Cache);
}
