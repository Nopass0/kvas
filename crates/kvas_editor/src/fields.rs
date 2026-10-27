//! Generic widgets for reflected fields: one function draws any component or script property.

use crate::i18n::{t, td};
use eframe::egui::{self, Color32, DragValue, Ui};
use kvas_engine::mesh::PRIMITIVES;
use kvas_engine::reflect::{AssetKind, Field, FieldRef};

/// Names the widgets can offer in dropdowns.
pub struct Lists<'a> {
    pub assets: &'a [String],
    pub entities: &'a [String],
}

fn matches(kind: AssetKind, path: &str) -> bool {
    let ext = path.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    match kind {
        AssetKind::Mesh => matches!(ext.as_str(), "obj" | "gltf" | "glb" | "fbx"),
        AssetKind::Texture => matches!(ext.as_str(), "png" | "jpg" | "jpeg"),
        AssetKind::Shader => ext == "wgsl" || ext == "kmat" || ext == "kshader",
        AssetKind::Audio => matches!(ext.as_str(), "wav" | "ogg" | "mp3" | "flac" | "m4a"),
    }
}

fn pick(ui: &mut Ui, id: &str, current: &mut String, options: impl Iterator<Item = String>, none_label: &str) -> bool {
    let mut changed = false;
    let shown = if current.is_empty() { none_label.to_string() } else { current.clone() };
    egui::ComboBox::from_id_salt(id).selected_text(shown).width(150.0).show_ui(ui, |ui| {
        if ui.selectable_label(current.is_empty(), none_label).clicked() && !current.is_empty() {
            current.clear();
            changed = true;
        }
        for o in options {
            if ui.selectable_label(*current == o, &o).clicked() && *current != o {
                *current = o;
                changed = true;
            }
        }
    });
    changed
}

fn vec3_ui(ui: &mut Ui, v: &mut glam::Vec3) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 3.0;
        for (label, color, value) in [
            ("X", Color32::from_rgb(230, 90, 90), &mut v.x),
            ("Y", Color32::from_rgb(110, 200, 110), &mut v.y),
            ("Z", Color32::from_rgb(100, 140, 240), &mut v.z),
        ] {
            ui.colored_label(color, label);
            changed |= ui.add(DragValue::new(value).speed(0.05).max_decimals(3)).changed();
        }
    });
    changed
}

/// Draws one field's editor and returns true if the value changed.
pub fn field_ui(ui: &mut Ui, field: &mut Field, id: &str, lists: &Lists) -> bool {
    let mut changed = false;
    match &mut field.value {
        FieldRef::F32(v) => changed |= ui.add(DragValue::new(&mut **v).speed(0.02).max_decimals(3)).changed(),
        FieldRef::I32(v) => changed |= ui.add(DragValue::new(&mut **v)).changed(),
        FieldRef::U32(v) => changed |= ui.add(DragValue::new(&mut **v)).changed(),
        FieldRef::Bool(v) => changed |= ui.checkbox(&mut **v, "").changed(),
        FieldRef::Vec3(v) => changed |= vec3_ui(ui, v),
        FieldRef::Color(c) => changed |= ui.color_edit_button_rgba_unmultiplied(&mut **c).changed(),
        FieldRef::Text(s) => changed |= ui.text_edit_singleline(&mut **s).changed(),
        FieldRef::Asset(s, kind) => {
            let kind = *kind;
            let mut options: Vec<String> = Vec::new();
            if kind == AssetKind::Mesh {
                options.extend(PRIMITIVES.iter().map(|p| format!("primitive:{p}")));
            }
            options.extend(lists.assets.iter().filter(|a| matches(kind, a)).cloned());
            changed |= pick(ui, id, s, options.into_iter(), t("None"));
        }
        FieldRef::Entity(s) => changed |= pick(ui, id, s, lists.entities.iter().cloned(), t("(World)")),
        FieldRef::Choice { options, index, set } => {
            egui::ComboBox::from_id_salt(id).selected_text(options[*index]).show_ui(ui, |ui| {
                for (i, name) in options.iter().enumerate() {
                    if ui.selectable_label(*index == i, *name).clicked() && *index != i {
                        *index = i;
                        set(i);
                        changed = true;
                    }
                }
            });
        }
    }
    changed
}

/// A label + editor row for every field. Returns true if any changed.
pub fn fields_grid(ui: &mut Ui, id: &str, fields: &mut [Field], lists: &Lists) -> bool {
    let mut changed = false;
    egui::Grid::new(id).num_columns(2).spacing([10.0, 5.0]).min_col_width(78.0).show(ui, |ui| {
        for f in fields.iter_mut() {
            ui.label(td(&f.name.replace('_', " ")));
            let field_id = format!("{id}/{}", f.name);
            changed |= field_ui(ui, f, &field_id, lists);
            ui.end_row();
        }
    });
    changed
}
