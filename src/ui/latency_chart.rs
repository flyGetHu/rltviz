use crate::metrics::MetricsSnapshot;

pub fn show(ui: &mut egui::Ui, snapshot: &MetricsSnapshot) {
    ui.heading("延迟分布 (ms)");

    let p50_ms = snapshot.latency_p50.as_secs_f64() * 1000.0;
    let p90_ms = snapshot.latency_p90.as_secs_f64() * 1000.0;
    let p99_ms = snapshot.latency_p99.as_secs_f64() * 1000.0;

    let bars = [
        ("P50", p50_ms, egui::Color32::from_rgb(76, 175, 80)),
        ("P90", p90_ms, egui::Color32::from_rgb(255, 152, 0)),
        ("P99", p99_ms, egui::Color32::from_rgb(244, 67, 54)),
    ];

    let available_width = ui.available_width();
    let chart_height = 200.0;
    let spacing = 16.0;
    let bar_width = (available_width - spacing * 4.0) / 3.0;

    let (response, painter) = ui.allocate_painter(
        egui::vec2(available_width, chart_height + 20.0),
        egui::Sense::hover(),
    );
    let rect = response.rect;
    let max_bar_h = chart_height - 20.0;
    let max_val = p99_ms.max(1.0) * 1.2;

    for (i, (label, val, color)) in bars.iter().enumerate() {
        let x = rect.left() + spacing + i as f32 * (bar_width + spacing);
        let fraction = (*val / max_val) as f32;
        let h = fraction * max_bar_h;
        let y = rect.bottom() - 20.0 - h;

        painter.rect_filled(
            egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(bar_width, h)),
            egui::CornerRadius::same(4),
            *color,
        );

        painter.text(
            egui::pos2(x + bar_width / 2.0, y - 4.0),
            egui::Align2::CENTER_BOTTOM,
            format!("{:.1}", val),
            egui::FontId::new(13.0, egui::FontFamily::Proportional),
            *color,
        );

        painter.text(
            egui::pos2(x + bar_width / 2.0, rect.bottom() - 6.0),
            egui::Align2::CENTER_BOTTOM,
            *label,
            egui::FontId::new(12.0, egui::FontFamily::Proportional),
            egui::Color32::GRAY,
        );
    }

    ui.add_space(4.0);
    ui.label(format!("已运行: {:.0}s | 当前阶梯: {}/{}", snapshot.elapsed.as_secs_f64(), snapshot.current_step + 1, snapshot.step_progress));

    // Step progress bar
    ui.add(egui::ProgressBar::new(snapshot.step_progress as f32).text(format!("阶梯进度 {:.0}%", snapshot.step_progress * 100.0)));
}
