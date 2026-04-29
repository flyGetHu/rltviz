# History Feature Design

## Overview

Add test run history persistence to rltviz. Each completed load test auto-saves its configuration and result summary to a local JSON file. Users can browse history, review past metrics, and reuse configurations.

## Requirements

- Auto-save on test stop (no manual action)
- Keep last 20 records, auto-evict oldest
- Left panel uses Tab switching: [Config] / [History]
- Selecting a record fills the config form and displays historical metrics on the dashboard
- "Reuse and run" button: switch to Config tab + fill form + start test

## Data Model

### File Location

`./history.json` next to the executable. Auto-created as `[]` if missing.

### Structures

```rust
struct HistoryRecord {
    id: u64,                    // Unix timestamp as ID
    timestamp: String,          // "2026-04-29 14:35:31" for display
    config: AppConfig,          // Reuse existing serde-enabled struct
    summary: ResultSummary,
}

struct ResultSummary {
    duration_secs: f64,
    total_requests: u64,
    qps: f64,
    error_rate: f64,            // 0.0 - 1.0
    p50_ms: f64,
    p90_ms: f64,
    p99_ms: f64,
    status_codes: HashMap<u16, u64>,
}
```

### JSON File Format

Array sorted newest-first. Capped at 20 entries.

```json
[
  {"id": 1777449331, "timestamp": "2026-04-29 14:35:31", "config": {...}, "summary": {...}},
  {"id": 1777449200, "timestamp": "2026-04-29 14:33:20", "config": {...}, "summary": {...}}
]
```

### Eviction

After each `add()`, if `records.len() > 20`, truncate to first 20 (newest 20).

## Persistence Layer

### Approach

Synchronous file I/O. Read entire file on startup, write entire file on each save. With 20 small records, full-file rewrite is <1ms.

### HistoryStore API

```rust
struct HistoryStore {
    records: Vec<HistoryRecord>,
    path: PathBuf,
}

impl HistoryStore {
    fn load(path: PathBuf) -> Self;           // Read JSON; empty Vec if file missing
    fn save(&self) -> Result<()>;              // Write JSON
    fn add(&mut self, record: HistoryRecord);  // Prepend + cap at 20 + save
    fn records(&self) -> &[HistoryRecord];     // Read-only access
}
```

### MetricsSnapshot to ResultSummary Conversion

Done inside `add()`, extracting the final metrics from `MetricsSnapshot` at test stop time. The UI layer never sees raw `MetricsSnapshot` for history — only `ResultSummary`.

## UI Design

### Left Panel — Tab Switching

Tab bar at top of the left panel with two tabs: **Config** and **History**.

**Config tab**: Unchanged — existing URL/Method/Headers/Body/ramp-up form.

**History tab**: Scrollable list of record cards. Each card shows:
- HTTP method (color-coded: GET=cyan, POST=yellow, PUT=blue, DELETE=red)
- URL (truncated with ellipsis)
- Timestamp
- Key metrics row: QPS, P99, error rate

Selected card has accent-colored left border highlight.

### Right Dashboard — Historical Metrics Mode

When a history record is selected:
- Top bar shows timestamp + "历史记录" label + run duration
- Stat cards (QPS, error rate, concurrency, total requests) from `ResultSummary`
- Latency distribution (P50/P90/P99) from `ResultSummary`
- Status code distribution from `ResultSummary.status_codes`
- Bottom: "复用此配置并运行" button

When no history record is selected, dashboard behaves as before (idle/running/stopped states).

### Interaction Flow

1. Click **History** tab → shows history list
2. Click a record → record highlights, right dashboard shows historical metrics
3. Click "复用此配置并运行" → switch to Config tab (form filled) + auto-start test
4. Test stops → auto-save new record → history list updates

## Code Changes

### New File

| File | Purpose |
|------|---------|
| `src/history.rs` | `HistoryRecord`, `ResultSummary`, `HistoryStore` — data structures and file I/O |

### Modified Files

| File | Change |
|------|--------|
| `src/app.rs` | Add `history_store: HistoryStore`, `active_tab: PanelTab` (Config/History enum), `selected_history: Option<usize>`. On test stop, call `history_store.add()`. |
| `src/ui/config_panel.rs` | Render Tab bar above existing form. In History tab, render scrollable card list. Handle selection clicks. |
| `src/ui/dashboard.rs` | Add history metrics rendering mode. When `selected_history` is Some, render stat cards/latency/status from `ResultSummary`. |
| `src/ui/mod.rs` | Expose new types if needed. |
| `src/main.rs` | Add `mod history;` declaration. |

### Unchanged Files

`engine.rs`, `metrics.rs`, `control.rs`, `theme.rs`, `ui/control_bar.rs`, `ui/stat_cards.rs`, `ui/latency_chart.rs`, `ui/status_chart.rs`.
