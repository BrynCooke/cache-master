use crate::layer::Layer;
use crate::AsyncCache;
use async_trait::async_trait;
use std::future::Future;
use std::hash::Hash;

pub struct Cache<K, V, F, Fut>
where
    K: Send + Sync + Eq + Hash + Clone + 'static,
    V: Send + Sync + Clone + 'static,
    F: Fn(&K) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Option<V>> + Send,
{
    layers: Vec<Layer<K, V>>,
    coalesce: Option<bool>,
    loading_fn: F,
}

#[buildstructor::buildstructor]
impl<K, V, F, Fut> Cache<K, V, F, Fut>
where
    K: Send + Sync + Eq + Hash + Clone + 'static,
    V: Send + Sync + Clone + 'static,
    F: Fn(&K) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Option<V>> + Send,
{
    #[builder(visibility = "pub")]
    fn new(layers: Vec<Layer<K, V>>, coalesce: Option<bool>, loading_fn: F) -> Cache<K, V, F, Fut> {
        Cache {
            layers,
            coalesce,
            loading_fn,
        }
    }
}

#[async_trait]
impl<K, V, F, Fut> AsyncCache for Cache<K, V, F, Fut>
where
    K: Send + Sync + Eq + Hash + Clone + 'static,
    V: Send + Sync + Clone + 'static,
    F: Fn(&K) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Option<V>> + Send,
{
    type K = K;
    type V = V;

    async fn get(&mut self, key: &Self::K) -> Option<Self::V> {
        todo!()
    }
}
