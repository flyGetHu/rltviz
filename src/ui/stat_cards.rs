use crate::metrics::MetricsSnapshot;
use crate::theme::{ACCENT, BG_TERTIARY, BORDER, NEGATIVE, POSITIVE, TEXT_TERTIARY};
use egui::StrokeKind;

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
    let card_h = 44.0;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, card_h + 18.0), egui::Sense::hover());

    let card_rect = egui::Rect::from_min_size(rect.left_top(), egui::vec2(width, card_h));
    let corner = egui::CornerRadius::same(8);
    ui.painter().rect_filled(card_rect, corner, BG_TERTIARY);
    ui.painter().rect_stroke(card_rect, corner, egui::Stroke::new(1.0, BORDER), StrokeKind::Inside);

    let cx = card_rect.center().x;
    ui.painter().text(
        egui::pos2(cx, card_rect.center().y),
        egui::Align2::CENTER_CENTER,
        value,
        egui::FontId::proportional(28.0),
        accent,
    );
    ui.painter().text(
        egui::pos2(cx, rect.bottom()),
        egui::Align2::CENTER_BOTTOM,
        label,
        egui::FontId::proportional(11.0),
        TEXT_TERTIARY,
    );
}
