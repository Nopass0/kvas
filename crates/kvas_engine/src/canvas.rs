//! Immediate-mode game interface for scripts: every frame a script describes what is on screen
//! (panels, text in any font, images, buttons, sliders, toggles) and the shell paints it over the
//! game. Clicks and slider changes come back on the next frame, so menus are plain `if`s:
//!
//! ```
//! use kvas_engine::canvas::{Canvas, Align};
//! let ui = Canvas::default();
//! ui.panel([0.0, 0.0, 1280.0, 720.0], [0.0, 0.0, 0.0, 0.6], 0.0);
//! ui.text("LAST SERVICE", [640.0, 200.0], 64.0, [1.0; 4], "fonts/terminal.ttf", Align::Center);
//! if ui.button("new", [540.0, 360.0, 200.0, 44.0], "NEW SHIFT") { /* start */ }
//! assert_eq!(ui.commands().len(), 3);
//! ```
//!
//! Coordinates are in a virtual screen 720 units high whose width follows the window's aspect
//! ratio (`ctx.ui_width()`); `[x, y, w, h]` rectangles start at the top-left corner.

use serde::{Deserialize, Serialize};
use std::cell::{Cell, RefCell};

/// Height of the virtual interface screen.
pub const UI_HEIGHT: f32 = 720.0;

/// Horizontal text alignment.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Align {
    #[default]
    Left,
    Center,
    Right,
}

/// One drawing command of the interface.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum UiCmd {
    /// Filled rounded rectangle, optional outline.
    Panel { rect: [f32; 4], color: [f32; 4], radius: f32, outline: [f32; 4] },
    /// Text at a point (the point is the top of the line; alignment around x).
    Text { pos: [f32; 2], text: String, size: f32, color: [f32; 4], font: String, align: Align, wrap: f32 },
    /// An image from the assets, tinted.
    Image { rect: [f32; 4], path: String, tint: [f32; 4] },
    /// Clickable button.
    Button { id: String, rect: [f32; 4], text: String, style: ButtonStyle },
    /// Horizontal slider for a 0..1 value.
    Slider { id: String, rect: [f32; 4], value: f32, style: ButtonStyle },
}

/// Look of buttons and sliders.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ButtonStyle {
    pub font: String,
    pub size: f32,
    pub text: [f32; 4],
    pub background: [f32; 4],
    pub hover: [f32; 4],
    pub outline: [f32; 4],
    pub radius: f32,
    pub align: Align,
}

impl Default for ButtonStyle {
    fn default() -> Self {
        Self { font: String::new(), size: 22.0, text: [0.92, 0.9, 0.85, 1.0], background: [0.0, 0.0, 0.0, 0.0], hover: [0.85, 0.12, 0.1, 0.35], outline: [0.0; 4], radius: 4.0, align: Align::Center }
    }
}

/// The interface of one frame plus what the player did with the previous one.
#[derive(Default, Clone)]
pub struct Canvas {
    cmds: RefCell<Vec<UiCmd>>,
    clicked: RefCell<Vec<String>>,
    hovered: RefCell<Vec<String>>,
    /// Slider values changed by the player: (id, new value).
    changed: RefCell<Vec<(String, f32)>>,
    width: Cell<f32>,
    style: RefCell<ButtonStyle>,
    /// Mouse position in interface units (when the cursor is free).
    mouse: Cell<[f32; 2]>,
    /// True while the shell shows any button, so the host can free the mouse.
    wants_mouse: Cell<bool>,
}

impl Canvas {
    /// Width of the virtual screen (720 high, width follows the window).
    pub fn width(&self) -> f32 {
        let w = self.width.get();
        if w > 0.0 { w } else { UI_HEIGHT * 16.0 / 9.0 }
    }
    /// Sets the look of the following buttons and sliders.
    pub fn set_style(&self, style: ButtonStyle) {
        *self.style.borrow_mut() = style;
    }
    /// The current button style.
    pub fn style(&self) -> ButtonStyle {
        self.style.borrow().clone()
    }
    /// A filled panel.
    pub fn panel(&self, rect: [f32; 4], color: [f32; 4], radius: f32) {
        self.cmds.borrow_mut().push(UiCmd::Panel { rect, color, radius, outline: [0.0; 4] });
    }
    /// A panel with an outline.
    pub fn frame(&self, rect: [f32; 4], color: [f32; 4], outline: [f32; 4], radius: f32) {
        self.cmds.borrow_mut().push(UiCmd::Panel { rect, color, radius, outline });
    }
    /// One line (or a wrapped paragraph when `wrap` > 0 is given via [`Self::paragraph`]) of text.
    pub fn text(&self, text: &str, pos: [f32; 2], size: f32, color: [f32; 4], font: &str, align: Align) {
        self.cmds.borrow_mut().push(UiCmd::Text { pos, text: text.to_string(), size, color, font: font.to_string(), align, wrap: 0.0 });
    }
    /// Text wrapped to `width` units.
    pub fn paragraph(&self, text: &str, pos: [f32; 2], width: f32, size: f32, color: [f32; 4], font: &str) {
        self.cmds.borrow_mut().push(UiCmd::Text { pos, text: text.to_string(), size, color, font: font.to_string(), align: Align::Left, wrap: width });
    }
    /// An image (PNG/JPG from the assets).
    pub fn image(&self, path: &str, rect: [f32; 4], tint: [f32; 4]) {
        self.cmds.borrow_mut().push(UiCmd::Image { rect, path: path.to_string(), tint });
    }
    /// A button; returns true on the frame after it was clicked.
    pub fn button(&self, id: &str, rect: [f32; 4], text: &str) -> bool {
        let style = self.style.borrow().clone();
        self.cmds.borrow_mut().push(UiCmd::Button { id: id.to_string(), rect, text: text.to_string(), style });
        self.wants_mouse.set(true);
        self.clicked.borrow().iter().any(|c| c == id)
    }
    /// Whether the pointer was over the button last frame.
    pub fn hovered(&self, id: &str) -> bool {
        self.hovered.borrow().iter().any(|c| c == id)
    }
    /// A slider showing `value` (0..1); returns the value after the player's change.
    pub fn slider(&self, id: &str, rect: [f32; 4], value: f32) -> f32 {
        let style = self.style.borrow().clone();
        let v = self.changed.borrow().iter().rev().find(|(i, _)| i == id).map_or(value, |(_, v)| *v);
        self.cmds.borrow_mut().push(UiCmd::Slider { id: id.to_string(), rect, value: v.clamp(0.0, 1.0), style });
        self.wants_mouse.set(true);
        v.clamp(0.0, 1.0)
    }
    /// A toggle drawn as a button with ON / OFF; returns the new state.
    pub fn toggle(&self, id: &str, rect: [f32; 4], label: &str, on: bool) -> bool {
        let clicked = self.button(id, rect, &format!("{label}: {}", if on { "ON" } else { "OFF" }));
        on ^ clicked
    }
    /// Mouse position in interface units.
    pub fn mouse(&self) -> [f32; 2] {
        self.mouse.get()
    }

    // ---- host side
    /// Commands of the current frame.
    pub fn commands(&self) -> Vec<UiCmd> {
        self.cmds.borrow().clone()
    }
    /// Whether any interactive element is on screen (the host then shows the cursor).
    pub fn wants_mouse(&self) -> bool {
        self.wants_mouse.get()
    }
    /// Starts a new frame of commands (called by the runtime before scripts update).
    pub fn begin(&self) {
        self.cmds.borrow_mut().clear();
        self.wants_mouse.set(false);
    }
    /// What the player did with the painted frame (called by the shell after painting).
    pub fn feedback(&self, clicked: Vec<String>, hovered: Vec<String>, changed: Vec<(String, f32)>, width: f32, mouse: [f32; 2]) {
        *self.clicked.borrow_mut() = clicked;
        *self.hovered.borrow_mut() = hovered;
        *self.changed.borrow_mut() = changed;
        self.width.set(width);
        self.mouse.set(mouse);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buttons_report_clicks_one_frame_later() {
        let c = Canvas::default();
        assert!(!c.button("a", [0.0; 4], "A"));
        c.feedback(vec!["a".into()], vec!["a".into()], vec![("vol".into(), 0.3)], 1280.0, [1.0, 2.0]);
        c.begin();
        assert!(c.button("a", [0.0; 4], "A"));
        assert!(c.hovered("a"));
        assert_eq!(c.slider("vol", [0.0; 4], 0.8), 0.3);
        assert!(c.toggle("t", [0.0; 4], "VHS", true));
        assert_eq!(c.width(), 1280.0);
        assert!(c.wants_mouse());
    }
}
