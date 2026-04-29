mod app;
mod config;
mod control;
mod engine;
mod history;
mod metrics;
mod theme;
mod ui;

fn main() {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    let _guard = rt.enter();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0]),
        ..Default::default()
    };

    eframe::run_native(
        "rltviz - HTTP Load Test Visualizer",
        options,
        Box::new(|cc| Ok(Box::new(app::RltvizApp::new(cc, rt.handle().clone())))),
    )
    .expect("Failed to start eframe");
}
