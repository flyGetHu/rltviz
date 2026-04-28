# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test Commands

```bash
cargo build               # debug build
cargo build --release     # release build (with embedded fonts)
cargo run --release       # run the GUI application
cargo test                # all tests (14 total)
cargo test <module>       # e.g., cargo test config, cargo test engine
cargo test <test_name>    # single test (e.g., cargo test test_percentiles)
cargo clippy -- -D warnings  # lint with no warnings allowed
cargo check               # fast compile check
```

## Architecture

Desktop GUI HTTP load testing tool built with egui 0.31 + Tokio. Two-window layout: left config panel (~35%), right metrics dashboard.

### Layer Separation (3 layers)

**GUI layer** (`src/ui/`) — Pure egui rendering, reads from shared `Arc<RwLock<MetricsSnapshot>>`:

| Component | File | Responsibility |
|-----------|------|---------------|
| config_panel | `ui/config_panel.rs` | URL/Method/Headers/Body/ramp-up form |
| control_bar | `ui/control_bar.rs` | Start/Pause/Resume/Stop buttons |
| dashboard | `ui/dashboard.rs` | Right panel layout, idle/running/stopped states |
| stat_cards | `ui/stat_cards.rs` | QPS/error rate/connections/total cards |
| latency_chart | `ui/latency_chart.rs` | P50/P90/P99 bar chart + step progress |
| status_chart | `ui/status_chart.rs` | Status code horizontal bars |

**AppCore layer** (`src/control.rs`, `src/metrics.rs`) — Orchestration and aggregation:
- `TestController` manages lifecycle: start/pause/resume/stop, ramp-up timer, auto-stop after final stage
- `MetricsCollector` records per-request data, computes QPS/percentiles/error rate on 100ms tick

**Engine layer** (`src/engine.rs`) — Async HTTP worker pool:
- `tokio::sync::Semaphore` for dynamic concurrency (add_permits for ramp-up)
- `watch::Receiver<bool>` for pause/cancel signals
- All workers spawned upfront; semaphore gates active count

### Data Flow

```
HTTP workers (engine.rs) ──mpsc channel──► MetricsCollector (metrics.rs) ──tick()──► Arc<RwLock<MetricsSnapshot>>
                                                                                              │
                                                                                              ▼
                                                                                egui::App::update() reads each frame
```

### State Machine

```
Idle ──start()──► Running ──pause()──► Paused ──resume()──► Running
                    │                                        │
                    └─────auto-stop (ramp-up done) ──────────┘
                    │                                        │
                    └────────stop() ──────────────────────────┘
                                                              ▼
                                                           Stopped
```

### Fonts (embedded)
- `fonts/JetBrainsMono-Regular.ttf` — monospace text
- `fonts/JetBrainsMono-Bold.ttf` — bold monospace
- `fonts/NotoSansSC-Regular.ttf` — CJK character fallback (11MB)

### Key Design Decisions
- **No tokio-util**: Cancellation uses `watch::Receiver<bool>` instead of CancellationToken
- **Per-run fresh channels**: Each `start()` creates new watch/mpsc channels (clean state)
- **add_enabled_ui**: UI uses `ui.add_enabled_ui(!running, |ui| ...)` for disabled-state gating
- **egui 0.31 API**: `CornerRadius::same(N)`, `Frame::NONE`, no `Rounding`/`set_enabled`
