use crate::config::AppConfig;
use crate::metrics::MetricsSnapshot;
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

const MAX_RECORDS: usize = 20;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ResultSummary {
    pub duration_secs: f64,
    pub total_requests: u64,
    pub qps: f64,
    pub error_rate: f64,
    pub p50_ms: f64,
    pub p90_ms: f64,
    pub p99_ms: f64,
    pub status_codes: HashMap<u16, u64>,
}

impl ResultSummary {
    pub fn from_snapshot(snapshot: &MetricsSnapshot) -> Self {
        Self {
            duration_secs: snapshot.elapsed.as_secs_f64(),
            total_requests: snapshot.total_requests,
            qps: snapshot.qps,
            error_rate: snapshot.error_rate,
            p50_ms: snapshot.latency_p50.as_secs_f64() * 1000.0,
            p90_ms: snapshot.latency_p90.as_secs_f64() * 1000.0,
            p99_ms: snapshot.latency_p99.as_secs_f64() * 1000.0,
            status_codes: snapshot.status_codes.clone(),
        }
    }

    pub fn to_snapshot(&self, end_concurrency: u32) -> MetricsSnapshot {
        MetricsSnapshot {
            qps: self.qps,
            latency_p50: Duration::from_secs_f64(self.p50_ms / 1000.0),
            latency_p90: Duration::from_secs_f64(self.p90_ms / 1000.0),
            latency_p99: Duration::from_secs_f64(self.p99_ms / 1000.0),
            error_rate: self.error_rate,
            status_codes: self.status_codes.clone(),
            active_connections: end_concurrency,
            total_requests: self.total_requests,
            elapsed: Duration::from_secs_f64(self.duration_secs),
            current_step: 0,
            step_progress: 1.0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HistoryRecord {
    pub id: u64,
    pub timestamp: String,
    pub config: AppConfig,
    pub summary: ResultSummary,
}

impl HistoryRecord {
    pub fn new(config: AppConfig, summary: ResultSummary) -> Self {
        let now = Local::now();
        Self {
            id: now.timestamp_millis() as u64,
            timestamp: now.format("%Y-%m-%d %H:%M:%S").to_string(),
            config,
            summary,
        }
    }
}

pub struct HistoryStore {
    records: Vec<HistoryRecord>,
    path: PathBuf,
}

impl HistoryStore {
    pub fn load(path: PathBuf) -> Self {
        let records = match fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(records) => records,
                Err(e) => {
                    eprintln!("WARNING: Failed to parse history file {}: {}. Starting with empty history.", path.display(), e);
                    let backup = path.with_extension("json.corrupted");
                    let _ = fs::copy(&path, &backup);
                    Vec::new()
                }
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(e) => {
                eprintln!("WARNING: Could not read history file {}: {}. Starting with empty history.", path.display(), e);
                Vec::new()
            }
        };
        Self { records, path }
    }

    pub fn add(&mut self, record: HistoryRecord) {
        self.records.insert(0, record);
        self.records.truncate(MAX_RECORDS);
        self.save();
    }

    pub fn records(&self) -> &[HistoryRecord] {
        &self.records
    }

    fn save(&self) {
        let Ok(json) = serde_json::to_string_pretty(&self.records) else {
            return;
        };
        let tmp = self.path.with_extension("json.tmp");
        if fs::write(&tmp, &json).is_err() {
            return;
        }
        if let Err(e) = fs::rename(&tmp, &self.path) {
            eprintln!("WARNING: Failed to save history: {}", e);
        }
    }
}

pub fn history_path() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.join("history.json")))
        .unwrap_or_else(|| PathBuf::from("history.json"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn test_path() -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "rltviz_test_{}_{}.json",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_file(&path);
        path
    }

    fn test_summary(id: u64) -> ResultSummary {
        let mut codes = HashMap::new();
        codes.insert(200, 1000 - id);
        codes.insert(404, id);
        ResultSummary {
            duration_secs: 60.0,
            total_requests: 1000,
            qps: 1000.0 / 60.0,
            error_rate: id as f64 / 1000.0,
            p50_ms: 10.0 + id as f64,
            p90_ms: 20.0 + id as f64,
            p99_ms: 50.0 + id as f64,
            status_codes: codes,
        }
    }

    fn test_record(id: u64) -> HistoryRecord {
        HistoryRecord {
            id,
            timestamp: format!("2026-04-29 14:{:02}:00", id % 60),
            config: AppConfig::default(),
            summary: test_summary(id),
        }
    }

    #[test]
    fn test_load_missing_file_returns_empty() {
        let path = test_path();
        let store = HistoryStore::load(path);
        assert!(store.records().is_empty());
    }

    #[test]
    fn test_add_and_persist() {
        let path = test_path();
        let mut store = HistoryStore::load(path.clone());
        store.add(test_record(1));
        assert_eq!(store.records().len(), 1);
        assert_eq!(store.records()[0].id, 1);

        let store2 = HistoryStore::load(path);
        assert_eq!(store2.records().len(), 1);
        assert_eq!(store2.records()[0].id, 1);
    }

    #[test]
    fn test_newest_first() {
        let path = test_path();
        let mut store = HistoryStore::load(path);
        store.add(test_record(1));
        store.add(test_record(2));
        assert_eq!(store.records()[0].id, 2);
        assert_eq!(store.records()[1].id, 1);
    }

    #[test]
    fn test_cap_at_20() {
        let path = test_path();
        let mut store = HistoryStore::load(path);
        for i in 0..25 {
            store.add(test_record(i));
        }
        assert_eq!(store.records().len(), 20);
        assert_eq!(store.records()[0].id, 24);
        assert_eq!(store.records()[19].id, 5);
    }

    #[test]
    fn test_result_summary_serde_roundtrip() {
        let summary = test_summary(42);
        let json = serde_json::to_string(&summary).unwrap();
        let back: ResultSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(summary, back);
    }
}
