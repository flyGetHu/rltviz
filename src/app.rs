use crate::config::AppConfig;
use crate::control::{TestController, TestState};
use crate::theme;
use crate::ui::{config_panel, control_bar, dashboard};

pub struct RltvizApp {
    pub config: AppConfig,
    pub controller: TestController,
    handle: tokio::runtime::Handle,
    curl_import_open: bool,
    curl_import_text: String,
    curl_import_error: Option<String>,
}

impl RltvizApp {
    pub fn new(cc: &eframe::CreationContext<'_>, handle: tokio::runtime::Handle) -> Self {
        theme::apply_theme(&cc.egui_ctx);
        Self {
            config: AppConfig::default(),
            controller: TestController::new(),
            handle,
            curl_import_open: false,
            curl_import_text: String::new(),
            curl_import_error: None,
        }
    }
}

impl eframe::App for RltvizApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check if auto-stop completed
        self.controller.check_done();

        let state = self.controller.state.clone();
        let snapshot = self.controller.snapshot.read().clone();

        egui::SidePanel::left("config_panel")
            .resizable(true)
            .default_width(340.0)
            .min_width(280.0)
            .show(ctx, |ui| {
                let running = state == TestState::Running || state == TestState::Paused;
                config_panel::show(
                    ui,
                    &mut self.config,
                    running,
                    &mut self.curl_import_open,
                    &mut self.curl_import_text,
                    &mut self.curl_import_error,
                );
                ui.add_space(18.0);
                control_bar::show(ui, &state, &mut self.controller, &self.config, &self.handle);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Frame::NONE
                .inner_margin(egui::Margin::same(24))
                .show(ui, |ui| {
                    dashboard::show(ui, &snapshot, &state);
                });
        });

        // Request repaint for real-time metrics
        if state == TestState::Running || state == TestState::Paused {
            ctx.request_repaint();
        }
    }
}
