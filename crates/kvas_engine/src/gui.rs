//! Serializable screen/world UI components. Native shells paint them and publish button signals.
use crate::{components::Component, reflect::Field};
use glam::Vec3;
use serde::{Deserialize, Serialize};

crate::choice_enum!(
    /// Element drawn by the native game UI shell.
    UiKind { Label, Button, Progress } default Label
);

/// Camera-overlay or world-anchored interface element. Coordinates use logical pixels.
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UiElement {
    /// Visual/interaction kind.
    pub kind: UiKind,
    /// Text shown above the background.
    pub text: String,
    /// Normalized screen anchor X/Y, with Z used as draw order.
    pub anchor: Vec3,
    /// Width and height in pixels; Z is unused.
    pub size: Vec3,
    /// Pixel offset from anchor (X/Y); Z is unused.
    pub offset: Vec3,
    /// Text size in logical pixels.
    pub font_size: f32,
    /// Background RGBA.
    pub background: [f32; 4],
    /// Text / progress-fill RGBA.
    pub foreground: [f32; 4],
    /// Whether to draw the element.
    pub visible: bool,
    /// Whether button interaction is enabled.
    pub interactable: bool,
    /// Position the element at the object's projected world transform.
    pub world_space: bool,
    /// Signal channel broadcast on button click; source entity is the sender.
    pub signal: String,
    /// Progress fraction (0..1).
    pub value: f32,
}
impl Default for UiElement {
    fn default() -> Self {
        Self { kind: UiKind::Label, text: "UI element".into(), anchor: Vec3::new(0.5, 0.1, 0.0), size: Vec3::new(240.0, 42.0, 0.0), offset: Vec3::ZERO, font_size: 18.0, background: [0.06, 0.08, 0.12, 0.88], foreground: [0.9, 0.95, 1.0, 1.0], visible: true, interactable: true, world_space: false, signal: "ui.click".into(), value: 0.5 }
    }
}
impl Component for UiElement {
    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![Field::choice("kind", &mut self.kind), Field::text("text", &mut self.text), Field::vec3("anchor", &mut self.anchor), Field::vec3("size", &mut self.size), Field::vec3("offset", &mut self.offset), Field::f32("font_size", &mut self.font_size), Field::color("background", &mut self.background), Field::color("foreground", &mut self.foreground), Field::bool("visible", &mut self.visible), Field::bool("interactable", &mut self.interactable), Field::bool("world_space", &mut self.world_space), Field::text("signal", &mut self.signal), Field::f32("value", &mut self.value)]
    }
}
