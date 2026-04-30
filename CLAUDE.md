# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test Commands

```bash
cargo build               # debug build
cargo build --release     # release build (with embedded fonts)
cargo run --release       # run the GUI application
cargo test                # all tests (44 total)
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
| history | `history.rs` | Persistent test result records (JSON, capped at 20) |
| control_bar | `ui/control_bar.rs` | Start/Pause/Resume/Stop buttons |
| dashboard | `ui/dashboard.rs` | Right panel layout, idle/running/stopped states |
| stat_cards | `ui/stat_cards.rs` | QPS/error rate/connections/total cards |
| latency_chart | `ui/latency_chart.rs` | P50/P90/P99 bar chart + step progress |
| status_chart | `ui/status_chart.rs` | Status code horizontal bars |
| curl_import | `curl_import.rs` | cURL command parsing, normalization, and import |
| config | `config.rs` | Request configuration model (method, URL, headers, body) |
| theme | `theme.rs` | Custom color scheme and egui visual style |

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
- **Atomic persistence**: `HistoryStore::save()` writes to `.json.tmp` then `fs::rename` — never direct write
- **State transition detection**: `prev_state` field tracks last frame's `TestState` to detect transitions (Running→Stopped triggers auto-save)

## CI/Release

`.github/workflows/release.yml` — cross-platform build (Windows/macOS ARM). Triggered on push to master. Produces GitHub Release with platform binaries.

## Gotchas

- **egui Window popup**: `Window::open(&mut bool)` borrows the bool — mutation must go through `Cell<bool>` from outside the closure
- **curl-parser crate**: Only supports short-form flags (`-X`, `-H`, `-d`). Long-form `--request`/`--url`/`--header`/`--data` must be preprocessed via `normalize_curl()`
- **FontData**: `FontData::from_owned(Vec<u8>)` returns `FontData`, must be wrapped in `Arc` for the `font_data` map
- **Flaky test**: `engine::tests::test_execute_request_404` hits httpbin.org — may fail transiently due to DNS/network
- **History file location**: Stored next to the executable (`current_exe().parent()/history.json`), not in a platform data directory
