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
        ui.vertical_centered(|ui| {
            ui.add_space(2.0);
            ui.label(theme::metric_value(value, accent));
            ui.add_space(2.0);
            ui.label(theme::metric_label(label));
        });
    });
}
