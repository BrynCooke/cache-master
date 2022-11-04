use crate::AsyncCache;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::{OwnedRwLockWriteGuard, RwLock};

pub struct Coalesce<C>
where
    C: AsyncCache,
{
    delegate: C,
    in_flight: Arc<Mutex<HashMap<C::K, Arc<RwLock<Option<C::V>>>>>>,
}

impl<C> Coalesce<C>
where
    C: AsyncCache,
{
    pub fn new(delegate: C) -> Coalesce<C> {
        Coalesce {
            delegate,
            in_flight: Default::default(),
        }
    }
}

struct Cleanup<C>
where
    C: AsyncCache,
{
    key: C::K,
    in_flight: Arc<Mutex<HashMap<C::K, Arc<RwLock<Option<C::V>>>>>>,
}

impl<C> Drop for Cleanup<C>
where
    C: AsyncCache,
{
    fn drop(&mut self) {
        self.in_flight
            .lock()
            .expect("lock poisoned")
            .remove(&self.key);
    }
}

#[async_trait]
impl<C> AsyncCache for Coalesce<C>
where
    C: AsyncCache + Sync + Send,
{
    type K = C::K;
    type V = C::V;

    async fn get(&mut self, key: &Self::K) -> Option<Self::V> {
        enum Role<V> {
            Fetcher(OwnedRwLockWriteGuard<Option<V>>),
            Waiter(Arc<RwLock<Option<V>>>),
        }

        let role;
        // Lock the in_flight map and decide if we are the Fetcher or Waiter.
        // in_flight is a sdt::sync::mutex so that we don't have an await during this block.
        // This is fine because we are not doing any blocking operations in this section.
        {
            let mut in_flight = self.in_flight.lock().unwrap();
            if let Some(sentinel) = (*in_flight).get(key).cloned() {
                // There was a lock in the in_flight map so there must be an existing fetcher in progress.
                role = Role::Waiter(sentinel);
            } else {
                // No fetch in progress. Insert an initially locked RwLock into the in_flight map.
                // Note that the RwLock is a tokio::sync::RwLock. Means that when long running operations
                // happen later we don't block a tokio thread.
                let lock = Arc::new(RwLock::new(None));
                role = Role::Fetcher(
                    lock.clone()
                        .try_write_owned()
                        .expect("we just created the lock, qed"),
                );
                (*in_flight).insert(key.clone(), lock.clone());
            }
        }
        // We know who we are! The in_flight map is unlocked. Now take action based on our role.
        // This is where long running/async stuff may happen.
        match role {
            Role::Fetcher(mut guard) => {
                // We are the fetcher
                // We need to make sure that we clean up the map after we've finished. RAII is used for this.
                let _cleanup: Cleanup<C> = Cleanup {
                    key: key.clone(),
                    in_flight: self.in_flight.clone(),
                };

                let value = self.delegate.get(key).await;
                *guard = value.clone();
                value
                // Cleanup happens here when _cleanup falls out of scope.
            }
            Role::Waiter(sentinel) => {
                // We are a waiter, wait on the RwLock. Then the fetcher has finished doing it's stuff we'll get the result.
                sentinel.read().await.clone()
            }
        }
    }
}
