use crate::metrics::MetricsSnapshot;

pub fn show(ui: &mut egui::Ui, snapshot: &MetricsSnapshot) {
    ui.horizontal(|ui| {
        stat_card(ui, "QPS", &format!("{:.0}", snapshot.qps), egui::Color32::from_rgb(33, 150, 243));
        stat_card(
            ui,
            "错误率",
            &format!("{:.1}%", snapshot.error_rate * 100.0),
            if snapshot.error_rate > 0.05 {
                egui::Color32::from_rgb(244, 67, 54)
            } else {
                egui::Color32::from_rgb(76, 175, 80)
            },
        );
        stat_card(
            ui,
            "活跃连接",
            &format!("{}", snapshot.active_connections),
            egui::Color32::from_rgb(156, 39, 176),
        );
        stat_card(
            ui,
            "总请求",
            &format!("{}", snapshot.total_requests),
            egui::Color32::from_rgb(255, 152, 0),
        );
    });
}

fn stat_card(ui: &mut egui::Ui, label: &str, value: &str, color: egui::Color32) {
    let frame = egui::Frame::none()
        .fill(egui::Color32::from_rgb(250, 250, 250))
        .rounding(egui::Rounding::same(6))
        .inner_margin(egui::Margin::symmetric(12, 8));

    frame.show(ui, |ui| {
        ui.set_min_width(100.0);
        ui.label(
            egui::RichText::new(value)
                .color(color)
                .size(22.0)
                .strong(),
        );
        ui.label(egui::RichText::new(label).size(12.0).color(egui::Color32::GRAY));
    });
}
