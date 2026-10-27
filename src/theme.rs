use eframe::egui::{
    self, Color32, FontId, Rounding, Stroke, TextStyle, Vec2, Visuals,
};

pub const ACCENT: Color32 = Color32::from_rgb(15, 108, 189);
pub const ACCENT_DARK: Color32 = Color32::from_rgb(9, 78, 140);
pub const ACCENT_SOFT: Color32 = Color32::from_rgb(229, 241, 251);
pub const RIBBON_BG: Color32 = Color32::from_rgb(243, 243, 243);
pub const RIBBON_BODY: Color32 = Color32::from_rgb(250, 250, 250);
pub const TAB_ACTIVE: Color32 = Color32::WHITE;
pub const BORDER: Color32 = Color32::from_rgb(214, 216, 219);
pub const TEXT: Color32 = Color32::from_rgb(32, 32, 32);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(105, 105, 105);
pub const TEXT_DIM: Color32 = Color32::from_rgb(140, 140, 140);
pub const ROW_ALT: Color32 = Color32::from_rgb(248, 250, 252);
pub const ROW_HOVER: Color32 = Color32::from_rgb(238, 246, 253);
pub const SELECT: Color32 = Color32::from_rgb(204, 232, 255);
pub const SELECT_BORDER: Color32 = Color32::from_rgb(153, 209, 255);
pub const FOLDER: Color32 = Color32::from_rgb(242, 177, 74);
pub const FOLDER_DARK: Color32 = Color32::from_rgb(205, 141, 45);
pub const ICON_SOFT: Color32 = Color32::from_rgb(150, 165, 180);
pub const FILE: Color32 = Color32::from_rgb(110, 128, 145);
pub const GREEN: Color32 = Color32::from_rgb(45, 145, 90);
pub const DANGER: Color32 = Color32::from_rgb(194, 45, 34);

pub fn install(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    let mut visuals = Visuals::light();
    visuals.panel_fill = Color32::WHITE;
    visuals.window_fill = Color32::WHITE;
    visuals.faint_bg_color = ROW_ALT;
    visuals.extreme_bg_color = Color32::WHITE;
    visuals.override_text_color = Some(TEXT);
    visuals.selection.bg_fill = SELECT;
    visuals.selection.stroke = Stroke::new(1.0_f32, ACCENT);
    visuals.widgets.noninteractive.bg_fill = RIBBON_BG;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, BORDER);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, TEXT);
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(252, 252, 252);
    visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(248, 248, 248);
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, BORDER);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, TEXT);
    visuals.widgets.hovered.bg_fill = ACCENT_SOFT;
    visuals.widgets.hovered.weak_bg_fill = ACCENT_SOFT;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, ACCENT);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, TEXT);
    visuals.widgets.active.bg_fill = Color32::from_rgb(204, 228, 247);
    visuals.widgets.active.weak_bg_fill = Color32::from_rgb(204, 228, 247);
    visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, ACCENT);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, TEXT);
    visuals.window_stroke = Stroke::new(1.0_f32, BORDER);
    visuals.window_rounding = Rounding::same(4.0);
    visuals.menu_rounding = Rounding::same(4.0);
    style.visuals = visuals;

    style.spacing.item_spacing = Vec2::new(5.0, 4.0);
    style.spacing.button_padding = Vec2::new(7.0, 3.0);
    style.spacing.interact_size = Vec2::new(22.0, 22.0);
    style.spacing.scroll.bar_width = 11.0;
    style.spacing.scroll.floating = false;
    style.spacing.menu_margin = egui::Margin::same(4.0);

    style
        .text_styles
        .insert(TextStyle::Heading, FontId::proportional(15.0));
    style
        .text_styles
        .insert(TextStyle::Body, FontId::proportional(12.5));
    style
        .text_styles
        .insert(TextStyle::Button, FontId::proportional(12.5));
    style
        .text_styles
        .insert(TextStyle::Small, FontId::proportional(10.5));
    style
        .text_styles
        .insert(TextStyle::Monospace, FontId::monospace(12.0));

    ctx.set_style(style);
}
