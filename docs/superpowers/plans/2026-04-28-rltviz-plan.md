# rltviz Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a desktop GUI HTTP load testing tool with egui + custom Tokio worker pool, featuring step-up pressure control and real-time metrics visualization.

**Architecture:** Three-layer: egui GUI reads from `Arc<RwLock<MetricsSnapshot>>`, an AppCore layer (TestController + MetricsCollector) manages lifecycle and aggregates metrics, and a custom HTTP worker pool (Semaphore-based concurrency control) replaces rlt's private Runner. Communication between GUI and engine is via shared state (no channels in the hot path).

**Tech Stack:** Rust, eframe 0.31, egui_plot 0.31, tokio 1.x, reqwest 0.12, parking_lot 0.12

**Key decision:** rlt's `Runner` and `BenchOpts` are crate-private (confirmed via API research). We implement our own lightweight HTTP worker pool using `tokio::sync::Semaphore` for dynamic concurrency control, `watch::channel` for pause/resume, and `CancellationToken` for stop. This gives us full control over step-up pressure and real-time metrics streaming.

---

### Task 1: Project scaffold

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`

- [ ] **Step 1: Write Cargo.toml**

```toml
[package]
name = "rltviz"
version = "0.1.0"
edition = "2021"

[dependencies]
eframe = "0.31"
egui_plot = "0.31"
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["rustls-tls"], default-features = false }
parking_lot = "0.12"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

- [ ] **Step 2: Write minimal main.rs**

```rust
mod app;
mod config;
mod control;
mod engine;
mod metrics;
mod theme;
mod ui;

fn main() {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    let _guard = rt.enter();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0]),
        ..Default::default()
    };

    eframe::run_native(
        "rltviz - HTTP Load Test Visualizer",
        options,
        Box::new(|cc| Ok(Box::new(app::RltvizApp::new(cc, rt.handle().clone())))),
    )
    .expect("Failed to start eframe");
}
```

- [ ] **Step 3: Build to verify**

Run: `cargo check`
Expected: errors about missing modules (we'll add them next)

---

### Task 2: Config module

**Files:**
- Create: `src/config.rs`

- [ ] **Step 1: Write config.rs with HttpConfig, RampUpConfig, and HttpMethod**

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
}

impl HttpMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::PUT => "PUT",
            HttpMethod::DELETE => "DELETE",
        }
    }
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HttpConfig {
    pub url: String,
    pub method: HttpMethod,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            url: "https://httpbin.org/get".to_string(),
            method: HttpMethod::GET,
            headers: vec![],
            body: String::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RampUpConfig {
    pub start_concurrency: u32,
    pub end_concurrency: u32,
    pub steps: u32,
    pub step_duration_secs: u64,
}

impl Default for RampUpConfig {
    fn default() -> Self {
        Self {
            start_concurrency: 10,
            end_concurrency: 100,
            steps: 5,
            step_duration_secs: 30,
        }
    }
}

impl RampUpConfig {
    pub fn total_stages(&self) -> u32 {
        self.steps + 1
    }

    pub fn concurrency_step_size(&self) -> u32 {
        if self.steps == 0 {
            return 0;
        }
        (self.end_concurrency.saturating_sub(self.start_concurrency)) / self.steps
    }

    pub fn concurrency_at_stage(&self, stage: u32) -> u32 {
        if stage >= self.total_stages() {
            return self.end_concurrency;
        }
        self.start_concurrency + self.concurrency_step_size() * stage
    }

    pub fn total_duration_secs(&self) -> u64 {
        self.step_duration_secs * self.total_stages() as u64
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub http: HttpConfig,
    pub ramp_up: RampUpConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            http: HttpConfig::default(),
            ramp_up: RampUpConfig::default(),
        }
    }
}
```

- [ ] **Step 2: Build to verify (create stub files for other modules first)**

Create stub files so `cargo check` passes:

`src/app.rs`:
```rust
use eframe::Frame;

pub struct RltvizApp;

impl RltvizApp {
    pub fn new(cc: &eframe::CreationContext<'_>, handle: tokio::runtime::Handle) -> Self {
        Self
    }
}

impl eframe::App for RltvizApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut Frame) {}
}
```

`src/control.rs`:
```rust
// placeholder
```

`src/engine.rs`:
```rust
// placeholder
```

`src/metrics.rs`:
```rust
// placeholder
```

`src/theme.rs`:
```rust
// placeholder
```

`src/ui/mod.rs`:
```rust
// placeholder
```

Run: `cargo check`
Expected: builds successfully

- [ ] **Step 3: Write and run unit tests for RampUpConfig**

Add to `src/config.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rampup_total_stages() {
        let cfg = RampUpConfig {
            start_concurrency: 10,
            end_concurrency: 100,
            steps: 5,
            step_duration_secs: 30,
        };
        assert_eq!(cfg.total_stages(), 6);
    }

    #[test]
    fn test_rampup_concurrency_at_stage() {
        let cfg = RampUpConfig {
            start_concurrency: 10,
            end_concurrency: 100,
            steps: 5,
            step_duration_secs: 30,
        };
        assert_eq!(cfg.concurrency_at_stage(0), 10);
        assert_eq!(cfg.concurrency_at_stage(5), 100);
        assert_eq!(cfg.concurrency_at_stage(10), 100); // beyond last stage
    }

    #[test]
    fn test_rampup_step_size_zero_steps() {
        let cfg = RampUpConfig {
            start_concurrency: 50,
            end_concurrency: 100,
            steps: 0,
            step_duration_secs: 30,
        };
        assert_eq!(cfg.concurrency_step_size(), 0);
        assert_eq!(cfg.total_stages(), 1);
    }

    #[test]
    fn test_rampup_total_duration() {
        let cfg = RampUpConfig {
            start_concurrency: 10,
            end_concurrency: 100,
            steps: 5,
            step_duration_secs: 30,
        };
        assert_eq!(cfg.total_duration_secs(), 180);
    }
}
```

Run: `cargo test`
Expected: all 4 tests pass

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml src/
git commit -m "feat: project scaffold and config module"
```

---

### Task 3: Metrics module

**Files:**
- Create: `src/metrics.rs`

- [ ] **Step 1: Write metrics.rs with MetricsSnapshot and MetricsCollector**

```rust
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct MetricsSnapshot {
    pub qps: f64,
    pub latency_p50: Duration,
    pub latency_p90: Duration,
    pub latency_p99: Duration,
    pub error_rate: f64,
    pub status_codes: HashMap<u16, u64>,
    pub active_connections: u32,
    pub total_requests: u64,
    pub elapsed: Duration,
    pub current_step: u32,
    pub step_progress: f64,
}

impl Default for MetricsSnapshot {
    fn default() -> Self {
        Self {
            qps: 0.0,
            latency_p50: Duration::ZERO,
            latency_p90: Duration::ZERO,
            latency_p99: Duration::ZERO,
            error_rate: 0.0,
            status_codes: HashMap::new(),
            active_connections: 0,
            total_requests: 0,
            elapsed: Duration::ZERO,
            current_step: 0,
            step_progress: 0.0,
        }
    }
}

pub struct MetricsCollector {
    snapshot: Arc<RwLock<MetricsSnapshot>>,
    latency_samples: Vec<Duration>,
    status_counts: HashMap<u16, u64>,
    error_count: u64,
    request_count: u64,
    window_start: std::time::Instant,
}

impl MetricsCollector {
    pub fn new() -> (Self, Arc<RwLock<MetricsSnapshot>>) {
        let snapshot = Arc::new(RwLock::new(MetricsSnapshot::default()));
        let collector = Self {
            snapshot: snapshot.clone(),
            latency_samples: Vec::with_capacity(100_000),
            status_counts: HashMap::new(),
            error_count: 0,
            request_count: 0,
            window_start: std::time::Instant::now(),
        };
        (collector, snapshot)
    }

    pub fn record(&mut self, status_code: u16, latency: Duration, is_error: bool) {
        self.request_count += 1;
        self.latency_samples.push(latency);
        *self.status_counts.entry(status_code).or_insert(0) += 1;
        if is_error {
            self.error_count += 1;
        }
    }

    pub fn tick(&mut self, active_connections: u32, current_step: u32, step_progress: f64) {
        let elapsed = self.window_start.elapsed();
        let qps = if elapsed.as_secs_f64() > 0.0 {
            self.request_count as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };

        let error_rate = if self.request_count > 0 {
            self.error_count as f64 / self.request_count as f64
        } else {
            0.0
        };

        let (p50, p90, p99) = Self::compute_percentiles(&mut self.latency_samples);

        let mut snapshot = self.snapshot.write();
        snapshot.qps = qps;
        snapshot.latency_p50 = p50;
        snapshot.latency_p90 = p90;
        snapshot.latency_p99 = p99;
        snapshot.error_rate = error_rate;
        snapshot.status_codes = self.status_counts.clone();
        snapshot.active_connections = active_connections;
        snapshot.total_requests = self.request_count;
        snapshot.elapsed = elapsed;
        snapshot.current_step = current_step;
        snapshot.step_progress = step_progress;
    }

    fn compute_percentiles(samples: &mut Vec<Duration>) -> (Duration, Duration, Duration) {
        if samples.is_empty() {
            return (Duration::ZERO, Duration::ZERO, Duration::ZERO);
        }
        samples.sort_unstable();
        let p50 = samples[((samples.len() - 1) as f64 * 0.50) as usize];
        let p90 = samples[((samples.len() - 1) as f64 * 0.90) as usize];
        let p99 = samples[((samples.len() - 1) as f64 * 0.99) as usize];
        (p50, p90, p99)
    }
}
```

- [ ] **Step 2: Write unit tests for MetricsCollector**

Add to `src/metrics.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_and_tick() {
        let (mut collector, snapshot) = MetricsCollector::new();

        collector.record(200, Duration::from_millis(10), false);
        collector.record(200, Duration::from_millis(20), false);
        collector.record(200, Duration::from_millis(30), false);
        collector.record(500, Duration::from_millis(100), true);
        collector.record(500, Duration::from_millis(200), true);

        collector.tick(5, 0, 0.5);

        let snap = snapshot.read();
        assert_eq!(snap.total_requests, 5);
        assert_eq!(snap.active_connections, 5);
        assert!((snap.error_rate - 0.4).abs() < 0.01);
        assert_eq!(snap.status_codes.get(&200), Some(&3));
        assert_eq!(snap.status_codes.get(&500), Some(&2));
        assert_eq!(snap.current_step, 0);
        assert!((snap.step_progress - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_percentiles() {
        let mut samples: Vec<Duration> = (1..=100)
            .map(|i| Duration::from_millis(i))
            .collect();
        let (p50, p90, p99) = MetricsCollector::compute_percentiles(&mut samples);
        assert!(p50.as_millis() >= 45 && p50.as_millis() <= 55);
        assert!(p90.as_millis() >= 85 && p90.as_millis() <= 95);
        assert!(p99.as_millis() >= 95 && p99.as_millis() <= 100);
    }

    #[test]
    fn test_empty_snapshot() {
        let snap = MetricsSnapshot::default();
        assert_eq!(snap.qps, 0.0);
        assert_eq!(snap.total_requests, 0);
    }
}
```

Run: `cargo test metrics`
Expected: all 3 tests pass

- [ ] **Step 3: Commit**

```bash
git add src/metrics.rs
git commit -m "feat: metrics module with MetricsCollector and MetricsSnapshot"
```

---

### Task 4: HTTP worker engine

**Files:**
- Create: `src/engine.rs`

- [ ] **Step 1: Write engine.rs with HttpWorkerPool**

```rust
use crate::config::{HttpConfig, HttpMethod, RampUpConfig};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, Semaphore, watch};
use tokio_util::sync::CancellationToken;

pub struct IterResult {
    pub duration: Duration,
    pub status_code: u16,
    pub bytes: u64,
    pub is_error: bool,
}

pub struct HttpWorkerPool {
    config: HttpConfig,
    ramp_up: RampUpConfig,
    semaphore: Arc<Semaphore>,
    pause_rx: watch::Receiver<bool>,
    cancel: CancellationToken,
    result_tx: mpsc::UnboundedSender<IterResult>,
}

impl HttpWorkerPool {
    pub fn new(
        config: HttpConfig,
        ramp_up: RampUpConfig,
        pause_rx: watch::Receiver<bool>,
        cancel: CancellationToken,
        result_tx: mpsc::UnboundedSender<IterResult>,
    ) -> Self {
        let semaphore = Arc::new(Semaphore::new(ramp_up.start_concurrency as usize));
        Self {
            config,
            ramp_up,
            semaphore,
            pause_rx,
            cancel,
            result_tx,
        }
    }

    pub fn spawn(self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move { self.run().await })
    }

    async fn run(self) {
        let client = reqwest::Client::new();

        // Spawn ramp-up task
        let semaphore = self.semaphore.clone();
        let ramp_up = self.ramp_up.clone();
        let cancel = self.cancel.clone();
        let ramp_handle = tokio::spawn(async move {
            let step_size = ramp_up.concurrency_step_size() as usize;
            let step_duration = Duration::from_secs(ramp_up.step_duration_secs);
            for _step in 0..ramp_up.steps {
                tokio::select! {
                    _ = cancel.cancelled() => return,
                    _ = tokio::time::sleep(step_duration) => {
                        semaphore.add_permits(step_size);
                    }
                }
            }
        });

        // The initial workers
        let total_workers = self.ramp_up.end_concurrency as usize;
        let mut handles = Vec::with_capacity(total_workers);

        // Spawn all potential workers; they'll wait on the semaphore
        for _ in 0..total_workers {
            let client = client.clone();
            let config = self.config.clone();
            let semaphore = self.semaphore.clone();
            let mut pause_rx = self.pause_rx.clone();
            let cancel = self.cancel.clone();
            let result_tx = self.result_tx.clone();

            let handle = tokio::spawn(async move {
                loop {
                    // Check pause before acquiring
                    while *pause_rx.borrow() {
                        tokio::select! {
                            _ = cancel.cancelled() => return,
                            _ = pause_rx.changed() => {
                                if pause_rx.borrow().is_err() { return; }
                            }
                        }
                    }

                    tokio::select! {
                        _ = cancel.cancelled() => return,
                        permit = semaphore.acquire() => {
                            if let Ok(permit) = permit {
                                let start = Instant::now();
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
        cancel.cancelled().await;
        ramp_handle.abort();
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
        };

        let req = config.headers.iter().fold(req, |req, (k, v)| {
            req.header(k.as_str(), v.as_str())
        });

        let req = if matches!(config.method, HttpMethod::POST | HttpMethod::PUT) && !config.body.is_empty() {
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
```

- [ ] **Step 2: Add tokio-util dependency**

Edit `Cargo.toml`, add to `[dependencies]`:
```toml
tokio-util = "0.7"
```

- [ ] **Step 3: Build to verify**

Run: `cargo check`
Expected: builds successfully

- [ ] **Step 4: Write engine unit test**

The engine is async and depends on external HTTP, so write a test with a mock or use `httpbin.org`. Add to `src/engine.rs`:

```rust
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
        };
        let client = reqwest::Client::new();
        let result = HttpWorkerPool::execute_request(&client, &config).await;
        assert_eq!(result.status_code, 200);
        assert!(!result.is_error);
        assert!(result.duration.as_millis() > 0);
    }
}
```

Run: `cargo test engine`
Expected: test passes (requires internet to reach httpbin.org)

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml src/engine.rs
git commit -m "feat: HTTP worker pool engine with semaphore-based concurrency"
```

---

### Task 5: TestController (lifecycle management)

**Files:**
- Create: `src/control.rs`

- [ ] **Step 1: Write control.rs with TestController**

```rust
use crate::config::{AppConfig, RampUpConfig};
use crate::engine::{HttpWorkerPool, IterResult};
use crate::metrics::{MetricsCollector, MetricsSnapshot};
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, watch};
use tokio_util::sync::CancellationToken;

#[derive(Clone, Debug, PartialEq)]
pub enum TestState {
    Idle,
    Running,
    Paused,
    Stopped,
}

pub struct TestController {
    pub state: TestState,
    pub snapshot: Arc<RwLock<MetricsSnapshot>>,
    pause_tx: watch::Sender<bool>,
    cancel_token: Option<CancellationToken>,
    start_time: Option<Instant>,
}

impl TestController {
    pub fn new() -> Self {
        let (pause_tx, _) = watch::channel(false);
        Self {
            state: TestState::Idle,
            snapshot: Arc::new(RwLock::new(MetricsSnapshot::default())),
            pause_tx,
            cancel_token: None,
            start_time: None,
        }
    }

    pub fn start(
        &mut self,
        config: AppConfig,
        handle: &tokio::runtime::Handle,
    ) {
        self.stop_blocking(handle);

        let cancel = CancellationToken::new();
        let (result_tx, mut result_rx) = mpsc::unbounded_channel::<IterResult>();
        let pause_rx = self.pause_tx.subscribe();
        let (mut collector, snapshot_arc) = MetricsCollector::new();
        self.snapshot = snapshot_arc.clone();

        let pool = HttpWorkerPool::new(
            config.http,
            config.ramp_up.clone(),
            pause_rx,
            cancel.clone(),
            result_tx,
        );
        let worker_handle = pool.spawn();

        // Spawn metrics collection task
        let cancel_clone = cancel.clone();
        let ramp_up = config.ramp_up;
        let start = Instant::now();
        self.start_time = Some(start);

        handle.spawn(async move {
            let mut tick_interval = tokio::time::interval(Duration::from_millis(100));
            loop {
                tokio::select! {
                    _ = cancel_clone.cancelled() => break,
                    _ = tick_interval.tick() => {}
                }

                // Drain results
                while let Ok(result) = result_rx.try_recv() {
                    collector.record(result.status_code, result.duration, result.is_error);
                }

                // Compute step info
                let elapsed = start.elapsed();
                let step_dur = Duration::from_secs(ramp_up.step_duration_secs);
                let total_stages = ramp_up.total_stages();
                let current_step = {
                    let s = (elapsed.as_secs_f64() / step_dur.as_secs_f64()) as u32;
                    s.min(total_stages.saturating_sub(1))
                };
                let step_elapsed = elapsed.as_secs_f64() - (current_step as f64 * step_dur.as_secs_f64());
                let step_progress = (step_elapsed / step_dur.as_secs_f64()).min(1.0);

                let active = ramp_up.concurrency_at_stage(current_step);
                collector.tick(active, current_step, step_progress);
            }

            // Final tick
            while let Ok(result) = result_rx.try_recv() {
                collector.record(result.status_code, result.duration, result.is_error);
            }
            let elapsed = start.elapsed();
            let last_stage = ramp_up.total_stages().saturating_sub(1);
            collector.tick(ramp_up.end_concurrency, last_stage, 1.0);

            // Set elapsed for final snapshot
            snapshot_arc.write().elapsed = elapsed;
        });

        self.cancel_token = Some(cancel);
        self.state = TestState::Running;
    }

    pub fn pause(&mut self) {
        if self.state == TestState::Running {
            let _ = self.pause_tx.send(true);
            self.state = TestState::Paused;
        }
    }

    pub fn resume(&mut self) {
        if self.state == TestState::Paused {
            let _ = self.pause_tx.send(false);
            self.state = TestState::Running;
        }
    }

    pub fn stop(&mut self, handle: &tokio::runtime::Handle) {
        self.stop_blocking(handle);
        self.state = TestState::Stopped;
    }

    fn stop_blocking(&mut self, handle: &tokio::runtime::Handle) {
        if let Some(cancel) = self.cancel_token.take() {
            cancel.cancel();
            // Give a brief moment for tasks to clean up
            let _ = handle.block_on(async {
                tokio::time::sleep(Duration::from_millis(100)).await;
            });
        }
    }
}
```

- [ ] **Step 2: Build to verify**

Run: `cargo check`
Expected: builds successfully (may need to add `use` in app.rs)

- [ ] **Step 3: Commit**

```bash
git add src/control.rs
git commit -m "feat: TestController with start/pause/resume/stop lifecycle"
```

---

### Task 6: Theme module

**Files:**
- Create: `src/theme.rs`

- [ ] **Step 1: Write theme.rs with egui visual style setup**

```rust
use egui::Context;

pub fn setup_theme(ctx: &Context) {
    let mut style = (*ctx.style()).clone();

    style.visuals.window_rounding = egui::Rounding::same(6.0);
    style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(245, 245, 245);

    // Dark table rows for readability in charts
    style.visuals.striped = true;

    ctx.set_style(style);
}
```

- [ ] **Step 2: Commit**

```bash
git add src/theme.rs
git commit -m "feat: basic egui theme setup"
```

---

### Task 7: UI shell — app.rs and ui/mod.rs

**Files:**
- Modify: `src/app.rs`
- Modify: `src/ui/mod.rs`

- [ ] **Step 1: Write ui/mod.rs**

```rust
pub mod config_panel;
pub mod control_bar;
pub mod dashboard;
pub mod latency_chart;
pub mod stat_cards;
pub mod status_chart;
```

- [ ] **Step 2: Write app.rs with full App state and update loop**

```rust
use crate::config::AppConfig;
use crate::control::{TestController, TestState};
use crate::ui::{config_panel, control_bar, dashboard};
use crate::theme;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct RltvizApp {
    pub config: AppConfig,
    pub controller: TestController,
    handle: tokio::runtime::Handle,
}

impl RltvizApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, handle: tokio::runtime::Handle) -> Self {
        theme::setup_theme(&_cc.egui_ctx);
        Self {
            config: AppConfig::default(),
            controller: TestController::new(),
            handle,
        }
    }
}

impl eframe::App for RltvizApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let state = self.controller.state.clone();
        let snapshot = self.controller.snapshot.read().clone();

        egui::SidePanel::left("config_panel")
            .resizable(true)
            .default_width(380.0)
            .min_width(300.0)
            .show(ctx, |ui| {
                let running = state == TestState::Running || state == TestState::Paused;
                config_panel::show(ui, &mut self.config, running);
                ui.separator();
                control_bar::show(ui, &state, &mut self.controller, &self.config, &self.handle);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            dashboard::show(ui, &snapshot, &state);
        });

        // Request repaint for real-time metrics
        if state == TestState::Running || state == TestState::Paused {
            ctx.request_repaint();
        }
    }
}
```

- [ ] **Step 3: Create stub files for all UI modules to build-check**

`src/ui/config_panel.rs`:
```rust
use crate::config::AppConfig;

pub fn show(ui: &mut egui::Ui, config: &mut AppConfig, running: bool) {}
```

`src/ui/control_bar.rs`:
```rust
use crate::config::AppConfig;
use crate::control::{TestController, TestState};

pub fn show(ui: &mut egui::Ui, state: &TestState, controller: &mut TestController, config: &AppConfig, handle: &tokio::runtime::Handle) {}
```

`src/ui/dashboard.rs`:
```rust
use crate::control::TestState;
use crate::metrics::MetricsSnapshot;

pub fn show(ui: &mut egui::Ui, snapshot: &MetricsSnapshot, state: &TestState) {}
```

`src/ui/stat_cards.rs`:
```rust
use crate::metrics::MetricsSnapshot;

pub fn show(ui: &mut egui::Ui, snapshot: &MetricsSnapshot) {}
```

`src/ui/latency_chart.rs`:
```rust
use crate::metrics::MetricsSnapshot;

pub fn show(ui: &mut egui::Ui, snapshot: &MetricsSnapshot) {}
```

`src/ui/status_chart.rs`:
```rust
use crate::metrics::MetricsSnapshot;

pub fn show(ui: &mut egui::Ui, snapshot: &MetricsSnapshot) {}
```

Run: `cargo check`
Expected: builds successfully

- [ ] **Step 4: Commit**

```bash
git add src/app.rs src/ui/
git commit -m "feat: UI shell with app state and layout stubs"
```

---

### Task 8: UI — control_bar

**Files:**
- Modify: `src/ui/control_bar.rs`

- [ ] **Step 1: Write control_bar.rs**

```rust
use crate::config::AppConfig;
use crate::control::{TestController, TestState};

pub fn show(
    ui: &mut egui::Ui,
    state: &TestState,
    controller: &mut TestController,
    config: &AppConfig,
    handle: &tokio::runtime::Handle,
) {
    ui.horizontal(|ui| {
        match state {
            TestState::Idle | TestState::Stopped => {
                let start_btn = egui::Button::new("▶ 启动")
                    .fill(egui::Color32::from_rgb(76, 175, 80))
                    .min_size(egui::vec2(100.0, 32.0));

                if ui.add_enabled(true, start_btn).clicked() {
                    controller.start(config.clone(), handle);
                }
            }
            TestState::Running => {
                let pause_btn = egui::Button::new("⏸ 暂停")
                    .fill(egui::Color32::from_rgb(255, 152, 0))
                    .min_size(egui::vec2(100.0, 32.0));

                if ui.add(pause_btn).clicked() {
                    controller.pause();
                }

                let stop_btn = egui::Button::new("⏹ 停止")
                    .fill(egui::Color32::from_rgb(244, 67, 54))
                    .min_size(egui::vec2(100.0, 32.0));

                if ui.add(stop_btn).clicked() {
                    controller.stop(handle);
                }
            }
            TestState::Paused => {
                let resume_btn = egui::Button::new("▶ 恢复")
                    .fill(egui::Color32::from_rgb(76, 175, 80))
                    .min_size(egui::vec2(100.0, 32.0));

                if ui.add(resume_btn).clicked() {
                    controller.resume();
                }

                let stop_btn = egui::Button::new("⏹ 停止")
                    .fill(egui::Color32::from_rgb(244, 67, 54))
                    .min_size(egui::vec2(100.0, 32.0));

                if ui.add(stop_btn).clicked() {
                    controller.stop(handle);
                }
            }
        }
    });
}
```

- [ ] **Step 2: Build to verify**

Run: `cargo check`
Expected: builds

- [ ] **Step 3: Commit**

```bash
git add src/ui/control_bar.rs
git commit -m "feat: control bar with start/pause/resume/stop buttons"
```

---

### Task 9: UI — config_panel

**Files:**
- Modify: `src/ui/config_panel.rs`

- [ ] **Step 1: Write config_panel.rs**

```rust
use crate::config::{AppConfig, HttpMethod};

pub fn show(ui: &mut egui::Ui, config: &mut AppConfig, running: bool) {
    ui.heading("压测配置");
    ui.add_space(8.0);

    ui.set_enabled(!running);

    // URL
    ui.label("目标 URL");
    ui.add(
        egui::TextEdit::singleline(&mut config.http.url)
            .hint_text("https://api.example.com/endpoint")
            .desired_width(f32::INFINITY),
    );
    ui.add_space(6.0);

    // Method
    ui.horizontal(|ui| {
        ui.label("HTTP Method");
        egui::ComboBox::from_id_salt("http_method")
            .selected_text(config.http.method.as_str())
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut config.http.method, HttpMethod::GET, "GET");
                ui.selectable_value(&mut config.http.method, HttpMethod::POST, "POST");
                ui.selectable_value(&mut config.http.method, HttpMethod::PUT, "PUT");
                ui.selectable_value(&mut config.http.method, HttpMethod::DELETE, "DELETE");
            });
    });
    ui.add_space(6.0);

    // Headers
    ui.label("Headers");
    let mut remove_idx = None;
    for (i, (key, value)) in config.http.headers.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(key).hint_text("Key").desired_width(120.0));
            ui.add(egui::TextEdit::singleline(value).hint_text("Value").desired_width(160.0));
            if ui.button("✕").clicked() {
                remove_idx = Some(i);
            }
        });
    }
    if let Some(i) = remove_idx {
        config.http.headers.remove(i);
    }
    if ui.button("+ 添加 Header").clicked() {
        config.http.headers.push((String::new(), String::new()));
    }
    ui.add_space(6.0);

    // Body
    if matches!(config.http.method, HttpMethod::POST | HttpMethod::PUT) {
        ui.label("Request Body");
        ui.add(
            egui::TextEdit::multiline(&mut config.http.body)
                .hint_text("{\"key\": \"value\"}")
                .desired_width(f32::INFINITY)
                .desired_rows(4),
        );
        ui.add_space(6.0);
    }

    ui.separator();

    // Ramp-up
    ui.heading("阶梯加压");
    ui.add_space(4.0);

    ui.horizontal(|ui| {
        ui.label("起始并发");
        ui.add(egui::DragValue::new(&mut config.ramp_up.start_concurrency).range(1..=10000));
    });
    ui.horizontal(|ui| {
        ui.label("最终并发");
        ui.add(egui::DragValue::new(&mut config.ramp_up.end_concurrency).range(1..=10000));
    });
    ui.horizontal(|ui| {
        ui.label("阶梯数");
        ui.add(egui::DragValue::new(&mut config.ramp_up.steps).range(0..=100));
    });
    ui.horizontal(|ui| {
        ui.label("每阶时长(秒)");
        ui.add(egui::DragValue::new(&mut config.ramp_up.step_duration_secs).range(1..=3600));
    });

    ui.add_space(6.0);

    // Preview
    ui.label(format!(
        "共 {} 阶段，总计 {} 秒",
        config.ramp_up.total_stages(),
        config.ramp_up.total_duration_secs()
    ));

    // Simple bar chart preview
    let total_stages = config.ramp_up.total_stages();
    if total_stages > 0 {
        let available_width = ui.available_width();
        let bar_height = 40.0;
        let spacing = 2.0;
        let bar_width = (available_width / total_stages as f32) - spacing;

        ui.add_space(4.0);
        let (response, painter) = ui.allocate_painter(
            egui::vec2(available_width, bar_height + 16.0),
            egui::Sense::hover(),
        );
        let rect = response.rect;

        for i in 0..total_stages {
            let concurrency = config.ramp_up.concurrency_at_stage(i);
            let max_conc = config.ramp_up.end_concurrency.max(1) as f32;
            let fraction = concurrency as f32 / max_conc;
            let x = rect.left() + i as f32 * (bar_width + spacing);
            let h = fraction * bar_height;
            let y = rect.bottom() - 16.0 - h;

            let color = egui::Color32::from_rgb(
                (50 + (205.0 * fraction) as u8).min(255),
                (180 - (100.0 * fraction) as u8).max(50),
                (100 + (50.0 * fraction) as u8).min(180),
            );

            painter.rect_filled(
                egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(bar_width, h)),
                egui::Rounding::same(2.0),
                color,
            );

            if total_stages <= 10 {
                painter.text(
                    egui::pos2(x + bar_width / 2.0, rect.bottom() - 4.0),
                    egui::Align2::CENTER_BOTTOM,
                    format!("{}", concurrency),
                    egui::FontId::new(10.0, egui::FontFamily::Proportional),
                    egui::Color32::GRAY,
                );
            }
        }
    }

    ui.set_enabled(true);
}
```

- [ ] **Step 2: Build to verify**

Run: `cargo check`
Expected: builds

- [ ] **Step 3: Commit**

```bash
git add src/ui/config_panel.rs
git commit -m "feat: config panel with URL, method, headers, body, and ramp-up params"
```

---

### Task 10: UI — stat_cards

**Files:**
- Modify: `src/ui/stat_cards.rs`

- [ ] **Step 1: Write stat_cards.rs**

```rust
use crate::metrics::MetricsSnapshot;

pub fn show(ui: &mut egui::Ui, snapshot: &MetricsSnapshot) {
    ui.horizontal(|ui| {
        stat_card(ui, "QPS", &format!("{:.0}", snapshot.qps), egui::Color32::from_rgb(33, 150, 243));
        stat_card(
            ui,
            "错误率",
            &format!("{:.1}%", snapshot.error_rate * 100.0),
            if snapshot.error_rate > 0.05 {
                egui::Color32::from_rgb(244, 67, 54)
            } else {
                egui::Color32::from_rgb(76, 175, 80)
            },
        );
        stat_card(
            ui,
            "活跃连接",
            &format!("{}", snapshot.active_connections),
            egui::Color32::from_rgb(156, 39, 176),
        );
        stat_card(
            ui,
            "总请求",
            &format!("{}", snapshot.total_requests),
            egui::Color32::from_rgb(255, 152, 0),
        );
    });
}

fn stat_card(ui: &mut egui::Ui, label: &str, value: &str, color: egui::Color32) {
    let frame = egui::Frame::none()
        .fill(egui::Color32::from_rgb(250, 250, 250))
        .rounding(egui::Rounding::same(6.0))
        .inner_margin(egui::Margin::symmetric(12.0, 8.0));

    frame.show(ui, |ui| {
        ui.set_min_width(100.0);
        ui.label(
            egui::RichText::new(value)
                .color(color)
                .size(22.0)
                .strong(),
        );
        ui.label(egui::RichText::new(label).size(12.0).color(egui::Color32::GRAY));
    });
}
```

- [ ] **Step 2: Build to verify**

Run: `cargo check`
Expected: builds

- [ ] **Step 3: Commit**

```bash
git add src/ui/stat_cards.rs
git commit -m "feat: stat cards showing QPS, error rate, connections, total requests"
```

---

### Task 11: UI — latency_chart

**Files:**
- Modify: `src/ui/latency_chart.rs`

- [ ] **Step 1: Write latency_chart.rs**

```rust
use crate::metrics::MetricsSnapshot;

pub fn show(ui: &mut egui::Ui, snapshot: &MetricsSnapshot) {
    ui.heading("延迟分布 (ms)");

    let p50_ms = snapshot.latency_p50.as_secs_f64() * 1000.0;
    let p90_ms = snapshot.latency_p90.as_secs_f64() * 1000.0;
    let p99_ms = snapshot.latency_p99.as_secs_f64() * 1000.0;

    // Build static bar chart showing P50/P90/P99
    let bars = [
        ("P50", p50_ms, egui::Color32::from_rgb(76, 175, 80)),
        ("P90", p90_ms, egui::Color32::from_rgb(255, 152, 0)),
        ("P99", p99_ms, egui::Color32::from_rgb(244, 67, 54)),
    ];

    let available_width = ui.available_width();
    let chart_height = 200.0;
    let spacing = 16.0;
    let bar_width = (available_width - spacing * 4.0) / 3.0;

    let (response, painter) = ui.allocate_painter(
        egui::vec2(available_width, chart_height + 20.0),
        egui::Sense::hover(),
    );
    let rect = response.rect;
    let max_bar_h = chart_height - 20.0;
    let max_val = p99_ms.max(1.0) * 1.2;

    for (i, (label, val, color)) in bars.iter().enumerate() {
        let x = rect.left() + spacing + i as f32 * (bar_width + spacing);
        let fraction = (*val / max_val) as f32;
        let h = fraction * max_bar_h;
        let y = rect.bottom() - 20.0 - h;

        painter.rect_filled(
            egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(bar_width, h)),
            egui::Rounding::same(4.0),
            *color,
        );

        painter.text(
            egui::pos2(x + bar_width / 2.0, y - 4.0),
            egui::Align2::CENTER_BOTTOM,
            format!("{:.1}", val),
            egui::FontId::new(13.0, egui::FontFamily::Proportional),
            *color,
        );

        painter.text(
            egui::pos2(x + bar_width / 2.0, rect.bottom() - 6.0),
            egui::Align2::CENTER_BOTTOM,
            *label,
            egui::FontId::new(12.0, egui::FontFamily::Proportional),
            egui::Color32::GRAY,
        );
    }

    ui.add_space(4.0);
    ui.label(format!("已运行: {:.0}s | 当前阶梯: {}/{}", snapshot.elapsed.as_secs_f64(), snapshot.current_step + 1, snapshot.step_progress));

    // Step progress bar
    ui.add(egui::ProgressBar::new(snapshot.step_progress as f32).text(format!("阶梯进度 {:.0}%", snapshot.step_progress * 100.0)));
}
```

- [ ] **Step 2: Build to verify**

Run: `cargo check`
Expected: builds

- [ ] **Step 3: Commit**

```bash
git add src/ui/latency_chart.rs
git commit -m "feat: latency chart showing P50/P90/P99 as bar chart with step progress"
```

---

### Task 12: UI — status_chart

**Files:**
- Modify: `src/ui/status_chart.rs`

- [ ] **Step 1: Write status_chart.rs**

```rust
use crate::metrics::MetricsSnapshot;

pub fn show(ui: &mut egui::Ui, snapshot: &MetricsSnapshot) {
    ui.heading("状态码分布");

    if snapshot.status_codes.is_empty() {
        ui.label(egui::RichText::new("等待数据...").color(egui::Color32::GRAY));
        return;
    }

    let mut codes: Vec<(u16, u64)> = snapshot.status_codes.iter().map(|(k, v)| (*k, *v)).collect();
    codes.sort_by_key(|(k, _)| *k);

    let total: u64 = codes.iter().map(|(_, v)| v).sum();
    let available_width = ui.available_width();
    let bar_height = 24.0;
    let gap = 4.0;

    let (response, painter) = ui.allocate_painter(
        egui::vec2(available_width, codes.len() as f32 * (bar_height + gap) + 10.0),
        egui::Sense::hover(),
    );
    let rect = response.rect;

    for (i, (code, count)) in codes.iter().enumerate() {
        let fraction = if total > 0 { *count as f32 / total as f32 } else { 0.0 };
        let bar_w = fraction * (available_width - 80.0);
        let y = rect.top() + i as f32 * (bar_height + gap);

        let color = match code / 100 {
            2 => egui::Color32::from_rgb(76, 175, 80),
            3 => egui::Color32::from_rgb(33, 150, 243),
            4 => egui::Color32::from_rgb(255, 152, 0),
            5 => egui::Color32::from_rgb(244, 67, 54),
            _ => egui::Color32::GRAY,
        };

        // Label
        painter.text(
            egui::pos2(rect.left(), y + bar_height / 2.0),
            egui::Align2::LEFT_CENTER,
            format!("{}", code),
            egui::FontId::new(12.0, egui::FontFamily::Proportional),
            egui::Color32::WHITE,
        );

        // Bar
        if bar_w > 0.0 {
            painter.rect_filled(
                egui::Rect::from_min_size(egui::pos2(rect.left() + 55.0, y + 2.0), egui::vec2(bar_w, bar_height - 4.0)),
                egui::Rounding::same(3.0),
                color,
            );
        }

        // Count
        painter.text(
            egui::pos2(rect.left() + 55.0 + bar_w + 4.0, y + bar_height / 2.0),
            egui::Align2::LEFT_CENTER,
            format!("{} ({:.1}%)", count, fraction * 100.0),
            egui::FontId::new(11.0, egui::FontFamily::Proportional),
            egui::Color32::GRAY,
        );
    }
}
```

- [ ] **Step 2: Build to verify**

Run: `cargo check`
Expected: builds

- [ ] **Step 3: Commit**

```bash
git add src/ui/status_chart.rs
git commit -m "feat: status code distribution bar chart"
```

---

### Task 13: UI — dashboard layout

**Files:**
- Modify: `src/ui/dashboard.rs`

- [ ] **Step 1: Write dashboard.rs**

```rust
use crate::control::TestState;
use crate::metrics::MetricsSnapshot;
use crate::ui::{latency_chart, stat_cards, status_chart};

pub fn show(ui: &mut egui::Ui, snapshot: &MetricsSnapshot, state: &TestState) {
    ui.heading("实时指标");

    if *state == TestState::Idle {
        ui.add_space(40.0);
        ui.vertical_centered(|ui| {
            ui.label(
                egui::RichText::new("配置压测参数后点击 ▶ 启动")
                    .size(18.0)
                    .color(egui::Color32::GRAY),
            );
        });
        return;
    }

    stat_cards::show(ui, snapshot);

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(8.0);

    latency_chart::show(ui, snapshot);

    ui.add_space(8.0);
    ui.separator();
    ui.add_space(8.0);

    status_chart::show(ui, snapshot);

    if *state == TestState::Stopped {
        ui.add_space(16.0);
        ui.separator();
        ui.add_space(8.0);
        ui.heading("压测完成");
        ui.label(format!("总请求数: {}", snapshot.total_requests));
        ui.label(format!("QPS: {:.1}", snapshot.qps));
        ui.label(format!("错误率: {:.1}%", snapshot.error_rate * 100.0));
        ui.label(format!(
            "延迟 P50/P90/P99: {:.1}ms / {:.1}ms / {:.1}ms",
            snapshot.latency_p50.as_secs_f64() * 1000.0,
            snapshot.latency_p90.as_secs_f64() * 1000.0,
            snapshot.latency_p99.as_secs_f64() * 1000.0,
        ));
        ui.label(format!("总耗时: {:.1}s", snapshot.elapsed.as_secs_f64()));
    }
}
```

- [ ] **Step 2: Build to verify**

Run: `cargo check`
Expected: builds

- [ ] **Step 3: Commit**

```bash
git add src/ui/dashboard.rs
git commit -m "feat: dashboard layout wiring stat cards, latency chart, and status chart"
```

---

### Task 14: Integration — build, run, and fix issues

**Files:**
- (check all files compile together)

- [ ] **Step 1: Full build**

Run: `cargo build`
Expected: builds with no errors

- [ ] **Step 2: Fix any compilation errors**

If `cargo build` fails, fix errors iteratively. Common issues:
- Missing `#[derive(Clone)]` on types shared across threads
- `parking_lot::RwLock` read signature differences vs std
- `egui_plot` API version differences — check actual API with `cargo doc`

- [ ] **Step 3: Run clippy**

Run: `cargo clippy -- -D warnings`
Expected: no warnings

- [ ] **Step 4: Run all tests**

Run: `cargo test`
Expected: all tests pass

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: full integration — rltviz builds and passes tests"
```

---

### Post-MVP ideas (NOT in scope, do not implement)

These are documented so future work can reference them:
- Time-series latency chart (scrolling line chart instead of static bar chart)
- Config import/export (JSON file)
- Request template library
- Multiple target URLs (comparison mode)
- Export results as JSON/CSV
- Custom egui_plot line chart with step markers (replacing current bar chart)
