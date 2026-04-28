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
                egui::Rounding::same(2),
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
