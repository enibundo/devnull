use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use std::{
    collections::HashMap,
    result::Result,
    sync::{Arc, Mutex},
};

type City = String;
type Temperature = u64;

#[async_trait]
pub trait Api: Send + Sync + 'static {
    async fn fetch(&self) -> Result<HashMap<City, Temperature>, String>;
    async fn subscribe(&self) -> BoxStream<Result<(City, Temperature), String>>;
}

pub struct StreamCache {
    results: Arc<Mutex<HashMap<String, u64>>>,
}

impl StreamCache {
    pub fn new(api: Arc<dyn Api>) -> Self {
        let instance = Self {
            results: Arc::new(Mutex::new(HashMap::new())),
        };
        instance.update_in_background(api);
        instance
    }

    pub fn get(&self, key: &str) -> Option<u64> {
        let results = self.results.lock().expect("poisoned");
        results.get(key).copied()
    }

    pub fn update_in_background(&self, api: Arc<dyn Api + 'static>) {
        let results = Arc::clone(&self.results);
        let results2 = Arc::clone(&self.results);

        let api_for_subscription = Arc::clone(&api);
        let api_for_fetch = Arc::clone(&api);

        // Spawn a background task for fetching the initial data
        tokio::spawn(async move {
            match api_for_fetch.fetch().await {
                Ok(data) => {
                    let mut results_guard = results2.lock().expect("Mutex poisoned");
                    for (city, temperature) in data {
                        results_guard.insert(city, temperature);
                    }
                }
                Err(err) => {
                    eprintln!("Failed to fetch initial data: {}", err);
                }
            }
        });

        // Spawn a background task for updating the cache with subscription data
        tokio::spawn(async move {
            let mut stream = api_for_subscription.subscribe().await;

            while let Some(update) = stream.next().await {
                match update {
                    Ok((city, temperature)) => {
                        let mut results_guard = results.lock().expect("Mutex poisoned");
                        results_guard.insert(city, temperature);
                    }
                    Err(err) => {
                        eprintln!("Error in subscription stream: {}", err);
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use tokio::sync::Notify;
    use tokio::time;

    use futures::{future, stream::select, FutureExt, StreamExt};
    use maplit::hashmap;

    use super::*;

    #[derive(Default)]
    pub struct TestApi {
        signal: Arc<Notify>,
    }

    #[async_trait]
    impl Api for TestApi {
        async fn fetch(&self) -> Result<HashMap<City, Temperature>, String> {
            // fetch is slow an may get delayed until after we receive the first updates
            self.signal.notified().await;

            Ok(hashmap! {
                "Berlin".to_string() => 29,
                "Paris".to_string() => 31,
            })
        }

        async fn subscribe(&self) -> BoxStream<Result<(City, Temperature), String>> {
            let results = vec![
                Ok(("London".to_string(), 27)),
                Ok(("Paris".to_string(), 32)),
            ];
            select(
                futures::stream::iter(results),
                async {
                    self.signal.notify_one();
                    time::sleep(Duration::from_millis(1_000)).await;
                    future::pending().await
                }
                .into_stream(),
            )
            .boxed()
        }
    }

    #[tokio::test]
    async fn works() {
        let api: Arc<dyn Api> = Arc::new(TestApi::default());
        let cache: StreamCache = StreamCache::new(api);

        // Allow cache to update
        time::sleep(Duration::from_millis(100)).await;

        assert_eq!(cache.get("Berlin"), Some(29));
        assert_eq!(cache.get("London"), Some(27));
        assert_eq!(cache.get("Paris"), Some(32));
    }
}
