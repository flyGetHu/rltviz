use crate::config::AppConfig;
use crate::control::{TestController, TestState};

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
                let start_btn = egui::Button::new("▶ 启动")
                    .fill(egui::Color32::from_rgb(76, 175, 80))
                    .min_size(egui::vec2(100.0, 32.0));

                if ui.add_enabled(true, start_btn).clicked() {
                    controller.start(config.clone(), handle);
                }
            }
            TestState::Running => {
                let pause_btn = egui::Button::new("⏸ 暂停")
                    .fill(egui::Color32::from_rgb(255, 152, 0))
                    .min_size(egui::vec2(100.0, 32.0));

                if ui.add(pause_btn).clicked() {
                    controller.pause();
                }

                let stop_btn = egui::Button::new("⏹ 停止")
                    .fill(egui::Color32::from_rgb(244, 67, 54))
                    .min_size(egui::vec2(100.0, 32.0));

                if ui.add(stop_btn).clicked() {
                    controller.stop();
                }
            }
            TestState::Paused => {
                let resume_btn = egui::Button::new("▶ 恢复")
                    .fill(egui::Color32::from_rgb(76, 175, 80))
                    .min_size(egui::vec2(100.0, 32.0));

                if ui.add(resume_btn).clicked() {
                    controller.resume();
                }

                let stop_btn = egui::Button::new("⏹ 停止")
                    .fill(egui::Color32::from_rgb(244, 67, 54))
                    .min_size(egui::vec2(100.0, 32.0));

                if ui.add(stop_btn).clicked() {
                    controller.stop();
                }
            }
        }
    });
}
