use crate::config::AppConfig;
use crate::engine::{HttpWorkerPool, IterResult};
use crate::metrics::{MetricsCollector, MetricsSnapshot};
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, watch};

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
    cancel_tx: watch::Sender<bool>,
    start_time: Option<Instant>,
}

impl TestController {
    pub fn new() -> Self {
        let (pause_tx, _) = watch::channel(false);
        let (cancel_tx, _) = watch::channel(false);
        Self {
            state: TestState::Idle,
            snapshot: Arc::new(RwLock::new(MetricsSnapshot::default())),
            pause_tx,
            cancel_tx,
            start_time: None,
        }
    }

    pub fn start(&mut self, config: AppConfig, handle: &tokio::runtime::Handle) {
        // Cancel any previous run
        let _ = self.cancel_tx.send(true);

        // Create new channels for this run
        let (pause_tx, pause_rx) = watch::channel(false);
        let (cancel_tx, cancel_rx) = watch::channel(false);
        let (result_tx, mut result_rx) = mpsc::unbounded_channel::<IterResult>();
        self.pause_tx = pause_tx;
        self.cancel_tx = cancel_tx;
        let (mut collector, snapshot_arc) = MetricsCollector::new();
        self.snapshot = snapshot_arc.clone();

        let pool = HttpWorkerPool::new(
            config.http.clone(),
            config.ramp_up.start_concurrency as usize,
            pause_rx,
            cancel_rx.clone(),
            result_tx,
        );

        let ramp_up = config.ramp_up.clone();
        let semaphore = pool.semaphore();
        let start = Instant::now();
        self.start_time = Some(start);

        // Spawn all workers upfront
        let total_workers = ramp_up.end_concurrency as usize;
        let _worker_handle = pool.spawn(total_workers);

        // Spawn ramp-up task
        let ramp_up_metrics = ramp_up.clone();
        let cancel_rx_clone = cancel_rx.clone();
        handle.spawn(async move {
            let step_size = ramp_up.concurrency_step_size() as usize;
            let step_duration = Duration::from_secs(ramp_up.step_duration_secs);
            for _step in 0..ramp_up.steps {
                let mut cancel_rx = cancel_rx_clone.clone();
                tokio::select! {
                    _ = cancel_rx.changed() => {
                        if *cancel_rx.borrow() { return; }
                    }
                    _ = tokio::time::sleep(step_duration) => {
                        semaphore.add_permits(step_size);
                    }
                }
            }
        });

        // Spawn metrics collection task
        let cancel_rx_clone = cancel_rx.clone();
        handle.spawn(async move {
            let mut tick_interval = tokio::time::interval(Duration::from_millis(100));
            let mut cancel_rx = cancel_rx_clone;
            loop {
                tokio::select! {
                    _ = cancel_rx.changed() => {
                        if *cancel_rx.borrow() { break; }
                    }
                    _ = tick_interval.tick() => {}
                }

                // Drain results
                while let Ok(result) = result_rx.try_recv() {
                    collector.record(result.status_code, result.duration, result.is_error);
                }

                // Compute step info
                let elapsed = start.elapsed();
                let step_dur = Duration::from_secs(ramp_up_metrics.step_duration_secs);
                let total_stages = ramp_up_metrics.total_stages();
                let current_step = {
                    let s = (elapsed.as_secs_f64() / step_dur.as_secs_f64()) as u32;
                    s.min(total_stages.saturating_sub(1))
                };
                let step_elapsed = elapsed.as_secs_f64()
                    - (current_step as f64 * step_dur.as_secs_f64());
                let step_progress = (step_elapsed / step_dur.as_secs_f64()).min(1.0);

                let active = ramp_up_metrics.concurrency_at_stage(current_step);
                collector.tick(active, current_step, step_progress);
            }

            // Final tick
            while let Ok(result) = result_rx.try_recv() {
                collector.record(result.status_code, result.duration, result.is_error);
            }
            let elapsed = start.elapsed();
            let last_stage = ramp_up_metrics.total_stages().saturating_sub(1);
            collector.tick(ramp_up_metrics.end_concurrency, last_stage, 1.0);
            snapshot_arc.write().elapsed = elapsed;
        });

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

    pub fn stop(&mut self) {
        let _ = self.cancel_tx.send(true);
        self.state = TestState::Stopped;
    }
}
