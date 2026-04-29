use crate::config::AppConfig;
use crate::control::{TestController, TestState};
use crate::history::{self, HistoryRecord, HistoryStore, ResultSummary};
use crate::theme;
use crate::ui::{config_panel, control_bar, dashboard};

#[derive(Clone, Debug, PartialEq)]
pub enum PanelTab {
    Config,
    History,
}

pub struct RltvizApp {
    pub config: AppConfig,
    pub controller: TestController,
    handle: tokio::runtime::Handle,
    curl_import_open: bool,
    curl_import_text: String,
    curl_import_error: Option<String>,
    history_store: HistoryStore,
    active_tab: PanelTab,
    selected_history: Option<usize>,
    prev_state: TestState,
}

impl RltvizApp {
    pub fn new(cc: &eframe::CreationContext<'_>, handle: tokio::runtime::Handle) -> Self {
        theme::apply_theme(&cc.egui_ctx);
        let history_path = history::history_path();
        Self {
            config: AppConfig::default(),
            controller: TestController::new(),
            handle,
            curl_import_open: false,
            curl_import_text: String::new(),
            curl_import_error: None,
            history_store: HistoryStore::load(history_path),
            active_tab: PanelTab::Config,
            selected_history: None,
            prev_state: TestState::Idle,
        }
    }
}

impl eframe::App for RltvizApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.controller.check_done();
        let state = self.controller.state.clone();
        let snapshot = self.controller.snapshot.read().clone();

        // Auto-save on transition to Stopped
        if state == TestState::Stopped && self.prev_state != TestState::Stopped {
            let summary = ResultSummary::from_snapshot(&snapshot);
            let record = HistoryRecord::new(self.config.clone(), summary);
            self.history_store.add(record);
            self.selected_history = self.selected_history.map(|i| i + 1);
        }

        // Clear selection when a test starts
        if state == TestState::Running && self.prev_state != TestState::Running {
            self.selected_history = None;
        }

        self.prev_state = state.clone();

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
                    &mut self.active_tab,
                    self.history_store.records(),
                    &mut self.selected_history,
                );
                ui.add_space(18.0);
                control_bar::show(ui, &state, &mut self.controller, &self.config, &self.handle);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Frame::NONE
                .inner_margin(egui::Margin::same(24))
                .show(ui, |ui| {
                    let is_live =
                        state == TestState::Running || state == TestState::Paused;

                    if is_live {
                        dashboard::show(ui, &snapshot, &state);
                    } else if let Some(idx) = self.selected_history {
                        if let Some(record) = self.history_store.records().get(idx) {
                            let reuse_clicked = dashboard::show_history(ui, record);
                            if reuse_clicked {
                                self.config = record.config.clone();
                                self.active_tab = PanelTab::Config;
                                self.selected_history = None;
                                self.controller.start(self.config.clone(), &self.handle);
                            }
                        } else {
                            self.selected_history = None;
                            dashboard::show(ui, &snapshot, &state);
                        }
                    } else {
                        dashboard::show(ui, &snapshot, &state);
                    }
                });
        });

        if state == TestState::Running || state == TestState::Paused {
            ctx.request_repaint();
        }
    }
}
