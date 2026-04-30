use crate::config::{HttpConfig, HttpMethod};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, Semaphore, watch};

pub struct IterResult {
    pub duration: Duration,
    pub status_code: u16,
    #[allow(dead_code)]
    pub bytes: u64,
    pub is_error: bool,
}

pub struct HttpWorkerPool {
    config: HttpConfig,
    semaphore: Arc<Semaphore>,
    pause_rx: watch::Receiver<bool>,
    cancel_rx: watch::Receiver<bool>,
    result_tx: mpsc::UnboundedSender<IterResult>,
}

impl HttpWorkerPool {
    pub fn new(
        config: HttpConfig,
        initial_concurrency: usize,
        pause_rx: watch::Receiver<bool>,
        cancel_rx: watch::Receiver<bool>,
        result_tx: mpsc::UnboundedSender<IterResult>,
    ) -> Self {
        let semaphore = Arc::new(Semaphore::new(initial_concurrency));
        Self {
            config,
            semaphore,
            pause_rx,
            cancel_rx,
            result_tx,
        }
    }

    pub fn semaphore(&self) -> Arc<Semaphore> {
        self.semaphore.clone()
    }

    #[allow(dead_code)]
    pub fn add_permits(&self, n: usize) {
        self.semaphore.add_permits(n);
    }

    pub fn spawn(self, total_workers: usize) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move { self.run(total_workers).await })
    }

    async fn run(self, total_workers: usize) {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(self.config.insecure)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        let mut handles = Vec::with_capacity(total_workers);

        for _ in 0..total_workers {
            let client = client.clone();
            let config = self.config.clone();
            let semaphore = self.semaphore.clone();
            let mut pause_rx = self.pause_rx.clone();
            let mut cancel_rx = self.cancel_rx.clone();
            let result_tx = self.result_tx.clone();

            let handle = tokio::spawn(async move {
                loop {
                    // Check cancellation
                    if *cancel_rx.borrow() {
                        return;
                    }

                    // Check pause
                    while *pause_rx.borrow() {
                        tokio::select! {
                            _ = cancel_rx.changed() => {
                                if *cancel_rx.borrow() { return; }
                            }
                            _ = pause_rx.changed() => {}
                        }
                    }

                    tokio::select! {
                        _ = cancel_rx.changed() => {
                            if *cancel_rx.borrow() { return; }
                        }
                        permit = semaphore.acquire() => {
                            if let Ok(permit) = permit {
                                let result = Self::execute_request(&client, &config).await;
                                let _ = result_tx.send(result);
                                drop(permit);
                            }
                        }
                    }
                }
            });
            handles.push(handle);
        }

        // Wait for cancellation
        let mut cancel_rx = self.cancel_rx;
        loop {
            tokio::select! {
                _ = cancel_rx.changed() => {
                    if *cancel_rx.borrow() { break; }
                }
            }
        }

        for h in handles {
            h.abort();
        }
    }

    async fn execute_request(client: &reqwest::Client, config: &HttpConfig) -> IterResult {
        let start = Instant::now();
        let req = match config.method {
            HttpMethod::GET => client.get(&config.url),
            HttpMethod::POST => client.post(&config.url),
            HttpMethod::PUT => client.put(&config.url),
            HttpMethod::DELETE => client.delete(&config.url),
            HttpMethod::PATCH => client.patch(&config.url),
            HttpMethod::HEAD => client.head(&config.url),
            HttpMethod::OPTIONS => client.request(reqwest::Method::OPTIONS, &config.url),
        };

        let req = config.headers.iter().filter(|(k, v)| !k.is_empty() && !v.is_empty()).fold(req, |req, (k, v)| {
            req.header(k.as_str(), v.as_str())
        });

        let req = if matches!(config.method, HttpMethod::POST | HttpMethod::PUT | HttpMethod::PATCH) && !config.body.is_empty() {
            req.body(config.body.clone())
        } else {
            req
        };

        match req.send().await {
            Ok(resp) => {
                let status = resp.status().as_u16();
                let bytes = resp.bytes().await.map(|b| b.len() as u64).unwrap_or(0);
                let is_error = status >= 400;
                IterResult {
                    duration: start.elapsed(),
                    status_code: status,
                    bytes,
                    is_error,
                }
            }
            Err(_) => IterResult {
                duration: start.elapsed(),
                status_code: 0,
                bytes: 0,
                is_error: true,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execute_request_get() {
        let config = HttpConfig {
            url: "https://httpbin.org/get".to_string(),
            method: HttpMethod::GET,
            headers: vec![],
            body: String::new(),
            insecure: false,
        };
        let client = reqwest::Client::new();
        let result = HttpWorkerPool::execute_request(&client, &config).await;
        assert_eq!(result.status_code, 200);
        assert!(!result.is_error);
        assert!(result.duration.as_millis() > 0);
    }

    #[tokio::test]
    async fn test_execute_request_404() {
        let config = HttpConfig {
            url: "https://httpbin.org/status/404".to_string(),
            method: HttpMethod::GET,
            headers: vec![],
            body: String::new(),
            insecure: false,
        };
        let client = reqwest::Client::new();
        let result = HttpWorkerPool::execute_request(&client, &config).await;
        assert_eq!(result.status_code, 404);
        assert!(result.is_error);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_worker_pool_start_stop() {
        let config = HttpConfig {
            url: "https://httpbin.org/get".to_string(),
            method: HttpMethod::GET,
            headers: vec![],
            body: String::new(),
            insecure: false,
        };
        let (result_tx, mut result_rx) = mpsc::unbounded_channel();
        let (_pause_tx, pause_rx) = watch::channel(false);
        let (cancel_tx, cancel_rx) = watch::channel(false);

        // Only 2 workers to keep test fast
        let pool = HttpWorkerPool::new(
            config,
            2,               // initial_concurrency
            pause_rx,
            cancel_rx,
            result_tx,
        );
        let handle = pool.spawn(2);

        // Let it run for a brief time
        tokio::time::sleep(Duration::from_secs(3)).await;

        // Stop
        let _ = cancel_tx.send(true);
        handle.await.unwrap();

        // Count results after everything is done
        let count = std::sync::Mutex::new(0u32);
        while let Ok(_) = result_rx.try_recv() {
            *count.lock().unwrap() += 1;
        }

        let final_count = count.lock().unwrap();
        assert!(*final_count > 0, "Should have completed at least one request");
    }
}
