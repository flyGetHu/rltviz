use egui::Context;

pub fn setup_theme(ctx: &Context) {
    let mut style = (*ctx.style()).clone();

    style.visuals.striped = true;

    ctx.set_style(style);
}
