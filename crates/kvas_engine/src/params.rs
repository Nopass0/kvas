//! Typed script variables. Declare them once with [`script_params!`](crate::script_params): they get
//! defaults, show up as Inspector fields on the attached script, are saved in the scene and can be read or
//! changed by other scripts (`Ctx::var` / `Ctx::set_var`).
//!
//! ```
//! use kvas_engine::prelude::*;
//! script_params! {
//!     /// Spins its object.
//!     pub struct Spinner {
//!         /// Degrees per second around each axis.
//!         speed: Vec3 = Vec3::new(0.0, 90.0, 0.0),
//!         enabled: bool = true,
//!         target: EntityRef = EntityRef::default(),
//!         tint: Color = Color([1.0, 0.5, 0.2, 1.0]),
//!         ticks: Local<u32> = Local::default(),
//!     }
//! }
//! let mut s = Spinner::default();
//! assert_eq!(s.params().len(), 4); // `Local` values are not shown
//! ```

use crate::reflect::{AssetKind, Field};
use glam::Vec3;
use serde::{Deserialize, Serialize};

/// A type that can be a script variable.
pub trait Param {
    /// The Inspector field for this value, or `None` for hidden state.
    fn field<'a>(&'a mut self, name: &'static str) -> Option<Field<'a>>;
}

impl Param for f32 {
    fn field<'a>(&'a mut self, name: &'static str) -> Option<Field<'a>> {
        Some(Field::f32(name, self))
    }
}
impl Param for i32 {
    fn field<'a>(&'a mut self, name: &'static str) -> Option<Field<'a>> {
        Some(Field::i32(name, self))
    }
}
impl Param for u32 {
    fn field<'a>(&'a mut self, name: &'static str) -> Option<Field<'a>> {
        Some(Field::u32(name, self))
    }
}
impl Param for bool {
    fn field<'a>(&'a mut self, name: &'static str) -> Option<Field<'a>> {
        Some(Field::bool(name, self))
    }
}
impl Param for Vec3 {
    fn field<'a>(&'a mut self, name: &'static str) -> Option<Field<'a>> {
        Some(Field::vec3(name, self))
    }
}
impl Param for String {
    fn field<'a>(&'a mut self, name: &'static str) -> Option<Field<'a>> {
        Some(Field::text(name, self))
    }
}
impl<T: crate::reflect::Choice + 'static> Param for T {
    fn field<'a>(&'a mut self, name: &'static str) -> Option<Field<'a>> {
        Some(Field::choice(name, self))
    }
}

/// An RGBA colour variable (shown with a colour picker).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Color(pub [f32; 4]);

impl Default for Color {
    fn default() -> Self {
        Color([1.0; 4])
    }
}
impl Param for Color {
    fn field<'a>(&'a mut self, name: &'static str) -> Option<Field<'a>> {
        Some(Field::color(name, &mut self.0))
    }
}

/// A reference to another object by name (shown as an object picker you can drop objects on).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EntityRef(pub String);

impl EntityRef {
    /// The referenced object, if one with that name exists.
    pub fn resolve(&self, ctx: &crate::script::Ctx) -> Option<kvas_ecs::Entity> {
        (!self.0.is_empty()).then(|| ctx.find(&self.0)).flatten()
    }
}
impl Param for EntityRef {
    fn field<'a>(&'a mut self, name: &'static str) -> Option<Field<'a>> {
        Some(Field::entity(name, &mut self.0))
    }
}

/// A model path, picked from the project's models.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MeshRef(pub String);

impl Param for MeshRef {
    fn field<'a>(&'a mut self, name: &'static str) -> Option<Field<'a>> {
        Some(Field::asset(name, &mut self.0, AssetKind::Mesh))
    }
}

/// A texture path, picked from the project's images.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TextureRef(pub String);

impl Param for TextureRef {
    fn field<'a>(&'a mut self, name: &'static str) -> Option<Field<'a>> {
        Some(Field::asset(name, &mut self.0, AssetKind::Texture))
    }
}

/// A sound clip path, picked from the project's audio files.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SoundRef(pub String);

impl Param for SoundRef {
    fn field<'a>(&'a mut self, name: &'static str) -> Option<Field<'a>> {
        Some(Field::asset(name, &mut self.0, AssetKind::Audio))
    }
}

/// Script state that is not shown in the Inspector or saved.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Local<T>(pub T);

impl<T> Param for Local<T> {
    fn field<'a>(&'a mut self, _name: &'static str) -> Option<Field<'a>> {
        None
    }
}
impl<T> std::ops::Deref for Local<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}
impl<T> std::ops::DerefMut for Local<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

/// Declares a script struct with typed, defaulted variables. Generates `Default` and
/// `params()`, which a script returns from `fields()`:
///
/// ```ignore
/// script_params! { pub struct Door { speed: f32 = 2.0, open: bool = false } }
/// impl Script for Door {
///     fn update(&mut self, ctx: &mut Ctx) { /* self.speed, self.open */ }
///     fn fields(&mut self) -> Vec<Field<'_>> { self.params() }
/// }
/// ```
#[macro_export]
macro_rules! script_params {
    ($(#[$meta:meta])* $vis:vis struct $name:ident { $($(#[$fmeta:meta])* $field:ident : $ty:ty = $default:expr),* $(,)? }) => {
        $(#[$meta])*
        $vis struct $name { $($(#[$fmeta])* pub $field: $ty),* }
        impl Default for $name {
            fn default() -> Self { Self { $($field: $default),* } }
        }
        impl $name {
            /// Inspector fields for every non-`Local` variable.
            pub fn params(&mut self) -> Vec<$crate::reflect::Field<'_>> {
                let mut out = Vec::new();
                $(if let Some(f) = $crate::params::Param::field(&mut self.$field, stringify!($field)) { out.push(f); })*
                out
            }
        }
    };
}

/// Lists the Inspector fields of a hand-written script in one line.
/// Every listed field must implement [`Param`] (numbers, `bool`, `Vec3`, `String`, [`Color`],
/// asset and entity references, enums with `Choice`, and [`Local`] to hide a variable).
///
/// ```
/// use kvas_engine::prelude::*;
///
/// #[derive(Default)]
/// struct Door { speed: f32, open: bool, name: String }
/// impl Script for Door {
///     fn fields(&mut self) -> Vec<Field<'_>> { fields!(self; speed, open, name) }
/// }
/// let mut door = Door::default();
/// assert_eq!(door.fields().len(), 3);
/// ```
#[macro_export]
macro_rules! fields {
    ($s:ident; $($f:ident),* $(,)?) => {{
        #[allow(unused_mut)]
        let mut out: Vec<$crate::reflect::Field<'_>> = Vec::new();
        $(if let Some(field) = $crate::params::Param::field(&mut $s.$f, stringify!($f)) { out.push(field); })*
        out
    }};
}

/// Declares a whole script in one block: the variables with their defaults (shown in the Inspector
/// and saved in the scene) and the callbacks. No `Default`, no `fields()`, no boilerplate.
///
/// Callbacks are `start`, `update` and `fixed_update`; each names `self` and the context.
/// Wrap a variable in [`Local`](crate::params::Local) to keep it out of the Inspector.
///
/// ```
/// use kvas_engine::prelude::*;
///
/// script! {
///     /// Spins around Y and counts the frames it has been running.
///     pub struct Spinner {
///         speed: f32 = 90.0,
///         axis: Vec3 = Vec3::Y,
///         frames: Local<u32> = Local(0),
///     }
///
///     start(self, ctx) {
///         ctx.log(format!("spinner started at speed {}", self.speed));
///     }
///
///     update(self, ctx) {
///         ctx.rotate(self.axis * self.speed * ctx.dt());
///         *self.frames += 1;
///     }
/// }
///
/// let mut spinner = Spinner::default();
/// assert_eq!(spinner.speed, 90.0);
/// assert_eq!(spinner.fields().len(), 2, "Local fields are hidden");
/// ```
#[macro_export]
macro_rules! script {
    (
        $(#[$meta:meta])* $vis:vis struct $name:ident { $($body:tt)* }
        $( $phase:ident ($s:ident, $c:ident) $blk:block )*
    ) => {
        $crate::script_params! { $(#[$meta])* $vis struct $name { $($body)* } }
        impl $crate::script::Script for $name {
            $( $crate::script!(@phase $phase $s $c $blk); )*
            fn fields(&mut self) -> Vec<$crate::reflect::Field<'_>> { self.params() }
        }
    };
    (@phase start $s:ident $c:ident $blk:block) => { fn start(&mut $s, $c: &mut $crate::script::Ctx) $blk };
    (@phase update $s:ident $c:ident $blk:block) => { fn update(&mut $s, $c: &mut $crate::script::Ctx) $blk };
    (@phase fixed_update $s:ident $c:ident $blk:block) => { fn fixed_update(&mut $s, $c: &mut $crate::script::Ctx) $blk };
}
