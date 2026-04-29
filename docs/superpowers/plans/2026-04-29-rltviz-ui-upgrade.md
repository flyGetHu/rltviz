# rltviz UI 全面升级实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将 rltviz 从 Material 风格 UI 升级为 Apple 浅色简洁风格 — 单色灰阶 + Blue 强调色、留白分层、统一排版

**Architecture:** 修改流程从底层 token 开始，自下而上。先建 theme 模块（palette + 排版 + 全局 visuals），再升级面板布局，最后逐个改组件。每个 task 可独立验证。

**Tech Stack:** egui 0.31, eframe 0.31, Rust 2021 edition

---

## 文件结构

| 文件 | 角色 | 改动类型 |
|---|---|---|
| `src/theme.rs` | 色彩常量、排版 helper、全局 visuals 设置 | 重写 |
| `src/app.rs` | 左右面板布局、frame 配置 | 修改 |
| `src/ui/control_bar.rs` | 按钮样式（启动/暂停/停止） | 修改 |
| `src/ui/config_panel.rs` | 左侧配置表单、cURL 导入弹窗 | 修改 |
| `src/ui/stat_cards.rs` | 右侧 4 个指标卡片 | 修改 |
| `src/ui/dashboard.rs` | 右侧面板总装、间距、Idle/Stopped 状态 | 修改 |
| `src/ui/latency_chart.rs` | 延迟柱状图（自定义 painter） | 修改 |
| `src/ui/status_chart.rs` | 状态码分布图（自定义 painter） | 修改 |
| `src/ui/mod.rs` | 模块声明 | 不修改 |
| `src/config.rs` | 配置数据模型 | 不修改 |
| `src/control.rs` | 测试生命周期 | 不修改 |
| `src/engine.rs` | HTTP worker 池 | 不修改 |
| `src/metrics.rs` | 指标收集 | 不修改 |

---

### Task 1: theme.rs — 色彩 palette + 全局样式

**Files:**
- Modify: `src/theme.rs`

- [ ] **Step 1: Write the new theme.rs**

```rust
use egui::{Color32, Context, CornerRadius, Stroke};
use std::sync::Arc;

// ── Color palette ──────────────────────────────────────────

pub const BG_PRIMARY: Color32 = Color32::from_rgb(0xFF, 0xFF, 0xFF);
pub const BG_SECONDARY: Color32 = Color32::from_rgb(0xF5, 0xF5, 0xF7);
pub const BG_TERTIARY: Color32 = Color32::from_rgb(0xEB, 0xEB, 0xED);
pub const BORDER: Color32 = Color32::from_rgb(0xD1, 0xD1, 0xD6);
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(0x1D, 0x1D, 0x1F);
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(0x6E, 0x6E, 0x73);
pub const TEXT_TERTIARY: Color32 = Color32::from_rgb(0xAE, 0xAE, 0xB2);
pub const ACCENT: Color32 = Color32::from_rgb(0x00, 0x7A, 0xFF);
pub const ACCENT_HOVER: Color32 = Color32::from_rgb(0x00, 0x62, 0xCC);
pub const POSITIVE: Color32 = Color32::from_rgb(0x34, 0xC7, 0x59);
pub const NEGATIVE: Color32 = Color32::from_rgb(0xFF, 0x3B, 0x30);
pub const WARNING: Color32 = Color32::from_rgb(0xFF, 0x95, 0x00);

// ── Typography helpers ─────────────────────────────────────

pub fn heading(text: &str) -> egui::RichText {
    egui::RichText::new(text)
        .size(15.0)
        .color(TEXT_PRIMARY)
        .strong()
}

pub fn body(text: &str) -> egui::RichText {
    egui::RichText::new(text)
        .size(13.0)
        .color(TEXT_PRIMARY)
}

pub fn body_small(text: &str) -> egui::RichText {
    egui::RichText::new(text)
        .size(11.0)
        .color(TEXT_SECONDARY)
}

pub fn metric_value(val: &str, color: Color32) -> egui::RichText {
    egui::RichText::new(val)
        .size(28.0)
        .color(color)
        .strong()
}

pub fn metric_label(text: &str) -> egui::RichText {
    egui::RichText::new(text)
        .size(11.0)
        .color(TEXT_TERTIARY)
}

pub fn mono(text: &str) -> egui::RichText {
    egui::RichText::new(text)
        .size(12.0)
        .color(TEXT_PRIMARY)
        .font(egui::FontId::monospace(12.0))
}

// ── Global theme application ───────────────────────────────

pub fn apply_theme(ctx: &Context) {
    let mut style = (*ctx.style()).clone();
    let visuals = &mut style.visuals;

    // Panel backgrounds
    visuals.window_fill = BG_PRIMARY;
    visuals.panel_fill = BG_SECONDARY;

    // Widget defaults
    let rounding = CornerRadius::same(6);
    visuals.widgets.noninteractive.rounding = rounding;
    visuals.widgets.inactive.rounding = rounding;
    visuals.widgets.hovered.rounding = rounding;
    visuals.widgets.active.rounding = rounding;

    visuals.widgets.noninteractive.bg_fill = BG_PRIMARY;
    visuals.widgets.inactive.bg_fill = BG_TERTIARY;
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(0xE0, 0xE0, 0xE5);
    visuals.widgets.active.bg_fill = ACCENT;

    let subtle_border = Stroke::new(1.0, BORDER);
    visuals.widgets.inactive.bg_stroke = subtle_border;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, ACCENT);
    visuals.widgets.active.bg_stroke = Stroke::new(2.0, ACCENT);

    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);
    visuals.widgets.active.fg_stroke = Stroke::new(2.0, ACCENT);

    // Selection color
    visuals.selection.bg_fill = Color32::from_rgba_premultiplied(0x00, 0x7A, 0xFF, 0x40);

    // Hyperlink
    visuals.hyperlink_color = ACCENT;

    // No stripes, no boxed button frames
    visuals.striped = false;
    visuals.button_frame = false;
    visuals.indent_has_left_vline = false;

    // Window (popup) corner radius
    visuals.window_corner_radius = CornerRadius::same(8);
    visuals.window_stroke = Stroke::new(1.0, BORDER);

    // Default text color override — softer black
    visuals.override_text_color = Some(TEXT_PRIMARY);

    ctx.set_style(style);

    setup_fonts(ctx);
}

fn setup_fonts(ctx: &Context) {
    let mut fonts = egui::FontDefinitions::empty();

    let jb_regular: &[u8] = include_bytes!("../fonts/JetBrainsMono-Regular.ttf");
    let jb_bold: &[u8] = include_bytes!("../fonts/JetBrainsMono-Bold.ttf");
    fonts.font_data
        .insert("JetBrainsMono-Regular".into(), Arc::new(egui::FontData::from_static(jb_regular)));
    fonts.font_data
        .insert("JetBrainsMono-Bold".into(), Arc::new(egui::FontData::from_static(jb_bold)));

    let noto_sc: &[u8] = include_bytes!("../fonts/NotoSansSC-Regular.ttf");
    fonts.font_data
        .insert("NotoSansSC".into(), Arc::new(egui::FontData::from_static(noto_sc)));

    fonts.families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .push("NotoSansSC".into());

    fonts.families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "JetBrainsMono-Regular".into());
    fonts.families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("NotoSansSC".into());

    ctx.set_fonts(fonts);
}
```

- [ ] **Step 2: Build check**

```bash
cargo check 2>&1
```
Expected: compiles without errors

- [ ] **Step 3: Commit**

```bash
git add src/theme.rs
git commit -m "feat(ui): add Apple-style color palette, typography helpers, and global visuals

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

### Task 2: app.rs — 面板 frame 与布局调整

**Files:**
- Modify: `src/app.rs`

- [ ] **Step 1: Update app.rs with new panel configuration**

Replace the entire `update` function in `src/app.rs`:

```rust
use crate::config::AppConfig;
use crate::control::{TestController, TestState};
use crate::theme;
use crate::ui::{config_panel, control_bar, dashboard};

pub struct RltvizApp {
    pub config: AppConfig,
    pub controller: TestController,
    handle: tokio::runtime::Handle,
    curl_import_open: bool,
    curl_import_text: String,
    curl_import_error: Option<String>,
}

impl RltvizApp {
    pub fn new(cc: &eframe::CreationContext<'_>, handle: tokio::runtime::Handle) -> Self {
        theme::apply_theme(&cc.egui_ctx);
        Self {
            config: AppConfig::default(),
            controller: TestController::new(),
            handle,
            curl_import_open: false,
            curl_import_text: String::new(),
            curl_import_error: None,
        }
    }
}

impl eframe::App for RltvizApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.controller.check_done();

        let state = self.controller.state.clone();
        let snapshot = self.controller.snapshot.read().clone();

        // Left panel — configuration
        egui::SidePanel::left("config_panel")
            .resizable(true)
            .default_width(340.0)
            .min_width(280.0)
            .show(ctx, |ui| {
                let running = state == TestState::Running || state == TestState::Paused;
                config_panel::show(
                    ui,
                    &mut self.config,
                    running,
                    &mut self.curl_import_open,
                    &mut self.curl_import_text,
                    &mut self.curl_import_error,
                );
                ui.add_space(18.0);
                control_bar::show(ui, &state, &mut self.controller, &self.config, &self.handle);
            });

        // Right panel — dashboard with inner margin
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Frame::NONE
                .inner_margin(egui::Margin::same(24))
                .show(ui, |ui| {
                    dashboard::show(ui, &snapshot, &state);
                });
        });

        if state == TestState::Running || state == TestState::Paused {
            ctx.request_repaint();
        }
    }
}
```

- [ ] **Step 2: Verify compilation**

```bash
cargo check 2>&1
```
Expected: compiles without errors

- [ ] **Step 3: Commit**

```bash
git add src/app.rs
git commit -m "feat(ui): adjust panel widths, add inner margin to dashboard

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

### Task 3: control_bar.rs — 按钮样式统一

**Files:**
- Modify: `src/ui/control_bar.rs`

- [ ] **Step 1: Rewrite control_bar.rs with unified button styles**

Replace the entire file:

```rust
use crate::config::AppConfig;
use crate::control::{TestController, TestState};
use crate::theme::{self, ACCENT, NEGATIVE, WARNING};

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
                let btn = egui::Button::new(
                    egui::RichText::new("启动").size(13.0).color(egui::Color32::WHITE).strong()
                )
                    .fill(ACCENT)
                    .min_size(egui::vec2(100.0, 32.0));

                if ui.add(btn).clicked() {
                    controller.start(config.clone(), handle);
                }
            }
            TestState::Running => {
                let pause_btn = egui::Button::new(
                    egui::RichText::new("暂停").size(13.0).color(egui::Color32::WHITE).strong()
                )
                    .fill(WARNING)
                    .min_size(egui::vec2(100.0, 32.0));

                if ui.add(pause_btn).clicked() {
                    controller.pause();
                }

                ui.add_space(8.0);

                let stop_btn = egui::Button::new(
                    egui::RichText::new("停止").size(13.0).color(egui::Color32::WHITE).strong()
                )
                    .fill(NEGATIVE)
                    .min_size(egui::vec2(100.0, 32.0));

                if ui.add(stop_btn).clicked() {
                    controller.stop();
                }
            }
            TestState::Paused => {
                let resume_btn = egui::Button::new(
                    egui::RichText::new("恢复").size(13.0).color(egui::Color32::WHITE).strong()
                )
                    .fill(ACCENT)
                    .min_size(egui::vec2(100.0, 32.0));

                if ui.add(resume_btn).clicked() {
                    controller.resume();
                }

                ui.add_space(8.0);

                let stop_btn = egui::Button::new(
                    egui::RichText::new("停止").size(13.0).color(egui::Color32::WHITE).strong()
                )
                    .fill(NEGATIVE)
                    .min_size(egui::vec2(100.0, 32.0));

                if ui.add(stop_btn).clicked() {
                    controller.stop();
                }
            }
        }
    });
}
```

- [ ] **Step 2: Verify compilation**

```bash
cargo check 2>&1
```
Expected: compiles without errors

- [ ] **Step 3: Commit**

```bash
git add src/ui/control_bar.rs
git commit -m "feat(ui): unify button styles with theme colors, remove emoji icons

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

### Task 4: config_panel.rs — 输入控件与 section 间距

**Files:**
- Modify: `src/ui/config_panel.rs`

- [ ] **Step 1: Rewrite config_panel.rs**

Replace the entire file:

```rust
use crate::config::{AppConfig, HttpMethod};
use crate::theme::{self, ACCENT, NEGATIVE};
use curl_parser::ParsedRequest;
use std::str::FromStr;

pub fn show(
    ui: &mut egui::Ui,
    config: &mut AppConfig,
    running: bool,
    curl_import_open: &mut bool,
    curl_import_text: &mut String,
    curl_import_error: &mut Option<String>,
) {
    ui.add_space(8.0);

    ui.add_enabled_ui(!running, |ui| {
        // Import curl button — secondary (text-only) style
        if ui
            .add(
                egui::Button::new(
                    egui::RichText::new("从 cURL 导入").size(13.0).color(ACCENT)
                )
                    .fill(egui::Color32::TRANSPARENT)
                    .min_size(egui::vec2(0.0, 24.0)),
            )
            .clicked()
        {
            *curl_import_open = true;
            *curl_import_error = None;
            curl_import_text.clear();
        }

        ui.add_space(12.0);

        // ── Target section ──
        ui.add(theme::heading("请求目标"));
        ui.add_space(4.0);

        // URL
        ui.label(theme::body("URL"));
        ui.add(
            egui::TextEdit::singleline(&mut config.http.url)
                .hint_text("https://api.example.com/endpoint")
                .desired_width(f32::INFINITY),
        );

        ui.add_space(4.0);

        // Method
        ui.label(theme::body("Method"));
        egui::ComboBox::from_id_salt("http_method")
            .selected_text(config.http.method.as_str())
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut config.http.method, HttpMethod::GET, "GET");
                ui.selectable_value(&mut config.http.method, HttpMethod::POST, "POST");
                ui.selectable_value(&mut config.http.method, HttpMethod::PUT, "PUT");
                ui.selectable_value(&mut config.http.method, HttpMethod::DELETE, "DELETE");
            });

        ui.add_space(12.0);

        // ── Headers section ──
        ui.add(theme::heading("Headers"));
        ui.add_space(4.0);

        let mut remove_idx = None;
        for (i, (key, value)) in config.http.headers.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(key)
                        .hint_text("Key")
                        .desired_width(120.0),
                );
                ui.add(
                    egui::TextEdit::singleline(value)
                        .hint_text("Value")
                        .desired_width(150.0),
                );
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new("移除").size(11.0).color(NEGATIVE)
                        )
                            .fill(egui::Color32::TRANSPARENT)
                            .min_size(egui::vec2(0.0, 20.0)),
                    )
                    .clicked()
                {
                    remove_idx = Some(i);
                }
            });
        }
        if let Some(i) = remove_idx {
            config.http.headers.remove(i);
        }
        if ui
            .add(
                egui::Button::new(
                    egui::RichText::new("+ 添加 Header").size(13.0).color(ACCENT)
                )
                    .fill(egui::Color32::TRANSPARENT)
                    .min_size(egui::vec2(0.0, 24.0)),
            )
            .clicked()
        {
            config.http.headers.push((String::new(), String::new()));
        }

        // ── Body section (conditional) ──
        if matches!(config.http.method, HttpMethod::POST | HttpMethod::PUT) {
            ui.add_space(12.0);
            ui.add(theme::heading("Request Body"));
            ui.add_space(4.0);
            ui.add(
                egui::TextEdit::multiline(&mut config.http.body)
                    .hint_text("{\"key\": \"value\"}")
                    .desired_width(f32::INFINITY)
                    .desired_rows(4),
            );
        }

        ui.add_space(18.0);

        // ── Ramp-up section ──
        ui.add(theme::heading("阶梯加压"));
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.label(theme::body("起始并发"));
            ui.add(
                egui::DragValue::new(&mut config.ramp_up.start_concurrency)
                    .range(1..=10000)
                    .speed(1),
            );
        });
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label(theme::body("最终并发"));
            ui.add(
                egui::DragValue::new(&mut config.ramp_up.end_concurrency)
                    .range(1..=10000)
                    .speed(1),
            );
        });
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label(theme::body("阶梯数"));
            ui.add(
                egui::DragValue::new(&mut config.ramp_up.steps)
                    .range(0..=100)
                    .speed(1),
            );
        });
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label(theme::body("每阶时长(秒)"));
            ui.add(
                egui::DragValue::new(&mut config.ramp_up.step_duration_secs)
                    .range(1..=3600)
                    .speed(1),
            );
        });

        ui.add_space(4.0);
        ui.label(theme::body_small(&format!(
            "共 {} 阶段，总计 {} 秒",
            config.ramp_up.total_stages(),
            config.ramp_up.total_duration_secs()
        )));

        // Ramp-up bar preview
        let total_stages = config.ramp_up.total_stages();
        if total_stages > 0 {
            let available_width = ui.available_width();
            let bar_height = 40.0;
            let spacing = 2.0;
            let bar_w = (available_width / total_stages as f32) - spacing;
            let max_conc = config.ramp_up.end_concurrency.max(1) as f32;

            ui.add_space(4.0);
            let (response, painter) = ui.allocate_painter(
                egui::vec2(available_width, bar_height + 16.0),
                egui::Sense::hover(),
            );
            let rect = response.rect;

            for i in 0..total_stages {
                let concurrency = config.ramp_up.concurrency_at_stage(i);
                let fraction = concurrency as f32 / max_conc;
                let x = rect.left() + i as f32 * (bar_w + spacing);
                let h = fraction * bar_height;
                let y = rect.bottom() - 16.0 - h;

                let color = egui::Color32::from_rgb(
                    200 - (120.0 * fraction) as u8,
                    210 - (90.0 * fraction) as u8,
                    240 - (60.0 * fraction) as u8,
                );

                painter.rect_filled(
                    egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(bar_w, h)),
                    egui::CornerRadius::same(2),
                    color,
                );

                if total_stages <= 10 {
                    painter.text(
                        egui::pos2(x + bar_w / 2.0, rect.bottom() - 4.0),
                        egui::Align2::CENTER_BOTTOM,
                        format!("{}", concurrency),
                        egui::FontId::new(10.0, egui::FontFamily::Proportional),
                        theme::TEXT_TERTIARY,
                    );
                }
            }
        }
    });

    // ── Import curl popup window ──
    if *curl_import_open {
        use std::cell::Cell;
        let mut open = true;
        let import_clicked = Cell::new(false);
        let cancel_clicked = Cell::new(false);

        egui::Window::new("从 cURL 导入")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .min_width(500.0)
            .show(ui.ctx(), |ui| {
                ui.label(theme::body("粘贴 cURL 命令:"));
                ui.add_space(4.0);

                ui.add(
                    egui::TextEdit::multiline(curl_import_text)
                        .hint_text("curl -X POST https://api.example.com \\\n  -H \"Content-Type: application/json\" \\\n  -d '{\"key\": \"value\"}'")
                        .desired_width(f32::INFINITY)
                        .desired_rows(6)
                        .font(egui::FontId::monospace(12.0)),
                );
                ui.add_space(8.0);

                if let Some(err) = curl_import_error.as_ref() {
                    ui.colored_label(NEGATIVE, format!("错误: {}", err));
                    ui.add_space(4.0);
                }

                ui.horizontal(|ui| {
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new("取消").size(13.0).color(theme::TEXT_PRIMARY)
                            )
                                .fill(egui::Color32::TRANSPARENT)
                                .min_size(egui::vec2(56.0, 28.0)),
                        )
                        .clicked()
                    {
                        cancel_clicked.set(true);
                    }

                    ui.add_space(8.0);

                    let import_btn = egui::Button::new(
                        egui::RichText::new("导入").size(13.0).color(egui::Color32::WHITE).strong()
                    )
                        .fill(ACCENT)
                        .min_size(egui::vec2(72.0, 32.0));

                    if ui.add(import_btn).clicked() {
                        import_clicked.set(true);
                    }
                });
            });

        if cancel_clicked.get() {
            *curl_import_open = false;
            *curl_import_error = None;
        } else if import_clicked.get() {
            let trimmed = curl_import_text.trim();
            if trimmed.is_empty() {
                *curl_import_error = Some("cURL 命令为空".to_string());
            } else {
                let normalized = normalize_curl(trimmed);
                match ParsedRequest::from_str(&normalized) {
                    Ok(parsed) => {
                        if let Err(msg) = populate_config(config, &parsed) {
                            *curl_import_error = Some(msg);
                        } else {
                            *curl_import_open = false;
                            *curl_import_error = None;
                        }
                    }
                    Err(e) => {
                        *curl_import_error = Some(format!("解析失败: {}", e));
                    }
                }
            }
        }

        if !open {
            *curl_import_open = false;
        }
    }
}

/// Normalize a curl command for the parser: strip line continuations,
/// convert long-form flags (--request, --url, --header, --data) to short form.
fn normalize_curl(cmd: &str) -> String {
    let joined = cmd
        .lines()
        .map(|l| l.strip_suffix('\\').unwrap_or(l).trim_end())
        .collect::<Vec<_>>()
        .join(" ");

    let tokens: Vec<&str> = joined.split_whitespace().collect();
    let mut out = Vec::with_capacity(tokens.len());
    let mut i = 0;
    while i < tokens.len() {
        let t = tokens[i];
        match t {
            "--request" | "-X" => {
                out.push("-X".to_string());
                if let Some(&val) = tokens.get(i + 1) {
                    out.push(val.to_string());
                    i += 2;
                    continue;
                }
            }
            "--url" => {
                if let Some(&val) = tokens.get(i + 1) {
                    out.push(val.to_string());
                    i += 2;
                    continue;
                }
            }
            "--header" => {
                out.push("-H".to_string());
                if let Some(&val) = tokens.get(i + 1) {
                    out.push(val.to_string());
                    i += 2;
                    continue;
                }
            }
            "--data" | "--data-raw" | "--data-binary" => {
                out.push("-d".to_string());
                if let Some(&val) = tokens.get(i + 1) {
                    out.push(val.to_string());
                    i += 2;
                    continue;
                }
            }
            _ => {
                out.push(t.to_string());
            }
        }
        i += 1;
    }
    out.join(" ")
}

fn populate_config(config: &mut AppConfig, parsed: &ParsedRequest) -> Result<(), String> {
    let method = HttpMethod::from_str(parsed.method.as_str())
        .ok_or_else(|| format!("不支持的方法: {}。支持: GET, POST, PUT, DELETE", parsed.method))?;

    config.http.url = parsed.url.to_string();
    config.http.method = method;
    config.http.headers.clear();
    for (name, value) in parsed.headers.iter() {
        let key = name.as_str().to_string();
        let val = value.to_str().unwrap_or("").to_string();
        config.http.headers.push((key, val));
    }
    config.http.body = parsed.body().unwrap_or_default();
    Ok(())
}
```

- [ ] **Step 2: Verify compilation**

```bash
cargo check 2>&1
```
Expected: compiles without errors

- [ ] **Step 3: Commit**

```bash
git add src/ui/config_panel.rs
git commit -m "feat(ui): restyle config panel with themed inputs and section spacing

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

### Task 5: stat_cards.rs — 指标卡片重设计

**Files:**
- Modify: `src/ui/stat_cards.rs`

- [ ] **Step 1: Rewrite stat_cards.rs with white-background + bottom-border cards**

Replace the entire file:

```rust
use crate::metrics::MetricsSnapshot;
use crate::theme::{self, ACCENT, NEGATIVE, POSITIVE};

pub fn show(ui: &mut egui::Ui, snapshot: &MetricsSnapshot) {
    let card_w = (ui.available_width() - 24.0) / 4.0;

    ui.horizontal(|ui| {
        stat_card(
            ui,
            card_w,
            "QPS",
            &format!("{:.0}", snapshot.qps),
            ACCENT,
        );
        ui.add_space(8.0);
        stat_card(
            ui,
            card_w,
            "错误率",
            &format!("{:.1}%", snapshot.error_rate * 100.0),
            if snapshot.error_rate > 0.05 { NEGATIVE } else { POSITIVE },
        );
        ui.add_space(8.0);
        stat_card(
            ui,
            card_w,
            "活跃连接",
            &format!("{}", snapshot.active_connections),
            ACCENT,
        );
        ui.add_space(8.0);
        stat_card(
            ui,
            card_w,
            "总请求",
            &format!("{}", snapshot.total_requests),
            ACCENT,
        );
    });
}

fn stat_card(ui: &mut egui::Ui, width: f32, label: &str, value: &str, accent: egui::Color32) {
    ui.allocate_ui(egui::vec2(width, 56.0), |ui| {
        // Bottom border as a visual divider
        let rect = ui.max_rect();
        ui.painter().hline(
            rect.left()..=rect.right(),
            rect.bottom() - 1.0,
            egui::Stroke::new(1.0, theme::BORDER),
        );

        ui.vertical_centered(|ui| {
            ui.add_space(2.0);
            ui.label(theme::metric_value(value, accent));
            ui.add_space(2.0);
            ui.label(theme::metric_label(label));
        });
    });
}
```

- [ ] **Step 2: Verify compilation**

```bash
cargo check 2>&1
```
Expected: compiles without errors

- [ ] **Step 3: Commit**

```bash
git add src/ui/stat_cards.rs
git commit -m "feat(ui): redesign stat cards — white background, bottom border, unified typography

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

### Task 6: dashboard.rs — 右侧面板间距和状态排版

**Files:**
- Modify: `src/ui/dashboard.rs`

- [ ] **Step 1: Rewrite dashboard.rs**

Replace the entire file:

```rust
use crate::control::TestState;
use crate::metrics::MetricsSnapshot;
use crate::theme;
use crate::ui::{latency_chart, stat_cards, status_chart};

pub fn show(ui: &mut egui::Ui, snapshot: &MetricsSnapshot, state: &TestState) {
    ui.add(theme::heading("实时指标"));

    if *state == TestState::Idle {
        ui.add_space(60.0);
        ui.vertical_centered(|ui| {
            ui.label(
                egui::RichText::new("配置压测参数后点击启动")
                    .size(15.0)
                    .color(theme::TEXT_TERTIARY),
            );
        });
        return;
    }

    ui.add_space(18.0);

    stat_cards::show(ui, snapshot);

    ui.add_space(24.0);

    latency_chart::show(ui, snapshot);

    ui.add_space(24.0);

    status_chart::show(ui, snapshot);

    if *state == TestState::Stopped {
        ui.add_space(24.0);
        ui.add(theme::heading("压测完成"));
        ui.add_space(8.0);
        ui.label(theme::body(&format!("总请求数: {}", snapshot.total_requests)));
        ui.label(theme::body(&format!("QPS: {:.1}", snapshot.qps)));
        ui.label(theme::body(&format!(
            "错误率: {:.1}%",
            snapshot.error_rate * 100.0
        )));
        ui.label(theme::body(&format!(
            "延迟 P50/P90/P99: {:.1}ms / {:.1}ms / {:.1}ms",
            snapshot.latency_p50.as_secs_f64() * 1000.0,
            snapshot.latency_p90.as_secs_f64() * 1000.0,
            snapshot.latency_p99.as_secs_f64() * 1000.0,
        )));
        ui.label(theme::body(&format!(
            "总耗时: {:.1}s",
            snapshot.elapsed.as_secs_f64()
        )));
    }
}
```

- [ ] **Step 2: Verify compilation**

```bash
cargo check 2>&1
```
Expected: compiles without errors

- [ ] **Step 3: Commit**

```bash
git add src/ui/dashboard.rs
git commit -m "feat(ui): refine dashboard spacing, remove separators, styled idle/stopped states

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

### Task 7: latency_chart.rs — 单色 accent 柱状图

**Files:**
- Modify: `src/ui/latency_chart.rs`

- [ ] **Step 1: Rewrite latency_chart.rs with unified accent color and repositioned layout**

Replace the entire file:

```rust
use crate::metrics::MetricsSnapshot;
use crate::theme::{self, ACCENT, TEXT_TERTIARY};

pub fn show(ui: &mut egui::Ui, snapshot: &MetricsSnapshot) {
    let p50_ms = snapshot.latency_p50.as_secs_f64() * 1000.0;
    let p90_ms = snapshot.latency_p90.as_secs_f64() * 1000.0;
    let p99_ms = snapshot.latency_p99.as_secs_f64() * 1000.0;

    let bars = [("P50", p50_ms), ("P90", p90_ms), ("P99", p99_ms)];

    let available_width = ui.available_width();
    let left_margin = 40.0;    // space for label text on left
    let right_margin = 60.0;   // space for value text on right
    let bar_area_w = available_width - left_margin - right_margin;
    let bar_height = 32.0;
    let gap = 12.0;
    let chart_h = bars.len() as f32 * (bar_height + gap) + 28.0;

    let (response, painter) = ui.allocate_painter(
        egui::vec2(available_width, chart_h + 20.0),
        egui::Sense::hover(),
    );
    let rect = response.rect;
    let max_val = p99_ms.max(1.0) * 1.15;

    for (i, (label, val)) in bars.iter().enumerate() {
        let y = rect.top() + i as f32 * (bar_height + gap);
        let fraction = (*val / max_val) as f32;
        let bar_w = fraction * bar_area_w;

        // Label on the left
        painter.text(
            egui::pos2(rect.left() + left_margin - 8.0, y + bar_height / 2.0),
            egui::Align2::RIGHT_CENTER,
            *label,
            egui::FontId::new(12.0, egui::FontFamily::Proportional),
            theme::TEXT_PRIMARY,
        );

        // Bar
        if bar_w > 0.0 {
            painter.rect_filled(
                egui::Rect::from_min_size(
                    egui::pos2(rect.left() + left_margin, y + 2.0),
                    egui::vec2(bar_w, bar_height - 4.0),
                ),
                egui::CornerRadius::same(4),
                ACCENT,
            );
        }

        // Value on the right side of bar
        painter.text(
            egui::pos2(rect.left() + left_margin + bar_w + 6.0, y + bar_height / 2.0),
            egui::Align2::LEFT_CENTER,
            format!("{:.1} ms", val),
            egui::FontId::new(11.0, egui::FontFamily::Proportional),
            theme::TEXT_SECONDARY,
        );
    }

    // Step info — muted
    ui.add_space(4.0);
    ui.label(theme::body_small(&format!(
        "已运行 {:.0}s  |  阶梯 {}/{}",
        snapshot.elapsed.as_secs_f64(),
        snapshot.current_step + 1,
        snapshot.step_progress
    )));

    // Step progress bar
    ui.add(
        egui::ProgressBar::new(snapshot.step_progress as f32)
            .text(format!("阶梯进度 {:.0}%", snapshot.step_progress * 100.0)),
    );
}
```

- [ ] **Step 2: Verify compilation**

```bash
cargo check 2>&1
```
Expected: compiles without errors

- [ ] **Step 3: Commit**

```bash
git add src/ui/latency_chart.rs
git commit -m "feat(ui): redesign latency chart — unified accent bars, left labels, right values

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

### Task 8: status_chart.rs — 灰阶状态码分布图

**Files:**
- Modify: `src/ui/status_chart.rs`

- [ ] **Step 1: Rewrite status_chart.rs with grayscale bars + accent for 5xx**

Replace the entire file:

```rust
use crate::metrics::MetricsSnapshot;
use crate::theme::{self, ACCENT, TEXT_PRIMARY, TEXT_SECONDARY, TEXT_TERTIARY};

pub fn show(ui: &mut egui::Ui, snapshot: &MetricsSnapshot) {
    if snapshot.status_codes.is_empty() {
        ui.label(theme::body_small("等待数据..."));
        return;
    }

    let mut codes: Vec<(u16, u64)> = snapshot
        .status_codes
        .iter()
        .map(|(k, v)| (*k, *v))
        .collect();
    codes.sort_by_key(|(k, _)| *k);

    let total: u64 = codes.iter().map(|(_, v)| v).sum();
    let left_margin = 48.0;
    let right_margin = 100.0;
    let bar_height = 20.0;
    let gap = 6.0;

    let (response, painter) = ui.allocate_painter(
        egui::vec2(ui.available_width(), codes.len() as f32 * (bar_height + gap) + 10.0),
        egui::Sense::hover(),
    );
    let rect = response.rect;
    let bar_area_w = (rect.width() - left_margin - right_margin).max(40.0);

    for (i, (code, count)) in codes.iter().enumerate() {
        let fraction = if total > 0 {
            *count as f32 / total as f32
        } else {
            0.0
        };
        let bar_w = fraction * bar_area_w;
        let y = rect.top() + i as f32 * (bar_height + gap);

        let bar_color = match code / 100 {
            2 => TEXT_PRIMARY,    // dark gray for success
            3 => TEXT_SECONDARY,   // medium gray
            4 => TEXT_TERTIARY,    // light gray
            5 => ACCENT,           // accent blue for server errors
            _ => TEXT_TERTIARY,
        };

        // Status code label
        painter.text(
            egui::pos2(rect.left() + left_margin - 8.0, y + bar_height / 2.0),
            egui::Align2::RIGHT_CENTER,
            format!("{}", code),
            egui::FontId::new(12.0, egui::FontFamily::Proportional),
            TEXT_PRIMARY,
        );

        // Bar
        if bar_w > 0.0 {
            painter.rect_filled(
                egui::Rect::from_min_size(
                    egui::pos2(rect.left() + left_margin, y + 2.0),
                    egui::vec2(bar_w, bar_height - 4.0),
                ),
                egui::CornerRadius::same(3),
                bar_color,
            );
        }

        // Count and percentage
        painter.text(
            egui::pos2(rect.left() + left_margin + bar_w + 8.0, y + bar_height / 2.0),
            egui::Align2::LEFT_CENTER,
            format!("{} ({:.1}%)", count, fraction * 100.0),
            egui::FontId::new(11.0, egui::FontFamily::Proportional),
            TEXT_SECONDARY,
        );
    }
}
```

- [ ] **Step 2: Verify compilation**

```bash
cargo check 2>&1
```
Expected: compiles without errors

- [ ] **Step 3: Run clippy**

```bash
cargo clippy -- -D warnings 2>&1
```
Expected: no warnings

- [ ] **Step 4: Run tests**

```bash
cargo test 2>&1
```
Expected: all tests pass (engine::tests::test_execute_request_404 may fail due to network — acceptable)

- [ ] **Step 5: Commit**

```bash
git add src/ui/status_chart.rs
git commit -m "feat(ui): redesign status chart — grayscale bars with accent for 5xx

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## 验证清单

完成所有 task 后运行：

```bash
cargo check              # 编译通过
cargo clippy -- -D warnings  # 无警告
cargo test               # 测试通过
cargo run --release      # 手动验证 GUI
```

手动验证步骤：
1. Idle 状态 — 左侧配置面板（灰底），右侧居中引导文字（白底）
2. 填写 URL、Headers、调整阶梯参数 — 检查输入框样式和 section 间距
3. 启动压测 — 检查按钮样式、4 个指标卡片、延迟图、状态码图
4. 暂停 → 恢复 → 停止 — 确认各状态 UI 切换正确
5. 测试 cURL 导入弹窗
