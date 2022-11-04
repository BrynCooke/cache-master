use crate::AsyncCache;
use lru::LruCache;

pub enum Layer<K, V> {
    NoCache,
    LruCache(LruCache<K, V>),
    Moka(moka::sync::Cache<K, V>),
    Custom(Box<dyn AsyncCache<K = K, V = V>>),
}

impl<K, V> From<LruCache<K, V>> for Layer<K, V> {
    fn from(cache: LruCache<K, V>) -> Self {
        Layer::LruCache(cache)
    }
}

impl<K, V> From<moka::sync::Cache<K, V>> for Layer<K, V> {
    fn from(cache: moka::sync::Cache<K, V>) -> Self {
        Layer::Moka(cache)
    }
}
