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

    #[test]
    fn test_no_samples() {
        let mut empty: Vec<Duration> = vec![];
        let (p50, p90, p99) = MetricsCollector::compute_percentiles(&mut empty);
        assert_eq!(p50, Duration::ZERO);
        assert_eq!(p90, Duration::ZERO);
        assert_eq!(p99, Duration::ZERO);
    }
}
