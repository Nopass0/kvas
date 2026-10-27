//! Visual building blocks: flat surfaces with a 1px border, bars, cards and the animation helpers.

use crate::theme::{pal, ACCENT};
use eframe::egui::{self, Color32, Pos2, Rect, Shape, Stroke, Ui};

/// Smoothly eased 0..1 value following `target` (for hover / open animations).
pub fn animate(ui: &Ui, id: egui::Id, target: bool) -> f32 {
    ui.ctx().animate_bool_with_time_and_easing(id, target, 0.12, egui::emath::easing::cubic_out)
}

pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(
        (a.r() as f32 + (b.r() as f32 - a.r() as f32) * t) as u8,
        (a.g() as f32 + (b.g() as f32 - a.g() as f32) * t) as u8,
        (a.b() as f32 + (b.b() as f32 - a.b() as f32) * t) as u8,
        (a.a() as f32 + (b.a() as f32 - a.a() as f32) * t) as u8,
    )
}

/// A panel floating over the 3D view (tool strip, statistics pill).
pub fn float_panel(rect: Rect, radius: f32) -> Shape {
    let p = pal();
    let fill = Color32::from_rgba_unmultiplied(p.panel.r(), p.panel.g(), p.panel.b(), 238);
    Shape::Vec(vec![Shape::rect_filled(rect, radius, fill), Shape::rect_stroke(rect, radius, Stroke::new(1.0, p.border_strong), egui::StrokeKind::Inside)])
}

/// Full-width bar background (title bar, toolbars) with a hairline at the bottom.
pub fn bar_background(ui: &Ui, rect: Rect) {
    let p = pal();
    let painter = ui.painter();
    painter.rect_filled(rect, 0.0, p.bar);
    let y = rect.bottom() - 0.5;
    painter.line_segment([Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)], Stroke::new(1.0, if crate::theme::is_light() { p.border_strong } else { Color32::from_rgb(0x1E, 0x1F, 0x22) }));
}

/// A rounded tile (asset thumbnails, project rows): hover and selection are animated.
pub fn tile(ui: &Ui, rect: Rect, id: egui::Id, hovered: bool, selected: bool) {
    let p = pal();
    let hover = animate(ui, id.with("tile-hover"), hovered);
    let sel = animate(ui, id.with("tile-sel"), selected);
    let fill = mix(mix(Color32::TRANSPARENT, p.hover, hover), p.selection, sel);
    ui.painter().rect_filled(rect, 8.0, fill);
    if sel > 0.01 {
        ui.painter().rect_stroke(rect, 8.0, Stroke::new(1.0, ACCENT.gamma_multiply(sel)), egui::StrokeKind::Inside);
    }
}

/// A rounded square with initials, used as a project avatar (like the JetBrains Welcome screen).
pub fn avatar(ui: &mut Ui, text: &str, size: f32, seed: u32) -> egui::Response {
    const COLORS: [(u8, u8, u8); 6] = [(0xE0, 0x6C, 0x3C), (0x35, 0x74, 0xF0), (0x3D, 0x9A, 0x6B), (0xA8, 0x5F, 0xD8), (0xD6, 0x4F, 0x8B), (0x2F, 0x9B, 0xB5)];
    let (r, g, b) = COLORS[seed as usize % COLORS.len()];
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
    ui.painter().rect_filled(rect, size * 0.24, Color32::from_rgb(r, g, b));
    let initials: String = text.split_whitespace().filter_map(|w| w.chars().next()).take(2).collect::<String>().to_uppercase();
    ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, initials, egui::FontId::proportional(size * 0.42), Color32::WHITE);
    resp
}

/// A card with a clickable header (icon, bold title, chevron) and an animated body: the building
/// block of the inspector. Returns the header response (for context menus).
pub fn section(ui: &mut Ui, id: egui::Id, icon: Option<crate::icons::Icon>, title: &str, default_open: bool, add: impl FnOnce(&mut Ui)) -> egui::Response {
    use egui::collapsing_header::CollapsingState;
    let p = pal();
    let mut state = CollapsingState::load_with_default_open(ui.ctx(), id, default_open);
    let outer = egui::Frame::NONE
        .fill(p.card)
        .stroke(Stroke::new(1.0, if crate::theme::is_light() { p.border } else { Color32::from_rgb(0x3E, 0x40, 0x46) }))
        .corner_radius(8)
        .inner_margin(egui::Margin { left: 0, right: 0, top: 0, bottom: 0 })
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 30.0), egui::Sense::click());
            let hover = animate(ui, resp.id.with("hover"), resp.hovered());
            ui.painter().rect_filled(rect.shrink(1.0), 7.0, mix(Color32::TRANSPARENT, p.hover, hover * 0.6));
            let openness = state.openness(ui.ctx());
            let chevron = if openness > 0.5 { crate::icons::Icon::ChevronDown } else { crate::icons::Icon::ChevronRight };
            chevron.image(13.0, p.text_dim).paint_at(ui, egui::Rect::from_center_size(rect.left_center() + egui::vec2(15.0, 0.0), egui::Vec2::splat(13.0)));
            let mut x = 32.0;
            if let Some(i) = icon {
                i.image(15.0, ACCENT).paint_at(ui, egui::Rect::from_center_size(rect.left_center() + egui::vec2(x + 7.0, 0.0), egui::Vec2::splat(15.0)));
                x += 24.0;
            }
            ui.painter().text(rect.left_center() + egui::vec2(x, 0.0), egui::Align2::LEFT_CENTER, title, egui::FontId::new(13.5, crate::theme::bold_family()), p.text);
            if resp.clicked() {
                state.toggle(ui);
            }
            state.show_body_unindented(ui, |ui| {
                // A hairline under the header, then the content with comfortable padding.
                let y = ui.cursor().top();
                ui.painter().line_segment([Pos2::new(rect.left() + 8.0, y), Pos2::new(rect.right() - 8.0, y)], Stroke::new(1.0, p.border_strong.gamma_multiply(0.5)));
                egui::Frame::NONE.inner_margin(egui::Margin { left: 12, right: 12, top: 8, bottom: 10 }).show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    add(ui)
                });
            });
            resp
        });
    outer.inner
}

/// A floating card that follows the pointer while an asset or script is dragged: it grows in when the
/// drag starts, tilts slightly with the motion and shrinks away where it was dropped.
pub fn drag_ghost(ctx: &egui::Context) {
    use crate::assets_panel::{AssetDrag, ScriptDrag};
    let label = egui::DragAndDrop::payload::<AssetDrag>(ctx)
        .map(|a| (a.0.rsplit('/').next().unwrap_or(&a.0).to_string(), a.0.rsplit('.').next().unwrap_or("").to_string()))
        .or_else(|| egui::DragAndDrop::payload::<ScriptDrag>(ctx).map(|s| (s.0.clone(), "rs".to_string())))
        .or_else(|| egui::DragAndDrop::payload::<kvas_engine::prelude::Entity>(ctx).map(|e| (format!("Object {}", e.index()), "obj".to_string())));
    let pointer = ctx.input(|i| i.pointer.hover_pos().or(i.pointer.latest_pos()));
    let id = egui::Id::unique("drag-ghost");
    let last: Option<(String, String, egui::Pos2, f64)> = ctx.memory(|m| m.data.get_temp(id));
    let now = ctx.input(|i| i.time);
    let (name, ext, pos, grow) = match (&label, pointer) {
        (Some((n, e)), Some(p)) => {
            ctx.memory_mut(|m| m.data.insert_temp(id, (n.clone(), e.clone(), p, now)));
            ctx.set_cursor_icon(egui::CursorIcon::Grabbing);
            (n.clone(), e.clone(), p, ctx.animate_value_with_time(id.with("in"), 1.0, 0.14))
        }
        _ => {
            ctx.animate_value_with_time(id.with("in"), 0.0, 0.0);
            let Some((n, e, p, t0)) = last else { return };
            // Dropped: shrink and fade over a fifth of a second.
            let t = 1.0 - ((now - t0) / 0.2) as f32;
            if t <= 0.0 {
                ctx.memory_mut(|m| m.data.remove::<(String, String, egui::Pos2, f64)>(id));
                return;
            }
            (n, e, p, t)
        }
    };
    let scale = 0.7 + 0.3 * grow;
    let alpha = (grow * 235.0) as u8;
    let accent = crate::theme::ACCENT;
    let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Tooltip, id));
    let text_font = egui::FontId::proportional(12.5 * scale);
    let galley = painter.layout_no_wrap(name, text_font, Color32::from_rgba_unmultiplied(235, 235, 240, alpha));
    let size = egui::vec2(galley.size().x + 44.0 * scale, 30.0 * scale);
    let rect = egui::Rect::from_min_size(pos + egui::vec2(14.0, 10.0), size);
    painter.add(Shape::rect_filled(rect.translate(egui::vec2(2.0, 4.0)).expand(1.0), 9.0, Color32::from_black_alpha((60.0 * grow) as u8)));
    painter.rect_filled(rect, 9.0 * scale, Color32::from_rgba_unmultiplied(0x2B, 0x2D, 0x30, alpha));
    painter.rect_stroke(rect, 9.0 * scale, egui::Stroke::new(1.5, Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), alpha)), egui::StrokeKind::Inside);
    let badge = egui::Rect::from_min_size(rect.min + egui::vec2(6.0 * scale, 5.0 * scale), egui::vec2(20.0 * scale, 20.0 * scale));
    painter.rect_filled(badge, 5.0 * scale, Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), alpha));
    painter.text(badge.center(), egui::Align2::CENTER_CENTER, ext.chars().take(3).collect::<String>().to_uppercase(), egui::FontId::proportional(8.0 * scale), Color32::from_rgba_unmultiplied(255, 255, 255, alpha));
    painter.galley(egui::pos2(badge.max.x + 7.0 * scale, rect.center().y - galley.size().y / 2.0), galley, Color32::WHITE);
    ctx.request_repaint();
}
