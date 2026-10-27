//! Glue between egui (the window/input layer for both editor and player) and the engine.

use eframe::egui;
use glam::Vec2;
use kvas_engine::input::{Hud, Input, Key, MouseButton};
use std::collections::HashSet;

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
