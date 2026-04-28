use egui::Context;

pub fn setup_theme(ctx: &Context) {
    let mut style = (*ctx.style()).clone();
    style.visuals.striped = true;
    ctx.set_style(style);

    setup_cjk_font(ctx);
}

fn setup_cjk_font(ctx: &Context) {
    let mut fonts = egui::FontDefinitions::default();

    // Add system CJK fonts for Chinese text rendering
    // Windows: Microsoft YaHei (微软雅黑), SimHei (黑体)
    // macOS: PingFang SC, STHeiti
    // Linux: Noto Sans CJK SC, WenQuanYi Micro Hei
    let cjk_fonts = [
        "Microsoft YaHei",
        "PingFang SC",
        "Noto Sans CJK SC",
        "SimHei",
        "WenQuanYi Micro Hei",
    ];

    for family in [
        egui::FontFamily::Proportional,
        egui::FontFamily::Monospace,
    ] {
        if let Some(fonts_list) = fonts.families.get_mut(&family) {
            for cjk in &cjk_fonts {
                fonts_list.push(cjk.to_string());
            }
        }
    }

    ctx.set_fonts(fonts);
}
