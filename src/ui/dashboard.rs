use crate::control::TestState;
use crate::history::HistoryRecord;
use crate::metrics::MetricsSnapshot;
use crate::theme;
use crate::ui::{latency_chart, stat_cards, status_chart};

pub fn show(ui: &mut egui::Ui, snapshot: &MetricsSnapshot, state: &TestState) {
    ui.label(theme::heading("实时指标"));

    if *state == TestState::Idle {
        ui.add_space(60.0);
        ui.vertical_centered(|ui| {
            ui.label(
                egui::RichText::new("配置压测参数后点击启动")
                    .size(15.0)
                    .color(theme::TEXT_TERTIARY),
            );
        });
        return;
    }

    ui.add_space(18.0);

    stat_cards::show(ui, snapshot);

    ui.add_space(24.0);

    latency_chart::show(ui, snapshot, true);

    ui.add_space(24.0);

    status_chart::show(ui, snapshot);

    if *state == TestState::Stopped {
        ui.add_space(24.0);
        ui.label(theme::heading("压测完成"));
        ui.add_space(8.0);
        ui.label(theme::body(&format!("总请求数: {}", snapshot.total_requests)));
        ui.label(theme::body(&format!("QPS: {:.1}", snapshot.qps)));
        ui.label(theme::body(&format!("错误率: {:.1}%", snapshot.error_rate * 100.0)));
        ui.label(theme::body(&format!(
            "延迟 P50/P90/P99: {:.1}ms / {:.1}ms / {:.1}ms",
            snapshot.latency_p50.as_secs_f64() * 1000.0,
            snapshot.latency_p90.as_secs_f64() * 1000.0,
            snapshot.latency_p99.as_secs_f64() * 1000.0,
        )));
        ui.label(theme::body(&format!("总耗时: {:.1}s", snapshot.elapsed.as_secs_f64())));
    }
}

pub fn show_history(ui: &mut egui::Ui, record: &HistoryRecord) -> bool {
    ui.label(theme::heading(&format!("历史记录 — {}", record.timestamp)));
    ui.add_space(4.0);
    ui.label(theme::body_small(&format!(
        "运行 {:.1}s  |  总请求 {}",
        record.summary.duration_secs,
        record.summary.total_requests,
    )));
    ui.add_space(18.0);

    let snapshot = record
        .summary
        .to_snapshot(record.config.ramp_up.end_concurrency);
    stat_cards::show(ui, &snapshot);
    ui.add_space(24.0);
    latency_chart::show(ui, &snapshot, false);
    ui.add_space(24.0);
    status_chart::show(ui, &snapshot);

    ui.add_space(24.0);
    let mut clicked = false;
    ui.vertical_centered(|ui| {
        let btn = egui::Button::new(
            egui::RichText::new("复用配置")
                .size(13.0)
                .color(egui::Color32::WHITE)
                .strong(),
        )
        .fill(theme::ACCENT)
        .min_size(egui::vec2(160.0, 32.0));
        if ui.add(btn).clicked() {
            clicked = true;
        }
    });
    clicked
}
