use eframe::Frame;

pub struct RltvizApp;

impl RltvizApp {
    pub fn new(cc: &eframe::CreationContext<'_>, handle: tokio::runtime::Handle) -> Self {
        Self
    }
}

impl eframe::App for RltvizApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut Frame) {}
}
