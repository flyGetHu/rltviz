use egui::{Color32, Context, CornerRadius, Stroke};
use std::sync::Arc;

// ── Color palette ──────────────────────────────────────────

pub const BG_PRIMARY: Color32 = Color32::from_rgb(0xFF, 0xFF, 0xFF);
pub const BG_SECONDARY: Color32 = Color32::from_rgb(0xF5, 0xF5, 0xF7);
pub const BG_TERTIARY: Color32 = Color32::from_rgb(0xEB, 0xEB, 0xED);
pub const BG_HOVERED: Color32 = Color32::from_rgb(0xE0, 0xE0, 0xE5);
pub const BORDER: Color32 = Color32::from_rgb(0xD1, 0xD1, 0xD6);
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(0x1D, 0x1D, 0x1F);
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(0x6E, 0x6E, 0x73);
pub const TEXT_TERTIARY: Color32 = Color32::from_rgb(0xAE, 0xAE, 0xB2);
pub const ACCENT: Color32 = Color32::from_rgb(0x00, 0x7A, 0xFF);
#[allow(dead_code)]
pub const ACCENT_HOVER: Color32 = Color32::from_rgb(0x00, 0x62, 0xCC);
pub const POSITIVE: Color32 = Color32::from_rgb(0x34, 0xC7, 0x59);
pub const NEGATIVE: Color32 = Color32::from_rgb(0xFF, 0x3B, 0x30);
pub const WARNING: Color32 = Color32::from_rgb(0xFF, 0x95, 0x00);

// ── Typography helpers ─────────────────────────────────────

pub fn heading(text: &str) -> egui::RichText {
    egui::RichText::new(text)
        .size(15.0)
        .color(TEXT_PRIMARY)
        .strong()
}

pub fn body(text: &str) -> egui::RichText {
    egui::RichText::new(text)
        .size(13.0)
        .color(TEXT_PRIMARY)
}

pub fn body_small(text: &str) -> egui::RichText {
    egui::RichText::new(text)
        .size(11.0)
        .color(TEXT_SECONDARY)
}

pub fn metric_value(val: &str, color: Color32) -> egui::RichText {
    egui::RichText::new(val)
        .size(28.0)
        .color(color)
        .strong()
}

pub fn metric_label(text: &str) -> egui::RichText {
    egui::RichText::new(text)
        .size(11.0)
        .color(TEXT_TERTIARY)
}

#[allow(dead_code)]
pub fn mono(text: &str) -> egui::RichText {
    egui::RichText::new(text)
        .color(TEXT_PRIMARY)
        .font(egui::FontId::monospace(12.0))
}

// ── Global theme application ───────────────────────────────

pub fn apply_theme(ctx: &Context) {
    let mut style = (*ctx.style()).clone();
    let visuals = &mut style.visuals;

    // Light mode — required for DragValue, ComboBox etc. to use light backgrounds
    visuals.dark_mode = false;

    // Panel backgrounds
    visuals.window_fill = BG_PRIMARY;
    visuals.panel_fill = BG_SECONDARY;

    // Widget defaults
    let rounding = CornerRadius::same(6);
    visuals.widgets.noninteractive.corner_radius = rounding;
    visuals.widgets.inactive.corner_radius = rounding;
    visuals.widgets.hovered.corner_radius = rounding;
    visuals.widgets.active.corner_radius = rounding;

    visuals.widgets.noninteractive.bg_fill = BG_PRIMARY;
    visuals.widgets.noninteractive.weak_bg_fill = BG_PRIMARY;
    visuals.widgets.inactive.bg_fill = BG_TERTIARY;
    visuals.widgets.inactive.weak_bg_fill = BG_TERTIARY;
    visuals.widgets.hovered.bg_fill = BG_HOVERED;
    visuals.widgets.hovered.weak_bg_fill = BG_HOVERED;
    visuals.widgets.active.bg_fill = ACCENT;
    visuals.widgets.active.weak_bg_fill = ACCENT;
    visuals.widgets.open.bg_fill = BG_TERTIARY;
    visuals.widgets.open.weak_bg_fill = BG_TERTIARY;

    let subtle_border = Stroke::new(1.0, BORDER);
    visuals.widgets.inactive.bg_stroke = subtle_border;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, ACCENT);
    visuals.widgets.active.bg_stroke = Stroke::new(2.0, ACCENT);

    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);
    visuals.widgets.active.fg_stroke = Stroke::new(2.0, ACCENT);

    // Selection color
    visuals.selection.bg_fill = Color32::from_rgba_premultiplied(0x00, 0x1E, 0x40, 0x40);

    // Hyperlink
    visuals.hyperlink_color = ACCENT;

    // Input backgrounds — extreme_bg_color for TextEdit, code_bg_color for DragValue/ComboBox
    visuals.extreme_bg_color = BG_TERTIARY;
    visuals.code_bg_color = BG_TERTIARY;

    // Subtle background for inputs (button_frame = true required for bg_fill to render)
    visuals.striped = false;
    visuals.button_frame = true;
    visuals.indent_has_left_vline = false;

    // Window (popup) corner radius
    visuals.window_corner_radius = CornerRadius::same(8);
    visuals.window_stroke = Stroke::new(1.0, BORDER);

    // Default text color override — softer black
    visuals.override_text_color = Some(TEXT_PRIMARY);

    ctx.set_style(style);

    setup_fonts(ctx);
}

fn setup_fonts(ctx: &Context) {
    let mut fonts = egui::FontDefinitions::empty();

    let jb_regular: &[u8] = include_bytes!("../fonts/JetBrainsMono-Regular.ttf");
    let jb_bold: &[u8] = include_bytes!("../fonts/JetBrainsMono-Bold.ttf");
    fonts.font_data
        .insert("JetBrainsMono-Regular".into(), Arc::new(egui::FontData::from_static(jb_regular)));
    fonts.font_data
        .insert("JetBrainsMono-Bold".into(), Arc::new(egui::FontData::from_static(jb_bold)));

    let noto_sc: &[u8] = include_bytes!("../fonts/NotoSansSC-Regular.ttf");
    fonts.font_data
        .insert("NotoSansSC".into(), Arc::new(egui::FontData::from_static(noto_sc)));

    fonts.families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .push("NotoSansSC".into());

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
