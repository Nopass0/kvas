use crate::editor::Editor;
use crate::i18n::{t, td};
use crate::icons::Icon;
use eframe::egui::{self, Color32, Sense};
use eframe::egui::collapsing_header::CollapsingState;
use kvas_engine::prelude::*;

/// Icon and tint that tell what an object is at a glance.
pub fn kind_icon(world: &World, e: Entity) -> (Icon, Color32) {
    let green = Color32::from_rgb(110, 200, 120);
    let blue = Color32::from_rgb(90, 150, 230);
    if world.has::<Camera>(e) {
        return (Icon::Camera, green);
    }
    if let Some(l) = world.get::<Light>(e) {
        let icon = match l.kind {
            LightKind::Directional => Icon::Sun,
            _ => Icon::Lightbulb,
        };
        return (icon, Color32::from_rgb(240, 210, 90));
    }
    if world.has::<AudioSource>(e) {
        return (Icon::Volume, Color32::from_rgb(190, 120, 230));
    }
    if world.has::<Cloth>(e) {
        return (Icon::Shirt, Color32::from_rgb(230, 140, 170));
    }
    if world.has::<Grass>(e) {
        return (Icon::Wind, Color32::from_rgb(120, 200, 90));
    }
    if world.has::<VoxelSmoke>(e) {
        return (Icon::Flame, Color32::from_rgb(240, 150, 80));
    }
    if world.has::<ParticleEmitter>(e) {
        return (Icon::Sparkles, Color32::from_rgb(255, 170, 80));
    }
    if world.has::<VoxelWater>(e) {
        return (Icon::Waves, Color32::from_rgb(90, 170, 230));
    }
    if world.has::<VoxelClouds>(e) {
        return (Icon::Cloud, Color32::from_rgb(170, 200, 235));
    }
    if let Some(m) = world.get::<MeshRenderer>(e) {
        let icon = match m.mesh.strip_prefix("primitive:") {
            Some("cube") => Icon::Box,
            Some("sphere") => Icon::Sphere,
            Some("plane") => Icon::Plane,
            Some("cylinder") => Icon::Cylinder,
            Some("capsule") => Icon::Capsule,
            Some("cone") => Icon::Cone,
            Some("torus") => Icon::Torus,
            Some("rock") => Icon::Rock,
            Some(_) => Icon::Shapes,
            None if m.mesh.is_empty() => Icon::Box,
            None => Icon::FileBox,
        };
        return (icon, blue);
    }
    if world.has::<Joint>(e) {
        return (Icon::Link, Color32::from_gray(160));
    }
    (Icon::Circle, Color32::from_gray(130))
}

impl Editor {
    /// Items shared by the GameObject menu and the hierarchy "+" button.
    pub fn create_menu(&mut self, ui: &mut egui::Ui) {
        if ui.button(t("Create Empty")).clicked() {
            self.create_empty();
            ui.close();
        }
        ui.menu_button(t("3D Object"), |ui| {
            for p in ["cube", "sphere", "plane", "cylinder", "capsule", "cone", "torus", "rock"] {
                if ui.button(td(&(p[..1].to_uppercase() + &p[1..]))).clicked() {
                    self.create_primitive(p, false);
                    ui.close();
                }
            }
        });
        ui.menu_button(t("Effects"), |ui| {
            for (kind, label) in [("grass", "Grass (wind)"), ("cloth", "Cloth"), ("smoke", "Voxel smoke"), ("clouds", "Voxel clouds"), ("water_voxel", "Voxel water"), ("water", "Water (smooth)"), ("water_real", "Water (realistic)"), ("paint_plane", "Painted surface")] {
                if ui.button(t(label)).clicked() {
                    self.create_procedural(kind);
                    ui.close();
                }
            }
        });
        ui.menu_button(t("Particles"), |ui| {
            for (key, label) in [("fire", "Fire"), ("smoke", "Smoke"), ("sparks", "Sparks"), ("explosion", "Explosion"), ("magic", "Magic"), ("snow", "Snow"), ("rain", "Rain"), ("dust", "Dust motes"), ("fog", "Fog"), ("mist", "Mist"), ("steam", "Steam"), ("embers", "Embers"), ("fireflies", "Fireflies"), ("leaves", "Falling leaves"), ("bubbles", "Bubbles"), ("campfire_smoke", "Campfire smoke"), ("dust_storm", "Dust storm")] {
                if ui.button(t(label)).clicked() {
                    self.create_particles(key, label);
                    ui.close();
                }
            }
        });
        ui.menu_button(t("Materials"), |ui| {
            if ui.button(t("New node material...")).clicked() {
                self.new_material();
                ui.close();
            }
            ui.separator();
            if ui.button(t("Glass sphere")).clicked() {
                self.create_material_sphere(true);
                ui.close();
            }
            if ui.button(t("Chrome sphere")).clicked() {
                self.create_material_sphere(false);
                ui.close();
            }
        });
        ui.menu_button(t("Physics Object"), |ui| {
            for p in ["cube", "sphere", "cylinder", "capsule", "torus"] {
                if ui.button(td(&format!("Rigid {p}"))).clicked() {
                    self.create_primitive(p, true);
                    ui.close();
                }
            }
        });
        ui.menu_button(t("Light"), |ui| {
            if ui.button(t("Directional Light")).clicked() {
                self.create_light(LightKind::Directional);
                ui.close();
            }
            if ui.button(t("Point Light")).clicked() {
                self.create_light(LightKind::Point);
                ui.close();
            }
            if ui.button(t("Spot Light")).clicked() {
                self.create_light(LightKind::Spot);
                ui.close();
            }
        });
        if ui.button(t("Camera")).clicked() {
            self.create_camera();
            ui.close();
        }
    }

    pub fn hierarchy_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.menu_button(t("+ Create"), |ui| self.create_menu(ui));
            ui.add(egui::TextEdit::singleline(&mut self.filter).hint_text(t("Search...")).desired_width(f32::INFINITY));
        });
        ui.separator();

        let area = egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
            let filter = self.filter.to_lowercase();
            if filter.is_empty() {
                for root in hierarchy::roots(&self.active().world) {
                    self.hierarchy_node(ui, root);
                }
            } else {
                let matches: Vec<Entity> = self.active().world.entities().into_iter().filter(|&e| hierarchy::name_of(&self.active().world, e).to_lowercase().contains(&filter)).collect();
                for e in matches {
                    self.hierarchy_row(ui, e);
                }
            }
            // Empty space below the list: drop here to unparent, right-click to create.
            let rest = ui.allocate_response(ui.available_size().max(egui::vec2(10.0, 40.0)), Sense::click());
            if let Some(dragged) = rest.dnd_release_payload::<Entity>() {
                self.reparent(*dragged, None);
            }
            if rest.clicked() {
                self.selection = None;
            }
            rest.context_menu(|ui| self.create_menu(ui));
        });
        let _ = area;
    }

    fn hierarchy_node(&mut self, ui: &mut egui::Ui, e: Entity) {
        let children = hierarchy::children(&self.active().world, e);
        if children.is_empty() {
            ui.horizontal(|ui| {
                ui.add_space(ui.spacing().indent);
                self.hierarchy_row(ui, e);
            });
            return;
        }
        let id = ui.make_persistent_id(("hierarchy", e));
        let mut state = CollapsingState::load_with_default_open(ui.ctx(), id, true);
        let header = ui.horizontal(|ui| {
            state.show_toggle_button(ui, egui::collapsing_header::paint_default_icon);
            self.hierarchy_row(ui, e);
        });
        state.show_body_indented(&header.response, ui, |ui| {
            for c in children {
                self.hierarchy_node(ui, c);
            }
        });
    }

    fn hierarchy_row(&mut self, ui: &mut egui::Ui, e: Entity) {
        let (name, (icon, color), parent) = {
            let w = &self.active().world;
            (hierarchy::name_of(w, e).to_string(), kind_icon(w, e), hierarchy::parent_of(w, e))
        };
        ui.add(icon.image(14.0, color));
        let label = if name.is_empty() { t("(unnamed)") } else { &name };
        let resp = ui.selectable_label(self.selected_entities().contains(&e), label).interact(Sense::click_and_drag());
        if resp.clicked() {
            let mut order = Vec::new();
            for root in hierarchy::roots(&self.active().world) {
                order.push(root);
                order.extend(hierarchy::descendants(&self.active().world, root));
            }
            if !self.filter.is_empty() {
                let filter = self.filter.to_lowercase();
                order.retain(|&e| hierarchy::name_of(&self.active().world, e).to_lowercase().contains(&filter));
            }
            self.select_entity(Some(e), ui.input(|i| i.modifiers), &order);
        }
        if resp.drag_started() {
            if !self.selected_entities().contains(&e) { self.select_entity(Some(e), egui::Modifiers::NONE, &[]); }
            resp.dnd_set_drag_payload(e);
        }
        if resp.dnd_hover_payload::<Entity>().is_some_and(|p| *p != e) {
            ui.painter().rect_stroke(resp.rect, 2.0, egui::Stroke::new(1.5, crate::theme::ACCENT), egui::StrokeKind::Inside);
        }
        if resp.dnd_hover_payload::<crate::assets_panel::ScriptDrag>().is_some() {
            ui.painter().rect_stroke(resp.rect, 2.0, egui::Stroke::new(1.5, crate::theme::ACCENT), egui::StrokeKind::Inside);
        }
        if let Some(asset) = resp.dnd_release_payload::<crate::assets_panel::AssetDrag>() {
            self.apply_asset_to(e, &asset.0);
        }
        if let Some(script) = resp.dnd_release_payload::<crate::assets_panel::ScriptDrag>() {
            self.attach_script(e, &script.0);
        }
        if let Some(dragged) = resp.dnd_release_payload::<Entity>() {
            if *dragged != e {
                let roots = if self.selected_entities().contains(&dragged) { self.selection_roots() } else { vec![*dragged] };
                if !roots.iter().any(|&root| root == e || hierarchy::is_ancestor(&self.active().world, root, e)) {
                    for root in roots { self.reparent(root, Some(e)); }
                }
            }
        }
        if resp.double_clicked() {
            self.selection = Some(e);
            self.focus_selection();
        }
        resp.context_menu(|ui| {
            if ui.button(t("Create Empty Child")).clicked() {
                self.create_empty();
                if let Some(child) = self.selection {
                    self.reparent(child, Some(e));
                }
                ui.close();
            }
            if ui.button(t("Duplicate")).clicked() {
                self.duplicate_entity(e);
                ui.close();
            }
            if parent.is_some() && ui.button(t("Unparent")).clicked() {
                self.reparent(e, None);
                ui.close();
            }
            ui.separator();
            if ui.button(t("Delete")).clicked() {
                self.delete_entity(e);
                ui.close();
            }
        });
    }
}
