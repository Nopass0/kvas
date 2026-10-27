//! Flat, rounded look modelled on the JetBrains "New UI" (CLion, RustRover): calm grey surfaces,
//! 1px borders, a blue accent and a two-tone selection.

use eframe::egui::{self, Color32, CornerRadius, FontFamily, FontId, Stroke, TextStyle};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};

pub const ACCENT: Color32 = Color32::from_rgb(0x35, 0x74, 0xF0);

#[derive(Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Debug)]
pub enum Theme {
    #[default]
    Dark,
    Light,
}

impl Theme {
    pub fn name(self) -> &'static str {
        match self {
            Theme::Dark => crate::i18n::t("Dark theme"),
            Theme::Light => crate::i18n::t("Light theme"),
        }
    }
}

/// Colours that the widgets pick up at draw time.
#[derive(Clone, Copy)]
pub struct Palette {
    /// Tool windows, menus, dialogs.
    pub panel: Color32,
    /// Main toolbar and tab strips.
    pub bar: Color32,
    /// Text fields, lists, the code-like areas.
    pub field: Color32,
    pub border: Color32,
    /// Window background seen between docked panels.
    pub gap: Color32,
    pub border_strong: Color32,
    pub text: Color32,
    pub text_dim: Color32,
    pub hover: Color32,
    pub selection: Color32,
    pub card: Color32,
    pub button: Color32,
    pub button_hover: Color32,
}

const DARK: Palette = Palette {
    panel: Color32::from_rgb(0x2B, 0x2D, 0x30),
    bar: Color32::from_rgb(0x2B, 0x2D, 0x30),
    field: Color32::from_rgb(0x1E, 0x1F, 0x22),
    border: Color32::from_rgb(0x1E, 0x1F, 0x22),
    gap: Color32::from_rgb(0x1E, 0x1F, 0x22),
    border_strong: Color32::from_rgb(0x4E, 0x51, 0x57),
    text: Color32::from_rgb(0xDF, 0xE1, 0xE5),
    text_dim: Color32::from_rgb(0x86, 0x8A, 0x91),
    hover: Color32::from_rgb(0x39, 0x3B, 0x40),
    selection: Color32::from_rgb(0x2E, 0x43, 0x6E),
    card: Color32::from_rgb(0x31, 0x33, 0x38),
    button: Color32::from_rgb(0x2B, 0x2D, 0x30),
    button_hover: Color32::from_rgb(0x39, 0x3B, 0x40),
};

const LIGHT: Palette = Palette {
    panel: Color32::from_rgb(0xF7, 0xF8, 0xFA),
    bar: Color32::from_rgb(0xF7, 0xF8, 0xFA),
    field: Color32::from_rgb(0xFF, 0xFF, 0xFF),
    border: Color32::from_rgb(0xEB, 0xEC, 0xF0),
    gap: Color32::from_rgb(0xEB, 0xEC, 0xF0),
    border_strong: Color32::from_rgb(0xC9, 0xCC, 0xD6),
    text: Color32::from_rgb(0x1E, 0x1F, 0x22),
    text_dim: Color32::from_rgb(0x6C, 0x70, 0x7E),
    hover: Color32::from_rgb(0xDF, 0xE1, 0xE5),
    selection: Color32::from_rgb(0xD4, 0xE2, 0xFF),
    card: Color32::from_rgb(0xFF, 0xFF, 0xFF),
    button: Color32::from_rgb(0xFF, 0xFF, 0xFF),
    button_hover: Color32::from_rgb(0xF0, 0xF1, 0xF4),
};

static LIGHT_ON: AtomicBool = AtomicBool::new(false);

pub fn pal() -> Palette {
    if LIGHT_ON.load(Ordering::Relaxed) {
        LIGHT
    } else {
        DARK
    }
}

pub fn is_light() -> bool {
    LIGHT_ON.load(Ordering::Relaxed)
}

/// Colour of the play button and "ok" status text.
pub fn green() -> Color32 {
    if is_light() { Color32::from_rgb(0x3D, 0x8F, 0x4E) } else { Color32::from_rgb(0x5F, 0xB8, 0x65) }
}

pub fn red() -> Color32 {
    if is_light() { Color32::from_rgb(0xD0, 0x3B, 0x3B) } else { Color32::from_rgb(0xF0, 0x6B, 0x6B) }
}

/// Family used wherever text should be bold (egui has no synthetic bold, so a real bold face is registered).
pub fn bold_family() -> FontFamily {
    FontFamily::Name("kvas-bold".into())
}

fn first_font(paths: &[&str]) -> Option<Vec<u8>> {
    paths.iter().find_map(|p| std::fs::read(p).ok())
}

fn install_fonts(ctx: &egui::Context) {
    // System UI fonts read closer to the JetBrains look than the bundled one. Cyrillic is covered
    // by both, so a missing file only changes the style. Without a bold file the regular face is
    // used for bold text as well, which keeps `bold()` working everywhere.
    let regular = first_font(&[r"C:\Windows\Fonts\segoeui.ttf", "/usr/share/fonts/truetype/inter/Inter-Regular.ttf", "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", "/System/Library/Fonts/SFNS.ttf"]);
    let semibold = first_font(&[r"C:\Windows\Fonts\seguisb.ttf", r"C:\Windows\Fonts\segoeuib.ttf", "/usr/share/fonts/truetype/inter/Inter-SemiBold.ttf", "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf"]);
    let mut fonts = egui::FontDefinitions::default();
    let base: Vec<String> = fonts.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
    let mut regular_names = Vec::new();
    if let Some(bytes) = regular {
        fonts.font_data.insert("kvas-ui".into(), std::sync::Arc::new(egui::FontData::from_owned(bytes)));
        regular_names.push("kvas-ui".to_string());
    }
    let mut bold_names = Vec::new();
    if let Some(bytes) = semibold {
        fonts.font_data.insert("kvas-ui-bold".into(), std::sync::Arc::new(egui::FontData::from_owned(bytes)));
        bold_names.push("kvas-ui-bold".to_string());
    }
    let mut prop = regular_names.clone();
    prop.extend(base.iter().cloned());
    fonts.families.insert(FontFamily::Proportional, prop.clone());
    let mut bold_list = bold_names;
    bold_list.extend(prop);
    fonts.families.insert(bold_family(), bold_list);
    ctx.set_fonts(fonts);
}

pub fn init(ctx: &egui::Context) {
    install_fonts(ctx);
}

pub fn apply(ctx: &egui::Context, theme: Theme, scale: f32) {
    LIGHT_ON.store(theme == Theme::Light, Ordering::Relaxed);
    let p = pal();
    let mut v = if theme == Theme::Light { egui::Visuals::light() } else { egui::Visuals::dark() };
    v.panel_fill = p.panel;
    v.window_fill = p.panel;
    v.extreme_bg_color = p.field;
    v.faint_bg_color = p.card;
    v.code_bg_color = p.field;
    v.window_stroke = Stroke::new(1.0, p.border_strong);
    v.window_corner_radius = CornerRadius::same(10);
    v.menu_corner_radius = CornerRadius::same(8);
    let shadow = Color32::from_black_alpha(if theme == Theme::Light { 50 } else { 120 });
    v.window_shadow = egui::Shadow { offset: [0, 6], blur: 24, spread: 0, color: shadow };
    v.popup_shadow = egui::Shadow { offset: [0, 4], blur: 16, spread: 0, color: shadow };
    v.interact_cursor = Some(egui::CursorIcon::PointingHand);
    v.selection.bg_fill = p.selection;
    v.selection.stroke = Stroke::new(1.0, p.text);
    v.hyperlink_color = ACCENT;
    v.weak_text_color = Some(p.text_dim);
    v.striped = false;

    let radius = CornerRadius::same(6);
    for w in [&mut v.widgets.noninteractive, &mut v.widgets.inactive, &mut v.widgets.hovered, &mut v.widgets.active, &mut v.widgets.open] {
        w.corner_radius = radius;
        w.expansion = 0.0;
    }
    v.widgets.noninteractive.bg_fill = p.panel;
    v.widgets.noninteractive.weak_bg_fill = p.panel;
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, if theme == Theme::Light { p.border_strong } else { Color32::from_rgb(0x39, 0x3B, 0x40) });
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, p.text);
    // Slider rails, checkbox boxes and other control backgrounds.
    v.widgets.inactive.bg_fill = if theme == Theme::Light { Color32::from_rgb(0xDF, 0xE1, 0xE5) } else { Color32::from_rgb(0x43, 0x45, 0x4A) };
    v.widgets.inactive.weak_bg_fill = p.button;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, p.border_strong);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, p.text);
    v.widgets.hovered.bg_fill = p.button_hover;
    v.widgets.hovered.weak_bg_fill = p.button_hover;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, if theme == Theme::Light { Color32::from_rgb(0xA0, 0xA4, 0xB0) } else { Color32::from_rgb(0x6F, 0x73, 0x7A) });
    v.widgets.hovered.fg_stroke = Stroke::new(1.0, p.text);
    v.widgets.active.bg_fill = ACCENT;
    v.widgets.active.weak_bg_fill = p.hover;
    v.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT);
    v.widgets.active.fg_stroke = Stroke::new(1.0, p.text);
    v.widgets.open.bg_fill = p.hover;
    v.widgets.open.weak_bg_fill = p.hover;
    v.widgets.open.bg_stroke = Stroke::new(1.0, p.border_strong);
    ctx.set_visuals(v);

    ctx.set_zoom_factor(scale.clamp(0.75, 2.0));
    ctx.global_style_mut(|style| {
        style.spacing.item_spacing = egui::vec2(8.0, 5.0);
        style.spacing.button_padding = egui::vec2(10.0, 4.0);
        style.spacing.interact_size.y = 24.0;
        style.spacing.menu_margin = egui::Margin::same(6);
        style.spacing.window_margin = egui::Margin::same(12);
        style.spacing.indent = 16.0;
        // Slower, smoother colour fades on hover and press.
        style.animation_time = 0.14;
        style.spacing.combo_width = 120.0;
        style.spacing.slider_rail_height = 4.0;
        style.spacing.scroll.bar_width = 8.0;
        style.spacing.scroll.floating = true;
        style.text_styles.insert(TextStyle::Body, FontId::proportional(13.0));
        style.text_styles.insert(TextStyle::Button, FontId::proportional(13.0));
        style.text_styles.insert(TextStyle::Small, FontId::proportional(11.0));
        style.text_styles.insert(TextStyle::Heading, FontId::new(17.0, bold_family()));
        style.text_styles.insert(TextStyle::Monospace, FontId::monospace(12.5));
    });
}

/// Tab strips and separators of the docking layout.
pub fn dock_style(base: &egui::Style) -> egui_dock::Style {
    let p = pal();
    let mut s = egui_dock::Style::from_egui(base);
    s.main_surface_border_stroke = Stroke::NONE;
    s.dock_area_padding = Some(egui::Margin::same(4));
    s.separator.width = 4.0;
    s.separator.extra_interact_width = 4.0;
    s.separator.color_idle = Color32::TRANSPARENT;
    s.separator.color_hovered = ACCENT.gamma_multiply(0.6);
    s.separator.color_dragged = ACCENT;
    s.tab_bar.bg_fill = p.bar;
    s.tab_bar.height = 30.0;
    s.tab_bar.inner_margin = egui::Margin { left: 6, right: 6, top: 3, bottom: 0 };
    s.tab_bar.corner_radius = CornerRadius { nw: 8, ne: 8, sw: 0, se: 0 };
    s.tab_bar.hline_color = p.border_strong;
    s.tab.spacing = 2.0;
    s.tab.hline_below_active_tab_name = true;
    let tab = |bg: Color32, text: Color32| egui_dock::TabInteractionStyle { outline_color: Color32::TRANSPARENT, corner_radius: CornerRadius { nw: 6, ne: 6, sw: 0, se: 0 }, bg_fill: bg, text_color: text };
    s.tab.inactive = tab(Color32::TRANSPARENT, p.text_dim);
    s.tab.hovered = tab(p.hover, p.text);
    for active in [&mut s.tab.active, &mut s.tab.focused, &mut s.tab.active_with_kb_focus, &mut s.tab.focused_with_kb_focus] {
        *active = tab(p.panel, p.text);
    }
    s.tab.inactive_with_kb_focus = tab(Color32::TRANSPARENT, p.text_dim);
    s.tab.tab_body.bg_fill = p.panel;
    s.tab.tab_body.stroke = Stroke::new(1.0, p.border_strong);
    s.tab.tab_body.corner_radius = CornerRadius { nw: 0, ne: 0, sw: 8, se: 8 };
    s.tab.tab_body.inner_margin = egui::Margin::ZERO;
    s
}
