use crate::config::{AppConfig, HttpMethod};
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
    ui.heading("压测配置");
    ui.add_space(8.0);

    ui.add_enabled_ui(!running, |ui| {
        // Import curl button
        if ui.button("📋 从 cURL 导入").clicked() {
            *curl_import_open = true;
            *curl_import_error = None;
            curl_import_text.clear();
        }
        ui.add_space(8.0);

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
                    50 + (205.0 * fraction) as u8,
                    180 - (100.0 * fraction) as u8,
                    100 + (50.0 * fraction) as u8,
                );

                painter.rect_filled(
                    egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(bar_width, h)),
                    egui::CornerRadius::same(2),
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
    });

    // Import curl popup window
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
                ui.label("粘贴 cURL 命令:");
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
                    ui.colored_label(egui::Color32::RED, format!("错误: {}", err));
                    ui.add_space(4.0);
                }

                ui.horizontal(|ui| {
                    if ui.button("取消").clicked() {
                        cancel_clicked.set(true);
                    }
                    if ui.button("导入").clicked() {
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
    // Remove backslash line continuations and trailing spaces
    let joined = cmd
        .lines()
        .map(|l| l.strip_suffix('\\').unwrap_or(l).trim_end())
        .collect::<Vec<_>>()
        .join(" ");

    // Normalize token by token
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
                // --url VALUE → just emit VALUE as positional arg
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
