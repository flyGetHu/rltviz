use crate::metrics::MetricsSnapshot;

pub fn show(ui: &mut egui::Ui, snapshot: &MetricsSnapshot) {
    ui.heading("状态码分布");

    if snapshot.status_codes.is_empty() {
        ui.label(egui::RichText::new("等待数据...").color(egui::Color32::GRAY));
        return;
    }

    let mut codes: Vec<(u16, u64)> = snapshot.status_codes.iter().map(|(k, v)| (*k, *v)).collect();
    codes.sort_by_key(|(k, _)| *k);

    let total: u64 = codes.iter().map(|(_, v)| v).sum();
    let available_width = ui.available_width();
    let bar_height = 24.0;
    let gap = 4.0;

    let (response, painter) = ui.allocate_painter(
        egui::vec2(available_width, codes.len() as f32 * (bar_height + gap) + 10.0),
        egui::Sense::hover(),
    );
    let rect = response.rect;

    for (i, (code, count)) in codes.iter().enumerate() {
        let fraction = if total > 0 { *count as f32 / total as f32 } else { 0.0 };
        let bar_w = fraction * (available_width - 80.0);
        let y = rect.top() + i as f32 * (bar_height + gap);

        let color = match code / 100 {
            2 => egui::Color32::from_rgb(76, 175, 80),
            3 => egui::Color32::from_rgb(33, 150, 243),
            4 => egui::Color32::from_rgb(255, 152, 0),
            5 => egui::Color32::from_rgb(244, 67, 54),
            _ => egui::Color32::GRAY,
        };

        painter.text(
            egui::pos2(rect.left(), y + bar_height / 2.0),
            egui::Align2::LEFT_CENTER,
            format!("{}", code),
            egui::FontId::new(12.0, egui::FontFamily::Proportional),
            egui::Color32::WHITE,
        );

        if bar_w > 0.0 {
            painter.rect_filled(
                egui::Rect::from_min_size(egui::pos2(rect.left() + 55.0, y + 2.0), egui::vec2(bar_w, bar_height - 4.0)),
                egui::Rounding::same(3),
                color,
            );
        }

        painter.text(
            egui::pos2(rect.left() + 55.0 + bar_w + 4.0, y + bar_height / 2.0),
            egui::Align2::LEFT_CENTER,
            format!("{} ({:.1}%)", count, fraction * 100.0),
            egui::FontId::new(11.0, egui::FontFamily::Proportional),
            egui::Color32::GRAY,
        );
    }
}
