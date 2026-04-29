use crate::metrics::MetricsSnapshot;
use crate::theme::{self, ACCENT};

pub fn show(ui: &mut egui::Ui, snapshot: &MetricsSnapshot, show_progress: bool) {
    let p50_ms = snapshot.latency_p50.as_secs_f64() * 1000.0;
    let p90_ms = snapshot.latency_p90.as_secs_f64() * 1000.0;
    let p99_ms = snapshot.latency_p99.as_secs_f64() * 1000.0;

    let bars = [("P50", p50_ms), ("P90", p90_ms), ("P99", p99_ms)];

    let available_width = ui.available_width();
    let left_margin = 40.0;
    let right_margin = 60.0;
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
    if show_progress {
        ui.add_space(4.0);
        ui.label(theme::body_small(&format!(
            "已运行 {:.0}s  |  阶梯 {}",
            snapshot.elapsed.as_secs_f64(),
            snapshot.current_step + 1
        )));

        // Step progress bar
        ui.add(
            egui::ProgressBar::new(snapshot.step_progress as f32)
                .text(format!("阶梯进度 {:.0}%", snapshot.step_progress * 100.0)),
        );
    }
}
