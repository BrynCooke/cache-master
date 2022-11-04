mod cache;
mod coalesce;
mod layer;

use async_trait::async_trait;
use std::hash::Hash;

pub use cache::Cache;

#[async_trait]
pub trait AsyncCache: Send + Sync {
    type K: Send + Sync + Eq + Hash + Clone;
    type V: Send + Sync + Clone;
    async fn get(&mut self, key: &Self::K) -> Option<Self::V>;
}

#[cfg(test)]
mod tests {
    use crate::{AsyncCache, Cache};
    use lru::LruCache;
    use std::num::NonZeroUsize;

    #[tokio::test]
    async fn it_works() {
        let mut cache = Cache::builder()
            .loading_fn(|k: &String| async { Some("value".to_string()) })
            .layer(LruCache::new(NonZeroUsize::new(100).unwrap()))
            .layer(moka::sync::Cache::new(100))
            .coalesce(true)
            .build();

        let result = cache.get(&"key".to_string()).await;
        assert_eq!(result, Some("value".to_string()))
    }
}
