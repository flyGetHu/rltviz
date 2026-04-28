use eframe::Frame;

pub struct RltvizApp;

impl RltvizApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, _handle: tokio::runtime::Handle) -> Self {
        Self
    }
}

impl eframe::App for RltvizApp {
    fn update(&mut self, _ctx: &egui::Context, _frame: &mut Frame) {}
}
