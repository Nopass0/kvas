//! Glue between egui (the window/input layer for both editor and player) and the engine.

use eframe::egui;
use glam::Vec2;
use kvas_engine::input::{Hud, Input, Key, MouseButton};
use std::collections::HashSet;

/// Bounds of a GUI component in a viewport, or `None` when hidden/invalid/behind the camera.
pub fn ui_bounds(scene: &kvas_engine::scene::Scene, entity: kvas_ecs::Entity, element: &kvas_engine::gui::UiElement, viewport: egui::Rect, view: Option<&crate::View>) -> Option<egui::Rect> {
    if !element.visible || !element.anchor.is_finite() || !element.offset.is_finite() || !element.size.is_finite() || element.size.x <= 0.0 || element.size.y <= 0.0 || !kvas_engine::hierarchy::is_visible(&scene.world, entity) { return None; }
    let anchor = if element.world_space {
        let position = kvas_engine::hierarchy::global_matrix(&scene.world, entity).w_axis.truncate();
        let ndc = view?.project_visible(position)?;
        viewport.min + egui::vec2((ndc.x * 0.5 + 0.5) * viewport.width(), (0.5 - ndc.y * 0.5) * viewport.height())
    } else { viewport.min + egui::vec2(element.anchor.x * viewport.width(), element.anchor.y * viewport.height()) };
    Some(egui::Rect::from_center_size(anchor + egui::vec2(element.offset.x, element.offset.y), egui::vec2(element.size.x.min(8192.0), element.size.y.min(8192.0))))
}

/// Draws screen UI and world-anchored labels; returns clicked `(entity, signal channel)` pairs.
/// The host should send these through `Scene.signals` for delivery on the next simulation tick.
pub fn draw_game_ui(ui: &mut egui::Ui, scene: &kvas_engine::scene::Scene, viewport: egui::Rect, view: Option<&crate::View>, interactive: bool) -> Vec<(kvas_ecs::Entity, String)> {
    use kvas_engine::gui::UiKind;
    let mut elements: Vec<_> = scene.world.iter::<kvas_engine::gui::UiElement>().collect();
    elements.sort_by(|(ea, a), (eb, b)| a.anchor.z.total_cmp(&b.anchor.z).then(ea.cmp(eb)));
    let color = |rgba: [f32; 4]| egui::Color32::from_rgba_unmultiplied((rgba[0].clamp(0.0, 1.0) * 255.0) as u8, (rgba[1].clamp(0.0, 1.0) * 255.0) as u8, (rgba[2].clamp(0.0, 1.0) * 255.0) as u8, (rgba[3].clamp(0.0, 1.0) * 255.0) as u8);
    let painter = ui.painter().with_clip_rect(viewport);
    let mut actions = Vec::new();
    for (entity, element) in elements {
        let Some(rect) = ui_bounds(scene, entity, element, viewport, view) else { continue };
        let is_button = element.kind == UiKind::Button && element.interactable && interactive;
        let response = ui.interact(rect.intersect(viewport), egui::Id::unique(("game-ui-element", entity.to_bits())), if is_button { egui::Sense::click() } else { egui::Sense::hover() });
        let mut background = color(element.background);
        if is_button && response.hovered() { background = background.gamma_multiply(1.25); }
        painter.rect_filled(rect, 7.0, background);
        if element.kind == UiKind::Progress {
            let progress = egui::Rect::from_min_size(rect.min, egui::vec2(rect.width() * element.value.clamp(0.0, 1.0), rect.height()));
            painter.rect_filled(progress, 7.0, color(element.foreground).gamma_multiply(0.7));
        }
        let text = painter.layout(element.text.clone(), egui::FontId::proportional(element.font_size.clamp(6.0, 200.0)), color(element.foreground), (rect.width() - 12.0).max(1.0));
        painter.galley(rect.center() - text.size() * 0.5, text, color(element.foreground));
        if is_button { response.clone().on_hover_cursor(egui::CursorIcon::PointingHand); }
        if is_button && response.clicked() && !element.signal.trim().is_empty() { actions.push((entity, element.signal.clone())); }
    }
    actions
}

/// Feeds the current egui frame into the engine's `Input`. While a script has locked the cursor the
/// mouse delta is the distance from `center`, where `apply_cursor` keeps the pointer.
pub fn feed_input(input: &mut Input, i: &egui::InputState, center: Option<egui::Pos2>) {
    if !i.focused {
        input.reset();
        return;
    }
    let mut keys: HashSet<Key> = i.keys_down.iter().filter_map(|k| Key::from_name(k.name())).collect();
    if i.modifiers.shift {
        keys.insert(Key::Shift);
    }
    if i.modifiers.ctrl {
        keys.insert(Key::Ctrl);
    }
    if i.modifiers.alt {
        keys.insert(Key::Alt);
    }
    let mut buttons = HashSet::new();
    for (button, ours) in [
        (egui::PointerButton::Primary, MouseButton::Left),
        (egui::PointerButton::Secondary, MouseButton::Right),
        (egui::PointerButton::Middle, MouseButton::Middle),
    ] {
        if i.pointer.button_down(button) {
            buttons.insert(ours);
        }
    }
    // The pointer needs a few frames to reach the centre after locking; ignore that jump.
    static LOCKED_FRAMES: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let age = if input.cursor_locked() { LOCKED_FRAMES.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1 } else { LOCKED_FRAMES.store(0, std::sync::atomic::Ordering::Relaxed); 0 };
    let d = match (input.cursor_locked() && age > 4, center, i.pointer.latest_pos()) {
        (true, Some(c), Some(p)) => (p - c).clamp(egui::vec2(-300.0, -300.0), egui::vec2(300.0, 300.0)),
        _ if input.cursor_locked() => egui::Vec2::ZERO,
        _ => i.pointer.delta(),
    };
    input.update(keys, buttons, Vec2::new(d.x, d.y), i.smooth_scroll_delta.y);
}

/// Who holds the mouse while a game runs: it is captured (hidden and centred) when the game
/// starts, Escape frees it, and a click on the game view captures it again. Scripts can still
/// lock or free it themselves with `lock_cursor`.
#[derive(Default)]
pub struct MouseCapture {
    started: bool,
    freed: bool,
}

impl MouseCapture {
    /// Call once per frame before `feed_input`; `over_view` is true while the pointer is over the game.
    pub fn update(&mut self, ctx: &egui::Context, input: &Input, over_view: bool) {
        if !self.started {
            self.started = true;
            input.lock_cursor(true);
        }
        let (escape, click, focused) = ctx.input(|i| (i.key_pressed(egui::Key::Escape), i.pointer.primary_pressed(), i.focused));
        if !focused && input.cursor_locked() {
            input.lock_cursor(false);
            self.freed = true;
        }
        if escape && input.cursor_locked() {
            input.lock_cursor(false);
            self.freed = true;
        } else if click && over_view && self.freed && !input.cursor_locked() {
            input.lock_cursor(true);
            self.freed = false;
        }
    }

    /// The player pressed Escape (or the window lost focus) and has not clicked back yet.
    pub fn freed(&self) -> bool {
        self.freed
    }
}

/// Small hint at the bottom of the game view about the mouse capture.
pub fn draw_capture_hint(painter: &egui::Painter, rect: egui::Rect, text: &str) {
    let galley = painter.layout_no_wrap(text.to_string(), egui::FontId::proportional(13.0), egui::Color32::from_white_alpha(235));
    let pill = egui::Rect::from_center_size(egui::pos2(rect.center().x, rect.bottom() - 30.0), galley.size() + egui::vec2(24.0, 12.0));
    painter.rect_filled(pill, 8.0, egui::Color32::from_black_alpha(150));
    painter.galley(pill.center() - galley.size() * 0.5, galley, egui::Color32::WHITE);
}

/// Hides the cursor and re-centres it while the game has it locked; shows it again otherwise.
pub fn apply_cursor(ctx: &egui::Context, input: &Input, center: egui::Pos2) {
    let locked = input.cursor_locked();
    ctx.send_viewport_cmd(egui::ViewportCommand::CursorVisible(!locked));
    if locked {
        ctx.send_viewport_cmd(egui::ViewportCommand::CursorPosition(center));
    }
}

/// Crosshair, interaction prompt, message and objective of the game's `Hud`.
pub fn draw_hud(painter: &egui::Painter, rect: egui::Rect, hud: &Hud) {
    let white = egui::Color32::from_white_alpha(235);
    let shadow = egui::Color32::from_black_alpha(170);
    let pill = |pos: egui::Pos2, text: &str, size: f32, align: egui::Align2| {
        let galley = painter.layout_no_wrap(text.to_string(), egui::FontId::proportional(size), white);
        let box_rect = align.anchor_size(pos, galley.size() + egui::vec2(26.0, 14.0));
        painter.rect_filled(box_rect, 9.0, egui::Color32::from_black_alpha(125));
        painter.rect_stroke(box_rect, 9.0, egui::Stroke::new(1.0, egui::Color32::from_white_alpha(38)), egui::StrokeKind::Inside);
        painter.galley(box_rect.center() - galley.size() * 0.5, galley, white);
    };
    let c = rect.center();
    if hud.crosshair {
        let (inner, outer) = (if hud.prompt.is_empty() { 3.0 } else { 6.0 }, if hud.prompt.is_empty() { 4.5 } else { 8.0 });
        painter.circle_stroke(c, outer, egui::Stroke::new(3.0, shadow));
        painter.circle_stroke(c, inner, egui::Stroke::new(1.6, white));
    }
    if !hud.prompt.is_empty() {
        pill(c + egui::vec2(0.0, 44.0), &hud.prompt, 17.0, egui::Align2::CENTER_CENTER);
    }
    if hud.message_time > 0.0 && !hud.message.is_empty() {
        pill(egui::pos2(c.x, rect.min.y + 70.0), &hud.message, 19.0, egui::Align2::CENTER_CENTER);
    }
    if !hud.objective.is_empty() {
        pill(rect.min + egui::vec2(18.0, 64.0), &hud.objective, 15.0, egui::Align2::LEFT_TOP);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lost_focus_clears_held_input_and_releases_cursor_request() {
        let mut input = Input::default();
        input.update([Key::W].into_iter().collect(), Default::default(), Vec2::ONE, 1.0);
        input.lock_cursor(true);
        let mut state = egui::InputState::default();
        state.focused = false;
        feed_input(&mut input, &state, None);
        assert!(!input.key_down(Key::W));
        assert!(!input.cursor_locked());
        assert_eq!(input.mouse_delta, Vec2::ZERO);
        assert_eq!(input.scroll, 0.0);
    }
}
