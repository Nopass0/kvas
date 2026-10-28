//! Frame input state, fed by whichever shell hosts the engine (editor or player).
//! Besides raw keys it supports named *actions* ("Jump") bound to keys / mouse buttons.
//! Bindings come from the project settings and can be changed from scripts at run time.
//!
//! The shell calls [`Input::update`] once before each runtime tick. Pressed and
//! released edges describe differences from the previous shell snapshot; reading
//! an edge does not consume it. [`Input::reset`] handles focus loss.
//!
//! ```
//! use kvas_engine::input::{Input, Key};
//! use glam::Vec2;
//! let mut input = Input::default();
//! input.rebind("Jump", &["Space"]);
//! input.update([Key::Space].into_iter().collect(), Default::default(), Vec2::ZERO, 0.0);
//! assert!(input.action_pressed("Jump"));
//! input.update(Default::default(), Default::default(), Vec2::ZERO, 0.0);
//! assert!(input.action_released("Jump"));
//! ```

use glam::Vec2;
use std::cell::{Cell, RefCell, RefMut};
use std::collections::HashSet;

macro_rules! keys {
    ($($k:ident),+ $(,)?) => {
        /// Physical keyboard keys accepted by the engine shells.
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        pub enum Key { $(#[doc = stringify!($k)] $k),+ }
        impl Key {
            /// Matches the names egui uses for its keys.
            pub fn from_name(name: &str) -> Option<Key> {
                match name { $(stringify!($k) => Some(Key::$k),)+ _ => None }
            }
            /// Canonical case-sensitive binding name, inverse of [`Self::from_name`].
            pub fn name(self) -> &'static str {
                match self { $(Key::$k => stringify!($k),)+ }
            }
        }
    };
}

keys!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, Num0, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8,
    Num9, Space, Enter, Escape, Tab, ArrowLeft, ArrowRight, ArrowUp, ArrowDown, Shift, Ctrl, Alt
);

/// Supported physical pointer buttons.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum MouseButton {
    /// Primary pointer button.
    Left,
    /// Secondary pointer button.
    Right,
    /// Wheel or middle pointer button.
    Middle,
}

/// One physical input an action can be bound to.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Source {
    /// Keyboard source.
    Key(Key),
    /// Pointer source.
    Mouse(MouseButton),
}

impl Source {
    /// "W", "Space", "MouseLeft", ...
    pub fn parse(name: &str) -> Option<Source> {
        match name {
            "MouseLeft" => Some(Source::Mouse(MouseButton::Left)),
            "MouseRight" => Some(Source::Mouse(MouseButton::Right)),
            "MouseMiddle" => Some(Source::Mouse(MouseButton::Middle)),
            _ => Key::from_name(name).map(Source::Key),
        }
    }

    /// Canonical binding string accepted by [`Self::parse`].
    pub fn name(self) -> &'static str {
        match self {
            Source::Key(k) => k.name(),
            Source::Mouse(MouseButton::Left) => "MouseLeft",
            Source::Mouse(MouseButton::Right) => "MouseRight",
            Source::Mouse(MouseButton::Middle) => "MouseMiddle",
        }
    }
}

/// Named action and its alternative physical inputs (logical OR).
#[derive(Clone, Debug)]
pub struct Binding {
    /// Case-sensitive action identifier.
    pub name: String,
    /// Alternative keys or buttons; an empty list disables the action.
    pub sources: Vec<Source>,
}

/// On-screen text a game can show; the shell (editor Play view or player) draws it.
#[derive(Clone, Default, Debug)]
pub struct Hud {
    /// Whether the shell draws a crosshair.
    pub crosshair: bool,
    /// Interaction hint ("Press E to use"); cleared every frame, so set it each frame it applies.
    pub prompt: String,
    /// Short message (pickup, door opened) shown for `message_time` seconds.
    pub message: String,
    /// Remaining message lifetime in seconds.
    pub message_time: f32,
    /// Persistent goal text in the corner.
    pub objective: String,
}

/// Input snapshot, editable action bindings and shell requests.
///
/// Interior mutability permits shared script access to bindings and the HUD.
/// Do not hold a [`Self::hud`] borrow while calling another HUD operation.
#[derive(Default, Clone)]
pub struct Input {
    down: HashSet<Key>,
    pressed: HashSet<Key>,
    released: HashSet<Key>,
    mouse_down: HashSet<MouseButton>,
    mouse_pressed: HashSet<MouseButton>,
    mouse_released: HashSet<MouseButton>,
    /// Pointer movement accumulated by the shell for this frame, in shell units.
    pub mouse_delta: Vec2,
    /// Vertical wheel movement for this frame, in shell units.
    pub scroll: f32,
    // Vec + linear search instead of a map: no hasher state shared with script libraries.
    bindings: RefCell<Vec<Binding>>,
    cursor_lock: Cell<bool>,
    hud: RefCell<Hud>,
    /// A scene change asked for by a script, applied by the host after the frame.
    scene_request: RefCell<Option<SceneRequest>>,
    /// Asset path of the running scene, kept by the host.
    scene_path: RefCell<String>,
}

/// A scene change asked for by a script (see `ScriptCtx::load_scene`).
#[derive(Clone, Debug, PartialEq)]
pub enum SceneRequest {
    /// Replace the running scene with this scene file.
    Load(String),
    /// Add every object of this scene file to the running scene.
    Add(String),
    /// Start the running scene again from its file.
    Reload,
}

impl Input {
    /// Whether a key is held in the current snapshot.
    pub fn key_down(&self, k: Key) -> bool {
        self.down.contains(&k)
    }
    /// True only on the frame the key went down.
    pub fn key_pressed(&self, k: Key) -> bool {
        self.pressed.contains(&k)
    }
    /// True only in the snapshot where a previously held key was released.
    pub fn key_released(&self, k: Key) -> bool {
        self.released.contains(&k)
    }

    /// Whether a pointer button is currently held.
    pub fn mouse_down(&self, b: MouseButton) -> bool {
        self.mouse_down.contains(&b)
    }
    /// True only in the snapshot where a pointer button went down.
    pub fn mouse_pressed(&self, b: MouseButton) -> bool {
        self.mouse_pressed.contains(&b)
    }

    /// True only in the snapshot where a pointer button was released.
    pub fn mouse_released(&self, b: MouseButton) -> bool {
        self.mouse_released.contains(&b)
    }

    /// Scripts ask the shell to hide the cursor and keep it centred (mouse look).
    pub fn lock_cursor(&self, lock: bool) {
        self.cursor_lock.set(lock);
    }
    /// Current cursor-lock request; the hosting shell is responsible for applying it.
    pub fn cursor_locked(&self) -> bool {
        self.cursor_lock.get()
    }
    /// Asks the host to change scenes after this frame. A later request in the same frame wins.
    pub fn request_scene(&self, request: SceneRequest) {
        *self.scene_request.borrow_mut() = Some(request);
    }
    /// Takes the pending scene change, if any (hosts call this after each tick).
    pub fn take_scene_request(&self) -> Option<SceneRequest> {
        self.scene_request.borrow_mut().take()
    }
    /// Asset path of the running scene (empty when the host did not say).
    pub fn scene_path(&self) -> String {
        self.scene_path.borrow().clone()
    }
    /// Records the asset path of the running scene (hosts call this).
    pub fn set_scene_path(&self, path: &str) {
        *self.scene_path.borrow_mut() = path.to_string();
    }
    /// Mutably borrows shell HUD state.
    ///
    /// # Panics
    /// Panics if another HUD borrow is still held.
    pub fn hud(&self) -> RefMut<'_, Hud> {
        self.hud.borrow_mut()
    }
    /// Called by the game loop every frame before scripts run.
    pub fn begin_frame(&self, dt: f32) {
        let mut h = self.hud.borrow_mut();
        h.prompt.clear();
        let dt = if dt.is_finite() { dt.max(0.0) } else { 0.0 };
        h.message_time = (h.message_time - dt).max(0.0);
    }

    /// Axis from two keys: +1 when `plus` is held, -1 when `minus` is held.
    pub fn axis(&self, minus: Key, plus: Key) -> f32 {
        self.key_down(plus) as i32 as f32 - self.key_down(minus) as i32 as f32
    }

    fn source_down(&self, s: Source) -> bool {
        match s {
            Source::Key(k) => self.key_down(k),
            Source::Mouse(b) => self.mouse_down(b),
        }
    }

    fn source_pressed(&self, s: Source) -> bool {
        match s {
            Source::Key(k) => self.key_pressed(k),
            Source::Mouse(b) => self.mouse_pressed(b),
        }
    }

    fn with_action<R>(&self, name: &str, f: impl Fn(&Binding) -> R, default: R) -> R {
        self.bindings.borrow().iter().find(|b| b.name == name).map_or(default, f)
    }

    /// True while any input bound to the action is held.
    pub fn action_down(&self, name: &str) -> bool {
        self.with_action(name, |b| b.sources.iter().any(|&s| self.source_down(s)), false)
    }

    /// True on the frame an input bound to the action went down.
    pub fn action_pressed(&self, name: &str) -> bool {
        self.with_action(name, |b| b.sources.iter().any(|&s| self.source_pressed(s)), false)
    }

    /// True when the last held source of an action is released.
    /// Releasing one alternative while another is held does not release the action.
    /// A press and release of different sources in the same snapshot keeps it active.
    pub fn action_released(&self, name: &str) -> bool {
        self.with_action(
            name,
            |b| {
                !b.sources.iter().any(|&s| self.source_down(s))
                    && b.sources.iter().any(|&s| match s {
                        Source::Key(k) => self.key_released(k),
                        Source::Mouse(button) => self.mouse_released(button),
                    })
            },
            false,
        )
    }

    /// Two-dimensional action vector, clamped to unit length to avoid faster diagonals.
    /// Positive X is `right`, positive Y is `up`; absent actions contribute zero.
    pub fn action_vector(&self, left: &str, right: &str, down: &str, up: &str) -> Vec2 {
        Vec2::new(self.action_axis(left, right), self.action_axis(down, up)).clamp_length_max(1.0)
    }

    /// Whether an action exists, including actions with no sources.
    pub fn has_action(&self, name: &str) -> bool {
        self.with_action(name, |_| true, false)
    }

    /// Removes an action, returning whether it existed.
    pub fn remove_action(&self, name: &str) -> bool {
        let mut bindings = self.bindings.borrow_mut();
        let before = bindings.len();
        bindings.retain(|b| b.name != name);
        before != bindings.len()
    }

    /// Validates all input names before replacing a binding.
    ///
    /// Returns the first invalid name without changing existing bindings.
    /// Unlike [`Self::rebind`], this catches configuration typos.
    pub fn try_rebind(&self, name: &str, sources: &[&str]) -> Result<(), String> {
        for source in sources {
            if Source::parse(source).is_none() {
                return Err(format!("unknown input source '{source}'"));
            }
        }
        self.rebind(name, sources);
        Ok(())
    }

    /// -1..1 from two actions, e.g. `action_axis("MoveLeft", "MoveRight")`.
    pub fn action_axis(&self, negative: &str, positive: &str) -> f32 {
        self.action_down(positive) as i32 as f32 - self.action_down(negative) as i32 as f32
    }

    /// Replaces (or creates) an action's bindings. Unknown names are ignored.
    pub fn rebind(&self, name: &str, sources: &[&str]) {
        let sources: Vec<Source> = sources.iter().filter_map(|s| Source::parse(s)).collect();
        let mut all = self.bindings.borrow_mut();
        match all.iter_mut().find(|b| b.name == name) {
            Some(b) => b.sources = sources,
            None => all.push(Binding { name: name.to_string(), sources }),
        }
    }

    /// Current bindings as `(action, input names)`.
    pub fn bindings(&self) -> Vec<(String, Vec<&'static str>)> {
        self.bindings.borrow().iter().map(|b| (b.name.clone(), b.sources.iter().map(|s| s.name()).collect())).collect()
    }

    /// Replaces all actions. Duplicate action names use the last definition;
    /// unknown physical input names are ignored as in [`Self::rebind`].
    pub fn set_bindings<S: AsRef<str>>(&self, defs: &[(String, Vec<S>)]) {
        self.bindings.borrow_mut().clear();
        for (name, sources) in defs {
            let names: Vec<&str> = sources.iter().map(|s| s.as_ref()).collect();
            self.rebind(name, &names);
        }
    }

    /// Call once per frame with the keys/buttons that are currently held.
    pub fn update(&mut self, keys: HashSet<Key>, buttons: HashSet<MouseButton>, mouse_delta: Vec2, scroll: f32) {
        self.pressed = keys.difference(&self.down).copied().collect();
        self.released = self.down.difference(&keys).copied().collect();
        self.down = keys;
        self.mouse_pressed = buttons.difference(&self.mouse_down).copied().collect();
        self.mouse_released = self.mouse_down.difference(&buttons).copied().collect();
        self.mouse_down = buttons;
        self.mouse_delta = mouse_delta;
        self.scroll = scroll;
    }

    /// Clears held inputs, frame edges and pointer motion, and releases cursor lock.
    /// Bindings and HUD content are preserved. No synthetic release edges are emitted.
    pub fn reset(&mut self) {
        self.down.clear();
        self.pressed.clear();
        self.released.clear();
        self.mouse_down.clear();
        self.mouse_pressed.clear();
        self.mouse_released.clear();
        self.mouse_delta = Vec2::ZERO;
        self.scroll = 0.0;
        self.cursor_lock.set(false);
    }
}
