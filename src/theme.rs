use egui::Context;
use std::sync::Arc;

pub fn setup_theme(ctx: &Context) {
    let mut style = (*ctx.style()).clone();
    style.visuals.striped = true;
    ctx.set_style(style);

    setup_fonts(ctx);
}

fn setup_fonts(ctx: &Context) {
    let mut fonts = egui::FontDefinitions::empty();

    // JetBrains Mono for monospace (code, numbers, latency values)
    let jb_regular: &[u8] = include_bytes!("../fonts/JetBrainsMono-Regular.ttf");
    let jb_bold: &[u8] = include_bytes!("../fonts/JetBrainsMono-Bold.ttf");
    fonts.font_data
        .insert("JetBrainsMono-Regular".into(), Arc::new(egui::FontData::from_static(jb_regular)));
    fonts.font_data
        .insert("JetBrainsMono-Bold".into(), Arc::new(egui::FontData::from_static(jb_bold)));

    // Noto Sans SC for CJK character fallback
    let noto_sc: &[u8] = include_bytes!("../fonts/NotoSansSC-Regular.ttf");
    fonts.font_data
        .insert("NotoSansSC".into(), Arc::new(egui::FontData::from_static(noto_sc)));

    // Proportional: default egui UI font → Noto Sans SC for CJK
    fonts.families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .push("NotoSansSC".into());

    // Monospace: JetBrains Mono → Noto Sans SC for CJK
    fonts.families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "JetBrainsMono-Regular".into());
    fonts.families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("NotoSansSC".into());

    ctx.set_fonts(fonts);
}
