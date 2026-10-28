//! Lucide icons (ISC license, https://lucide.dev), embedded as SVG and tinted at draw time.

use eframe::egui::{self, Color32, Vec2};

macro_rules! icons {
    ($($name:ident => $file:literal),+ $(,)?) => {
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        #[expect(dead_code, reason = "The embedded icon catalog intentionally includes assets reserved for editor panels")]
        pub enum Icon { $($name),+ }
        impl Icon {
            fn source(self) -> (&'static str, &'static [u8]) {
                match self { $(Icon::$name => (concat!("bytes://lucide-", $file, ".svg"), include_bytes!(concat!("../icons/", $file, ".svg")))),+ }
            }
        }
    };
}

icons! {
    ArrowLeft => "arrow-left",
    Box => "box",
    Boxes => "boxes",
    Camera => "camera",
    ChevronDown => "chevron-down",
    ChevronRight => "chevron-right",
    ChevronUp => "chevron-up",
    Circle => "circle",
    Copy => "copy",
    Cuboid => "cuboid",
    Eye => "eye",
    File => "file",
    FileBox => "file-box",
    FileCode => "file-code",
    FileJson => "file-json",
    Folder => "folder",
    FolderClosed => "folder-closed",
    FolderOpen => "folder-open",
    FolderPlus => "folder-plus",
    Gamepad2 => "gamepad-2",
    Grid3x3 => "grid-3x3",
    Hammer => "hammer",
    Home => "home",
    Image => "image",
    Layers => "layers",
    LayoutGrid => "layout-grid",
    Lightbulb => "lightbulb",
    Maximize => "maximize",
    Minus => "minus",
    Move3d => "move-3d",
    Music => "music",
    Package => "package",
    Pause => "pause",
    Play => "play",
    Plus => "plus",
    Redo2 => "redo-2",
    Rotate3d => "rotate-3d",
    RotateCcw => "rotate-ccw",
    Save => "save",
    Scale3d => "scale-3d",
    Scan => "scan",
    Search => "search",
    Settings => "settings",
    Shapes => "shapes",
    SkipForward => "skip-forward",
    Square => "square",
    Sun => "sun",
    Terminal => "terminal",
    Trash2 => "trash-2",
    Undo2 => "undo-2",
    Upload => "upload",
    X => "x",
    Brush => "brush",
    Magnet => "magnet",
    Flip => "flip",
    RotateCw => "rotate-cw",
    Target => "target",
    Drop => "drop",
    VertexSnap => "vertex",
    SurfaceSnap => "surface",
    Aperture => "aperture",
    Bell => "bell",
    Capsule => "capsule",
    Cloud => "cloud",
    Cone => "cone",
    Cylinder => "cylinder",
    FilePlus => "file-plus",
    Flame => "flame",
    Link => "link",
    Mountain => "mountain",
    Palette => "palette",
    Plane => "plane",
    Rock => "rock",
    Shirt => "shirt",
    Sliders => "sliders",
    Sparkles => "sparkles",
    Sphere => "sphere",
    Tag => "tag",
    Torus => "torus",
    Volume => "volume",
    Waves => "waves",
    Wind => "wind",
    Keyboard => "keyboard",
}

pub fn install(ctx: &egui::Context) {
    egui_extras::install_image_loaders(ctx);
}

impl Icon {
    pub fn image(self, size: f32, tint: Color32) -> egui::Image<'static> {
        let (uri, bytes) = self.source();
        egui::Image::from_bytes(uri, bytes).fit_to_exact_size(Vec2::splat(size)).tint(tint)
    }
}

/// Draws an icon inline.
pub fn show(ui: &mut egui::Ui, icon: Icon, size: f32) -> egui::Response {
    let tint = crate::theme::pal().text_dim;
    ui.add(icon.image(size, tint))
}

/// A square icon-only button with a hover tooltip. `active` highlights it (toggles, current tool).
/// Hover and active states fade in and out.
pub fn button(ui: &mut egui::Ui, icon: Icon, tip: &str, active: bool) -> egui::Response {
    let p = crate::theme::pal();
    let size = 26.0;
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(size), egui::Sense::click());
    let hover = crate::ui_kit::animate(ui, resp.id.with("hover"), resp.hovered());
    let on = crate::ui_kit::animate(ui, resp.id.with("active"), active);
    let fill = crate::ui_kit::mix(crate::ui_kit::mix(Color32::TRANSPARENT, p.hover, hover), p.selection, on);
    ui.painter().rect_filled(rect, 6.0, fill);
    let idle = if crate::theme::is_light() { Color32::from_rgb(0x5A, 0x5D, 0x63) } else { Color32::from_rgb(0xB4, 0xB8, 0xBF) };
    let tint = crate::ui_kit::mix(crate::ui_kit::mix(idle, p.text, hover), crate::theme::ACCENT, on);
    let s = 16.0;
    icon.image(s, tint).paint_at(ui, egui::Rect::from_center_size(rect.center(), Vec2::splat(s)));
    resp.on_hover_cursor(egui::CursorIcon::PointingHand).on_hover_text(tip)
}

/// The Kvas logo (vector, tinted by nothing), for title bars and the hub.
pub fn logo(size: f32) -> egui::Image<'static> {
    egui::Image::from_bytes("bytes://kvas-logo.svg", include_bytes!("../assets/logo.svg").as_slice()).fit_to_exact_size(Vec2::splat(size))
}

/// The window / taskbar icon.
pub fn window_icon() -> Option<egui::IconData> {
    let img = image::load_from_memory(include_bytes!("../assets/logo-256.png")).ok()?.to_rgba8();
    Some(egui::IconData { width: img.width(), height: img.height(), rgba: img.into_raw() })
}
