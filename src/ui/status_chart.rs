use crate::metrics::MetricsSnapshot;
use crate::theme::{self, NEGATIVE, POSITIVE, TEXT_PRIMARY, TEXT_SECONDARY, TEXT_TERTIARY, WARNING};

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
            2 => POSITIVE,
            3 => TEXT_SECONDARY,
            4 => WARNING,
            5 => NEGATIVE,
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
