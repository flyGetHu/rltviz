use crate::app::PanelTab;
use crate::config::{AppConfig, HttpMethod};
use crate::history::HistoryRecord;
use crate::theme::{self, ACCENT, NEGATIVE, POSITIVE, WARNING};
use curl_parser::ParsedRequest;
use std::str::FromStr;

pub fn show(
    ui: &mut egui::Ui,
    config: &mut AppConfig,
    running: bool,
    curl_import_open: &mut bool,
    curl_import_text: &mut String,
    curl_import_error: &mut Option<String>,
    active_tab: &mut PanelTab,
    history_records: &[HistoryRecord],
    selected_history: &mut Option<usize>,
) {
    ui.add_space(8.0);

    // ── Tab bar ──
    ui.horizontal(|ui| {
        ui.selectable_value(active_tab, PanelTab::Config, "Config");
        ui.selectable_value(active_tab, PanelTab::History, "History");
    });
    ui.add_space(8.0);

    match active_tab {
        PanelTab::History => {
            show_history_tab(ui, history_records, selected_history);
            return;
        }
        PanelTab::Config => {}
    }

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
        ui.label(theme::heading("请求目标"));
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
        ui.label(theme::heading("Headers"));
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
            ui.label(theme::heading("Request Body"));
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
        ui.label(theme::heading("阶梯加压"));
        ui.add_space(4.0);

        egui::Grid::new("ramp_up_params")
            .num_columns(2)
            .spacing([4.0, 4.0])
            .show(ui, |ui| {
                ui.label(theme::body("起始并发"));
                ui.add(
                    egui::DragValue::new(&mut config.ramp_up.start_concurrency)
                        .range(1..=10000)
                        .speed(1),
                );
                ui.end_row();

                ui.label(theme::body("最终并发"));
                ui.add(
                    egui::DragValue::new(&mut config.ramp_up.end_concurrency)
                        .range(1..=10000)
                        .speed(1),
                );
                ui.end_row();

                ui.label(theme::body("阶梯数"));
                ui.add(
                    egui::DragValue::new(&mut config.ramp_up.steps)
                        .range(0..=100)
                        .speed(1),
                );
                ui.end_row();

                ui.label(theme::body("每阶时长(秒)"));
                ui.add(
                    egui::DragValue::new(&mut config.ramp_up.step_duration_secs)
                        .range(1..=3600)
                        .speed(1),
                );
                ui.end_row();
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

fn show_history_tab(
    ui: &mut egui::Ui,
    records: &[HistoryRecord],
    selected: &mut Option<usize>,
) {
    if records.is_empty() {
        ui.add_space(60.0);
        ui.vertical_centered(|ui| {
            ui.label(
                egui::RichText::new("暂无历史记录")
                    .size(15.0)
                    .color(theme::TEXT_TERTIARY),
            );
        });
        return;
    }

    egui::ScrollArea::vertical()
        .max_height(f32::INFINITY)
        .show(ui, |ui| {
            for (i, record) in records.iter().enumerate() {
                let is_selected = *selected == Some(i);
                let frame = if is_selected {
                    egui::Frame::NONE
                        .inner_margin(egui::Margin::same(8))
                        .stroke(egui::Stroke::new(2.0, ACCENT))
                        .outer_margin(egui::Margin::symmetric(0, 2))
                } else {
                    egui::Frame::NONE
                        .inner_margin(egui::Margin::same(8))
                        .outer_margin(egui::Margin::symmetric(0, 2))
                };

                let response = frame.show(ui, |ui| {
                    ui.set_min_width(ui.available_width());

                    ui.horizontal(|ui| {
                        let method_color = match record.config.http.method {
                            HttpMethod::GET => ACCENT,
                            HttpMethod::POST => WARNING,
                            HttpMethod::PUT => POSITIVE,
                            HttpMethod::DELETE => NEGATIVE,
                        };
                        ui.label(
                            egui::RichText::new(record.config.http.method.as_str())
                                .size(12.0)
                                .color(method_color)
                                .strong(),
                        );
                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                ui.label(theme::body_small(&record.timestamp));
                            },
                        );
                    });

                    ui.label(
                        egui::RichText::new(truncate_url(&record.config.http.url, 40))
                            .size(11.0)
                            .color(theme::TEXT_SECONDARY),
                    );

                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("QPS: {:.0}", record.summary.qps))
                                .size(10.0)
                                .color(theme::TEXT_TERTIARY),
                        );
                        ui.label(
                            egui::RichText::new(format!(
                                "P99: {:.0}ms",
                                record.summary.p99_ms
                            ))
                            .size(10.0)
                            .color(theme::TEXT_TERTIARY),
                        );
                        ui.label(
                            egui::RichText::new(format!(
                                "Err: {:.1}%",
                                record.summary.error_rate * 100.0
                            ))
                            .size(10.0)
                            .color(theme::TEXT_TERTIARY),
                        );
                    });
                });

                if response.response.clicked() {
                    *selected = Some(i);
                }
            }
        });
}

fn truncate_url(url: &str, max_len: usize) -> String {
    if url.len() <= max_len {
        url.to_string()
    } else {
        format!("{}...", &url[..max_len.saturating_sub(3)])
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
