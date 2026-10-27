//! Tiny reflection layer. A component (or script) lists its editable fields as `Field`s;
//! the editor draws them and the scene/script layer converts them to and from JSON.
//! One mechanism, no per-type UI or serialization code.

use glam::Vec3;
use serde_json::{json, Value};

/// Asset picker filter used by the Inspector.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AssetKind {
    /// Geometry asset (OBJ, FBX, glTF or GLB).
    Mesh,
    /// Image asset or procedural texture.
    Texture,
    /// WGSL shader asset.
    Shader,
    /// Sound clip (wav, ogg, mp3 or flac).
    Audio,
}

/// Conversion between enum variants and Inspector dropdown indices.
pub trait Choice: Copy {
    /// Ordered labels matching the enum's declaration order.
    const OPTIONS: &'static [&'static str];
    /// Index of this variant in [`Self::OPTIONS`].
    fn to_index(self) -> usize;
    /// Variant at an index; [`crate::choice_enum!`] clamps out-of-range indices to the last variant.
    fn from_index(i: usize) -> Self;
}

/// Declares a serde enum that can be picked from a dropdown.
#[macro_export]
macro_rules! choice_enum {
    ($(#[$m:meta])* $name:ident { $($v:ident),+ $(,)? } default $d:ident) => {
        $(#[$m])*
        #[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
        pub enum $name { $(#[doc = stringify!($v)] $v),+ }
        impl Default for $name { fn default() -> Self { $name::$d } }
        impl $crate::reflect::Choice for $name {
            const OPTIONS: &'static [&'static str] = &[$(stringify!($v)),+];
            fn to_index(self) -> usize { self as usize }
            fn from_index(i: usize) -> Self {
                const ALL: &[$name] = &[$($name::$v),+];
                ALL[i.min(ALL.len() - 1)]
            }
        }
    };
}

/// Mutable borrowed value described by a [`Field`].
/// Descriptors must be dropped before borrowing the original value again.
pub enum FieldRef<'a> {
    /// Finite single-precision number.
    F32(&'a mut f32),
    /// Signed 32-bit integer.
    I32(&'a mut i32),
    /// Unsigned 32-bit integer, including bit masks and deterministic seeds.
    U32(&'a mut u32),
    /// Boolean checkbox.
    Bool(&'a mut bool),
    /// Three-component vector encoded as a JSON array.
    Vec3(&'a mut Vec3),
    /// RGBA colour encoded as four numbers.
    Color(&'a mut [f32; 4]),
    /// Editable UTF-8 string.
    Text(&'a mut String),
    /// Asset-relative string with a picker category.
    Asset(&'a mut String, AssetKind),
    /// Name of another entity in the scene.
    Entity(&'a mut String),
    /// Dropdown state and callback to write a selected variant.
    Choice {
        /// Ordered display labels.
        options: &'static [&'static str],
        /// Current selection; must index `options`.
        index: usize,
        /// Writes a selection into the borrowed original enum.
        set: Box<dyn FnMut(usize) + 'a>,
    },
}

/// Named editable property shared by Inspector widgets and scene script properties.
///
/// ```
/// use kvas_engine::reflect::Field;
/// let mut speed = 2.0;
/// { let mut field = Field::f32("speed", &mut speed);
///   field.set_json(&serde_json::json!(3.0)); }
/// assert_eq!(speed, 3.0);
/// ```
pub struct Field<'a> {
    /// Stable property key used in scene JSON, undo and the Inspector.
    pub name: &'static str,
    /// Borrowed value and editor presentation kind.
    pub value: FieldRef<'a>,
}

impl<'a> Field<'a> {
    /// Borrows a floating-point property under a stable field name.
    pub fn f32(name: &'static str, v: &'a mut f32) -> Self {
        Self { name, value: FieldRef::F32(v) }
    }
    /// Borrows a signed integer property.
    pub fn i32(name: &'static str, v: &'a mut i32) -> Self {
        Self { name, value: FieldRef::I32(v) }
    }
    /// Borrows an unsigned integer property without narrowing its range to i32.
    pub fn u32(name: &'static str, value: &'a mut u32) -> Self {
        Self { name, value: FieldRef::U32(value) }
    }
    /// Borrows a boolean checkbox property.
    pub fn bool(name: &'static str, v: &'a mut bool) -> Self {
        Self { name, value: FieldRef::Bool(v) }
    }
    /// Borrows a three-component vector property.
    pub fn vec3(name: &'static str, v: &'a mut Vec3) -> Self {
        Self { name, value: FieldRef::Vec3(v) }
    }
    /// Borrows an RGBA colour property.
    pub fn color(name: &'static str, v: &'a mut [f32; 4]) -> Self {
        Self { name, value: FieldRef::Color(v) }
    }
    /// Borrows an editable string property.
    pub fn text(name: &'static str, v: &'a mut String) -> Self {
        Self { name, value: FieldRef::Text(v) }
    }
    /// Borrows an asset path with the specified picker category.
    pub fn asset(name: &'static str, v: &'a mut String, kind: AssetKind) -> Self {
        Self { name, value: FieldRef::Asset(v, kind) }
    }
    /// Borrows an entity-name reference; names are resolved by gameplay code.
    pub fn entity(name: &'static str, v: &'a mut String) -> Self {
        Self { name, value: FieldRef::Entity(v) }
    }
    /// Borrows an enum as a dropdown using its Choice implementation.
    pub fn choice<T: Choice + 'a>(name: &'static str, v: &'a mut T) -> Self {
        let index = v.to_index();
        Self { name, value: FieldRef::Choice { options: T::OPTIONS, index, set: Box::new(move |i| *v = T::from_index(i)) } }
    }

    /// Copies the current reflected value into JSON.
    /// Non-finite numeric values serialize as null through serde_json.
    pub fn to_json(&self) -> Value {
        match &self.value {
            FieldRef::F32(v) => json!(**v),
            FieldRef::I32(v) => json!(**v),
            FieldRef::U32(v) => json!(**v),
            FieldRef::Bool(v) => json!(**v),
            FieldRef::Vec3(v) => json!(v.to_array()),
            FieldRef::Color(v) => json!(**v),
            FieldRef::Text(v) | FieldRef::Asset(v, _) | FieldRef::Entity(v) => json!(v),
            FieldRef::Choice { options, index, .. } => json!(options.get(*index)),
        }
    }

    /// Applies a JSON value; values of the wrong shape are ignored.
    pub fn set_json(&mut self, value: &Value) {
        match &mut self.value {
            FieldRef::F32(v) => {
                if let Some(x) = value.as_f64() {
                    if x.is_finite() && (x as f32).is_finite() {
                        **v = x as f32
                    }
                }
            }
            FieldRef::I32(v) => {
                if let Some(x) = value.as_i64() {
                    if let Ok(x) = i32::try_from(x) {
                        **v = x
                    }
                }
            }
            FieldRef::U32(v) => {
                if let Some(x) = value.as_u64().and_then(|x| u32::try_from(x).ok()) {
                    **v = x;
                }
            }
            FieldRef::Bool(v) => {
                if let Some(x) = value.as_bool() {
                    **v = x
                }
            }
            FieldRef::Vec3(v) => {
                if let Some(a) = floats::<3>(value) {
                    **v = Vec3::from_array(a)
                }
            }
            FieldRef::Color(v) => {
                if let Some(a) = floats::<4>(value) {
                    **v = a
                }
            }
            FieldRef::Text(v) | FieldRef::Asset(v, _) | FieldRef::Entity(v) => {
                if let Some(s) = value.as_str() {
                    **v = s.to_string()
                }
            }
            FieldRef::Choice { options, index, set } => {
                if let Some(i) = value.as_str().and_then(|s| options.iter().position(|o| *o == s)) {
                    *index = i;
                    set(i);
                }
            }
        }
    }

    /// Applies a value only if its shape and numeric range match the field.
    /// Returns `false` without mutation for invalid data, unknown choice labels,
    /// integer overflow or non-finite float conversions.
    pub fn try_set_json(&mut self, value: &Value) -> bool {
        let valid = match &self.value {
            FieldRef::F32(_) => value.as_f64().is_some_and(|v| v.is_finite() && (v as f32).is_finite()),
            FieldRef::I32(_) => value.as_i64().is_some_and(|v| i32::try_from(v).is_ok()),
            FieldRef::U32(_) => value.as_u64().is_some_and(|v| u32::try_from(v).is_ok()),
            FieldRef::Bool(_) => value.is_boolean(),
            FieldRef::Vec3(_) => floats::<3>(value).is_some(),
            FieldRef::Color(_) => floats::<4>(value).is_some(),
            FieldRef::Text(_) | FieldRef::Asset(_, _) | FieldRef::Entity(_) => value.is_string(),
            FieldRef::Choice { options, .. } => value.as_str().is_some_and(|s| options.contains(&s)),
        };
        if valid {
            self.set_json(value);
        }
        valid
    }
}

/// Applies saved properties and returns warnings for unknown or invalid values.
/// Valid fields still apply when another property is invalid; missing properties
/// preserve current values. Warning order follows JSON map iteration order.
pub fn apply_props_checked(fields: &mut [Field], props: &serde_json::Map<String, Value>) -> Vec<String> {
    let mut warnings = Vec::new();
    for (name, value) in props {
        match fields.iter_mut().find(|f| f.name == name) {
            Some(field) => {
                if !field.try_set_json(value) {
                    warnings.push(format!("invalid value for property '{name}'"));
                }
            }
            None => warnings.push(format!("unknown property '{name}'")),
        }
    }
    warnings
}

fn floats<const N: usize>(v: &Value) -> Option<[f32; N]> {
    let arr = v.as_array()?;
    let mut out = [0.0; N];
    if arr.len() != N {
        return None;
    }
    for (o, x) in out.iter_mut().zip(arr) {
        *o = x.as_f64()? as f32;
        if !o.is_finite() {
            return None;
        }
    }
    Some(out)
}

/// Writes saved JSON values into matching fields.
pub fn apply_props(fields: &mut [Field], props: &serde_json::Map<String, Value>) {
    for f in fields {
        if let Some(v) = props.get(f.name) {
            f.set_json(v);
        }
    }
}

/// Snapshots editable values by property name; duplicate names use the last value.
pub fn read_props(fields: &[Field]) -> serde_json::Map<String, Value> {
    fields.iter().map(|f| (f.name.to_string(), f.to_json())).collect()
}
