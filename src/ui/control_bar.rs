use crate::config::AppConfig;
use crate::control::{TestController, TestState};
use crate::theme::{ACCENT, NEGATIVE, WARNING};

pub fn show(
    ui: &mut egui::Ui,
    state: &TestState,
    controller: &mut TestController,
    config: &AppConfig,
    handle: &tokio::runtime::Handle,
) {
    ui.horizontal(|ui| {
        match state {
            TestState::Idle | TestState::Stopped => {
                let btn = egui::Button::new(
                    egui::RichText::new("启动").size(13.0).color(egui::Color32::WHITE).strong()
                )
                    .fill(ACCENT)
                    .min_size(egui::vec2(100.0, 32.0));

                if ui.add(btn).clicked() {
                    controller.start(config.clone(), handle);
                }
            }
            TestState::Running => {
                let pause_btn = egui::Button::new(
                    egui::RichText::new("暂停").size(13.0).color(egui::Color32::WHITE).strong()
                )
                    .fill(WARNING)
                    .min_size(egui::vec2(100.0, 32.0));

                if ui.add(pause_btn).clicked() {
                    controller.pause();
                }

                ui.add_space(8.0);

                let stop_btn = egui::Button::new(
                    egui::RichText::new("停止").size(13.0).color(egui::Color32::WHITE).strong()
                )
                    .fill(NEGATIVE)
                    .min_size(egui::vec2(100.0, 32.0));

                if ui.add(stop_btn).clicked() {
                    controller.stop();
                }
            }
            TestState::Paused => {
                let resume_btn = egui::Button::new(
                    egui::RichText::new("恢复").size(13.0).color(egui::Color32::WHITE).strong()
                )
                    .fill(ACCENT)
                    .min_size(egui::vec2(100.0, 32.0));

                if ui.add(resume_btn).clicked() {
                    controller.resume();
                }

                ui.add_space(8.0);

                let stop_btn = egui::Button::new(
                    egui::RichText::new("停止").size(13.0).color(egui::Color32::WHITE).strong()
                )
                    .fill(NEGATIVE)
                    .min_size(egui::vec2(100.0, 32.0));

                if ui.add(stop_btn).clicked() {
                    controller.stop();
                }
            }
        }
    });
}
