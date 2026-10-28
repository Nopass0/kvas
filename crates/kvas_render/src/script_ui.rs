//! Paints the immediate-mode interface that scripts describe through `ctx.ui()` (see
//! `kvas_engine::canvas`): panels, text in custom fonts, images, buttons and sliders, and reports
//! clicks back to the scripts for the next frame.

use eframe::egui;
use kvas_engine::assets::Assets;
use kvas_engine::canvas::{Align, ButtonStyle, UiCmd, UI_HEIGHT};
use kvas_engine::input::Input;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

thread_local! {
    static FONTS: RefCell<HashSet<String>> = RefCell::new(HashSet::new());
    static FAILED: RefCell<HashSet<String>> = RefCell::new(HashSet::new());
    static IMAGES: RefCell<HashMap<String, Option<egui::TextureHandle>>> = RefCell::new(HashMap::new());
}

/// Makes a `.ttf` / `.otf` asset available as a font family named after its path. Characters the
/// font lacks (Cyrillic in a Latin pixel font...) fall back to the default font.
fn font_family(ctx: &egui::Context, assets: &Assets, path: &str) -> egui::FontFamily {
    if path.is_empty() || FAILED.with(|f| f.borrow().contains(path)) {
        return egui::FontFamily::Proportional;
    }
    let known = FONTS.with(|f| f.borrow().contains(path));
    if !known {
        match assets.read(path) {
            Ok(bytes) => {
                let mut defs = ctx.fonts(|f| f.definitions().clone());
                defs.font_data.insert(path.to_string(), std::sync::Arc::new(egui::FontData::from_owned(bytes)));
                let mut family = vec![path.to_string()];
                family.extend(defs.families.get(&egui::FontFamily::Proportional).cloned().unwrap_or_default());
                defs.families.insert(egui::FontFamily::Name(path.into()), family);
                ctx.set_fonts(defs);
                FONTS.with(|f| f.borrow_mut().insert(path.to_string()));
                // The new font is ready from the next frame.
                return egui::FontFamily::Proportional;
            }
            Err(e) => {
                eprintln!("font {path}: {e}");
                FAILED.with(|f| f.borrow_mut().insert(path.to_string()));
                return egui::FontFamily::Proportional;
            }
        }
    }
    egui::FontFamily::Name(path.into())
}

fn image(ctx: &egui::Context, assets: &Assets, path: &str) -> Option<egui::TextureId> {
    IMAGES.with(|m| {
        let mut m = m.borrow_mut();
        let entry = m.entry(path.to_string()).or_insert_with(|| {
            let bytes = assets.read(path).ok()?;
            let img = image::load_from_memory(&bytes).ok()?.to_rgba8();
            let color = egui::ColorImage::from_rgba_unmultiplied([img.width() as usize, img.height() as usize], &img);
            // Small pixel-art images stay crisp.
            let options = if img.width() <= 256 { egui::TextureOptions::NEAREST } else { egui::TextureOptions::LINEAR };
            Some(ctx.load_texture(format!("script-ui:{path}"), color, options))
        });
        entry.as_ref().map(|t| t.id())
    })
}

fn color(c: [f32; 4]) -> egui::Color32 {
    let f = |v: f32| (v.clamp(0.0, 1.0) * 255.0) as u8;
    egui::Color32::from_rgba_unmultiplied(f(c[0]), f(c[1]), f(c[2]), f(c[3]))
}

/// Paints the scripts' interface over `rect` and feeds clicks, hovers and slider changes back.
/// `interactive` is false while the editor shows the game paused under its own overlays.
pub fn draw_script_ui(ui: &mut egui::Ui, rect: egui::Rect, input: &Input, assets: &Assets, interactive: bool) {
    let cmds = input.canvas.commands();
    let s = rect.height() / UI_HEIGHT;
    let width = rect.width() / s.max(1e-3);
    let to = |r: [f32; 4]| egui::Rect::from_min_size(rect.min + egui::vec2(r[0], r[1]) * s, egui::vec2(r[2], r[3]) * s);
    let ctx = ui.ctx().clone();
    let painter = ui.painter().with_clip_rect(rect);
    let (mut clicked, mut hovered, mut changed) = (Vec::new(), Vec::new(), Vec::new());
    let text = |painter: &egui::Painter, t: &str, at: egui::Pos2, size: f32, c: [f32; 4], font: &str, align: Align, wrap: f32| {
        let family = font_family(&ctx, assets, font);
        let fid = egui::FontId::new((size * s).max(4.0), family);
        let galley = if wrap > 0.0 { painter.layout(t.to_string(), fid, color(c), wrap * s) } else { painter.layout_no_wrap(t.to_string(), fid, color(c)) };
        let x = match align {
            Align::Left => at.x,
            Align::Center => at.x - galley.size().x * 0.5,
            Align::Right => at.x - galley.size().x,
        };
        painter.galley(egui::pos2(x, at.y), galley, color(c));
    };
    let pointer = ui.input(|i| i.pointer.hover_pos());
    let pressed = ui.input(|i| i.pointer.primary_clicked());
    let down = ui.input(|i| i.pointer.primary_down());
    let button_frame = |r: egui::Rect, style: &ButtonStyle, hot: bool| {
        painter.rect_filled(r, style.radius * s, color(if hot { style.hover } else { style.background }));
        if style.outline[3] > 0.0 {
            painter.rect_stroke(r, style.radius * s, egui::Stroke::new(1.5, color(style.outline)), egui::StrokeKind::Inside);
        }
    };
    for cmd in &cmds {
        match cmd {
            UiCmd::Panel { rect: r, color: c, radius, outline } => {
                let r = to(*r);
                painter.rect_filled(r, radius * s, color(*c));
                if outline[3] > 0.0 {
                    painter.rect_stroke(r, radius * s, egui::Stroke::new(1.5, color(*outline)), egui::StrokeKind::Inside);
                }
            }
            UiCmd::Text { pos, text: t, size, color: c, font, align, wrap } => {
                text(&painter, t, rect.min + egui::vec2(pos[0], pos[1]) * s, *size, *c, font, *align, *wrap);
            }
            UiCmd::Image { rect: r, path, tint } => {
                if let Some(id) = image(&ctx, assets, path) {
                    painter.image(id, to(*r), egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)), color(*tint));
                }
            }
            UiCmd::Button { id, rect: r, text: t, style } => {
                let r = to(*r);
                let hot = interactive && pointer.is_some_and(|p| r.contains(p));
                if hot {
                    hovered.push(id.clone());
                    ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
                    if pressed {
                        clicked.push(id.clone());
                    }
                }
                button_frame(r, style, hot);
                let at = match style.align {
                    Align::Left => egui::pos2(r.left() + 12.0 * s, r.center().y - style.size * s * 0.62),
                    Align::Center => egui::pos2(r.center().x, r.center().y - style.size * s * 0.62),
                    Align::Right => egui::pos2(r.right() - 12.0 * s, r.center().y - style.size * s * 0.62),
                };
                text(&painter, t, at, style.size, style.text, &style.font, style.align, 0.0);
            }
            UiCmd::Slider { id, rect: r, value, style } => {
                let r = to(*r);
                let hot = interactive && pointer.is_some_and(|p| r.expand(6.0).contains(p));
                button_frame(r, style, false);
                let fill = egui::Rect::from_min_size(r.min, egui::vec2(r.width() * value.clamp(0.0, 1.0), r.height()));
                painter.rect_filled(fill, style.radius * s, color(style.hover).gamma_multiply(if hot { 1.0 } else { 0.8 }));
                painter.rect_stroke(r, style.radius * s, egui::Stroke::new(1.0, color(style.text).gamma_multiply(0.5)), egui::StrokeKind::Inside);
                if hot {
                    hovered.push(id.clone());
                    if down {
                        if let Some(p) = pointer {
                            changed.push((id.clone(), ((p.x - r.left()) / r.width()).clamp(0.0, 1.0)));
                        }
                    }
                }
            }
        }
    }
    let mouse = pointer.map_or([0.0, 0.0], |p| [(p.x - rect.left()) / s, (p.y - rect.top()) / s]);
    input.canvas.feedback(clicked, hovered, changed, width, mouse);
}
