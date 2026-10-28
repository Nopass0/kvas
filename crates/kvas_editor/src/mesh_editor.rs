//! Persistent vertex editing, a surface sculpt brush, box UV projection and OBJ export.
use crate::{editor::Editor, viewport::{to_ndc, to_screen}};
use eframe::egui;
use kvas_engine::{mesh_edit::EditableMesh, prelude::*};
use kvas_render::{debug::DebugDraw, View};
use std::collections::BTreeSet;

pub struct MeshEditState {
    pub open: bool,
    pub entity: Option<Entity>,
    selected: BTreeSet<usize>,
    sculpt: bool,
    radius: f32,
    strength: f32,
    offset: Vec3,
    uv_scale: f32,
    stroke: bool,
}
impl Default for MeshEditState {
    fn default() -> Self { Self { open: false, entity: None, selected: BTreeSet::new(), sculpt: false, radius: 0.5, strength: 0.5, offset: Vec3::ZERO, uv_scale: 1.0, stroke: false } }
}

impl Editor {
    pub fn start_mesh_edit(&mut self) {
        let Some(e) = self.selection else { self.status = "Выберите объект с MeshRenderer.".into(); return };
        if self.is_playing() { return; }
        let w = &self.active().world;
        if w.has::<Cloth>(e) || w.has::<VoxelWater>(e) || w.has::<VoxelSmoke>(e) || w.has::<VoxelClouds>(e) || w.has::<Grass>(e) || w.has::<ParticleEmitter>(e) {
            self.status = "Для редактирования создайте статическую копию процедурного объекта.".into(); return;
        }
        if !w.has::<EditableMesh>(e) {
            let Some(mesh) = self.cpu_mesh(e) else { self.status = "Не удалось загрузить геометрию.".into(); return };
            match EditableMesh::from_mesh(&mesh) {
                Ok(editable) => {
                    self.begin_edit();
                    let world = &mut self.active_mut().world;
                    world.remove::<MeshModifiers>(e);
                    world.remove::<kvas_engine::meshmod::ModifierStateCell>(e);
                    world.insert(e, editable);
                }
                Err(e) => { self.console.error(e); return; }
            }
        }
        self.mesh_edit = MeshEditState { open: true, entity: Some(e), ..Default::default() };
    }

    pub fn mesh_editor_window(&mut self, ctx: &egui::Context) {
        if !self.mesh_edit.open { return; }
        let mut open = true;
        egui::Window::new("Геометрия — вершины / sculpt / UV").open(&mut open).default_size([620.0, 490.0]).show(ctx, |ui| {
            let Some(e) = self.mesh_edit.entity.filter(|&e| self.active().world.is_alive(e)) else { ui.label("Объект удалён или сцена изменена."); return };
            let Some(mut edit) = self.active().world.get::<EditableMesh>(e).cloned() else { ui.label("У объекта нет EditableMesh (возможно, выполнена отмена)."); return };
            self.begin_edit();
            ui.heading(hierarchy::name_of(&self.active().world, e));
            ui.label(format!("{} вершин · {} треугольников · выделено {}", edit.vertices.len(), edit.indices.len() / 3, self.mesh_edit.selected.len()));
            ui.checkbox(&mut self.mesh_edit.sculpt, "Кисть Sculpt в сцене (Ctrl = вдавить)");
            ui.add(egui::Slider::new(&mut self.mesh_edit.radius, 0.01..=5.0).logarithmic(true).text("Радиус в локальных единицах"));
            ui.add(egui::Slider::new(&mut self.mesh_edit.strength, 0.01..=3.0).text("Сила / секунду"));
            ui.weak("В режиме вершин: клик выбирает ближайшую вершину, Ctrl/Shift добавляет. Видимые маркеры не выполняют occlusion test.");
            let mut changed = false;
            ui.horizontal(|ui| {
                if ui.button("Все вершины").clicked() { self.mesh_edit.selected = (0..edit.vertices.len()).collect(); }
                if ui.button("Снять выделение").clicked() { self.mesh_edit.selected.clear(); }
                if ui.button("Пересчитать нормали").clicked() { let _ = edit.recalculate_normals(); changed = true; }
            });
            ui.horizontal(|ui| {
                ui.label("Смещение XYZ");
                for value in [&mut self.mesh_edit.offset.x, &mut self.mesh_edit.offset.y, &mut self.mesh_edit.offset.z] { ui.add(egui::DragValue::new(value).speed(0.01)); }
                if ui.button("Применить").clicked() {
                    for &i in &self.mesh_edit.selected { if let Some(v) = edit.vertices.get_mut(i) { v.pos = (Vec3::from(v.pos) + self.mesh_edit.offset).to_array(); } }
                    let _ = edit.recalculate_normals(); changed = true; self.mesh_edit.offset = Vec3::ZERO;
                }
            });
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut self.mesh_edit.uv_scale).range(0.001..=1000.0).speed(0.05).prefix("UV scale "));
                if ui.button("Box UV projection").clicked() { let _ = edit.box_uv(self.mesh_edit.uv_scale); changed = true; }
                if ui.button("Проверка UV").clicked() { self.debug_view = if self.debug_view == 4 { 0 } else { 4 }; }
            });
            egui::ScrollArea::vertical().id_salt("editable-vertices").max_height(180.0).show_rows(ui, 24.0, edit.vertices.len(), |ui, rows| {
                for i in rows { ui.push_id(i, |ui| { ui.horizontal(|ui| {
                    if ui.selectable_label(self.mesh_edit.selected.contains(&i), format!("{i:05}")).clicked() {
                        if !ui.input(|i| i.modifiers.ctrl || i.modifiers.shift) { self.mesh_edit.selected.clear(); }
                        if !self.mesh_edit.selected.insert(i) { self.mesh_edit.selected.remove(&i); }
                    }
                    for axis in &mut edit.vertices[i].pos { changed |= ui.add(egui::DragValue::new(axis).speed(0.01)).changed(); }
                }); }); }
            });
            if changed { edit.changed(); self.active_mut().world.insert(e, edit.clone()); }
            ui.horizontal(|ui| {
                if ui.button("Экспорт нового OBJ").clicked() {
                    let result = (|| -> Result<String, String> {
                        let text = edit.to_obj()?;
                        let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_err(|e| e.to_string())?.as_millis();
                        let rel = format!("models/edited/mesh_{}_{stamp}.obj", e.index());
                        let path = self.project.assets_dir().join(&rel);
                        std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
                        std::fs::write(path, text).map_err(|e| e.to_string())?; Ok(rel)
                    })();
                    match result { Ok(path) => { self.reload_assets(); self.status = format!("Экспортировано: {path}"); }, Err(e) => self.console.error(e) }
                }
                if ui.button("Сохранить сцену").clicked() { self.save_scene(false); }
            });
            ui.weak("EditableMesh сохраняется в сцене и работает в отдельной игре. Начало редактирования запекает стек модификаторов в геометрию. OBJ сохраняет геометрию/UV/нормали; glTF material runs не экспортируются.");
        });
        self.mesh_edit.open = open;
    }

    pub fn mesh_edit_in_viewport(&mut self, ui: &egui::Ui, resp: &egui::Response, rect: egui::Rect, view: &View, draw: &mut DebugDraw) {
        let Some(e) = self.mesh_edit.entity else { return };
        let Some(mut edit) = self.active().world.get::<EditableMesh>(e).cloned() else { return };
        let global = hierarchy::global_matrix(&self.active().world, e);
        let inv = global.inverse(); if !inv.is_finite() { return; }
        if !ui.input(|i| i.pointer.primary_down()) && self.mesh_edit.stroke {
            let _ = edit.recalculate_normals(); self.active_mut().world.insert(e, edit.clone()); self.mesh_edit.stroke = false;
        }
        let mouse = ui.input(|i| i.pointer.hover_pos()).filter(|p| rect.contains(*p) && resp.hovered());
        if self.mesh_edit.sculpt {
            let Some(mouse) = mouse else { return };
            let (origin, direction) = view.ray(to_ndc(rect, mouse));
            let origin = inv.transform_point3(origin); let direction = inv.transform_vector3(direction);
            let hit = edit.indices.chunks_exact(3).filter_map(|t| {
                let a = Vec3::from(edit.vertices.get(t[0] as usize)?.pos); let b = Vec3::from(edit.vertices.get(t[1] as usize)?.pos); let c = Vec3::from(edit.vertices.get(t[2] as usize)?.pos);
                triangle_hit(origin, direction, a, b, c)
            }).min_by(|a, b| a.total_cmp(b));
            if let Some(t) = hit {
                let center = origin + direction * t;
                let r = self.mesh_edit.radius;
                draw.overlay_box(global * Mat4::from_translation(center), Vec3::splat(-r), Vec3::splat(r), [0.2, 0.8, 1.0, 1.0]);
                if ui.input(|i| i.pointer.primary_down() && !i.modifiers.alt) {
                    self.begin_edit(); self.mesh_edit.stroke = true;
                    let amount = self.mesh_edit.strength * ui.input(|i| i.stable_dt.min(0.05)) * if ui.input(|i| i.modifiers.ctrl) { -1.0 } else { 1.0 };
                    if edit.sculpt(center, r, amount) { self.active_mut().world.insert(e, edit); }
                }
            }
        } else {
            let mut closest = None;
            for (i, v) in edit.vertices.iter().enumerate() {
                let world = global.transform_point3(Vec3::from(v.pos));
                if let (Some(mouse), Some(screen)) = (mouse, to_screen(view, rect, world)) {
                    let distance = screen.distance(mouse);
                    if distance < 10.0 && closest.is_none_or(|(_, d)| distance < d) { closest = Some((i, distance)); }
                }
                if self.mesh_edit.selected.contains(&i) || i % edit.vertices.len().div_ceil(4000).max(1) == 0 {
                    let radius = (world.distance(view.eye) * 0.003).max(0.01);
                    let color = if self.mesh_edit.selected.contains(&i) { [1.0, 0.6, 0.1, 1.0] } else { [0.2, 0.65, 1.0, 0.6] };
                    draw.overlay_line(world - Vec3::X * radius, world + Vec3::X * radius, color);
                    draw.overlay_line(world - Vec3::Y * radius, world + Vec3::Y * radius, color);
                }
            }
            if resp.clicked() && !ui.input(|i| i.modifiers.alt) {
                if !ui.input(|i| i.modifiers.ctrl || i.modifiers.shift) { self.mesh_edit.selected.clear(); }
                if let Some((i, _)) = closest { if !self.mesh_edit.selected.insert(i) { self.mesh_edit.selected.remove(&i); } }
            }
        }
    }
}

fn triangle_hit(o: Vec3, d: Vec3, a: Vec3, b: Vec3, c: Vec3) -> Option<f32> {
    let e1 = b - a; let e2 = c - a; let p = d.cross(e2); let det = e1.dot(p);
    if det.abs() < 1e-8 { return None; }
    let u = (o - a).dot(p) / det; let q = (o - a).cross(e1); let v = d.dot(q) / det; let t = e2.dot(q) / det;
    (u >= 0.0 && v >= 0.0 && u + v <= 1.0 && t > 0.0).then_some(t)
}
