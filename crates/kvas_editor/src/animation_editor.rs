//! Keyframe recording, scrubbing and local-transform timeline authoring.
use crate::{editor::Editor, theme};
use eframe::egui::{self, Color32};
use kvas_engine::{prelude::*, animation::Interpolation};

impl Editor {
    pub fn animation_window(&mut self, ctx: &egui::Context) {
        if !self.animation_open { return; }
        let mut open = true;
        egui::Window::new("Анимация — таймлайн").open(&mut open).default_size([720.0, 420.0]).show(ctx, |ui| {
            if let Some(e) = self.selection {
                ui.heading(hierarchy::name_of(&self.active().world, e));
                self.begin_edit();
                self.animation_clip_ui(ui, e);
            } else { ui.label("Выберите объект в иерархии."); }
        });
        self.animation_open = open;
    }

    pub fn animation_clip_ui(&mut self, ui: &mut egui::Ui, e: Entity) {
        let Some(mut clip) = self.active().world.get::<TransformAnimation>(e).cloned() else { return };
        let pose = self.active().world.get::<Transform>(e).copied().unwrap_or_default();
        let mut scrub = false;
        ui.horizontal(|ui| {
            ui.checkbox(&mut clip.playing, "Играть в Play");
            ui.checkbox(&mut clip.looping, "Цикл");
            ui.label("Скорость"); ui.add(egui::DragValue::new(&mut clip.speed).speed(0.05).range(-10.0..=10.0));
        });
        ui.horizontal(|ui| {
            ui.label("Интерполяция");
            for (kind, name) in [(Interpolation::Step, "Шаг"), (Interpolation::Linear, "Линейная"), (Interpolation::Smooth, "Плавная")] { ui.selectable_value(&mut clip.interpolation, kind, name); }
        });
        ui.horizontal(|ui| {
            ui.label("Время (с)");
            scrub |= ui.add(egui::DragValue::new(&mut clip.time).speed(1.0 / 30.0).range(0.0..=3600.0)).changed();
            if ui.button("◆ Записать позу").clicked() { clip.set_key(clip.time, pose); }
        });
        let duration = clip.duration().max(1.0).max(clip.time);
        let (rect, response) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 86.0), egui::Sense::click_and_drag());
        let painter = ui.painter();
        painter.rect_filled(rect, 6.0, theme::pal().field);
        for i in 0..=10 {
            let x = rect.left() + rect.width() * i as f32 / 10.0;
            painter.line_segment([egui::pos2(x, rect.top() + 20.0), egui::pos2(x, rect.bottom())], egui::Stroke::new(1.0, theme::pal().border_strong));
            painter.text(egui::pos2(x + 2.0, rect.top() + 4.0), egui::Align2::LEFT_TOP, format!("{:.2}", duration * i as f32 / 10.0), egui::FontId::monospace(10.0), theme::pal().text_dim);
        }
        for key in &clip.keys {
            let pos = egui::pos2(rect.left() + key.time / duration * rect.width(), rect.center().y + 8.0);
            painter.text(pos, egui::Align2::CENTER_CENTER, "◆", egui::FontId::proportional(18.0), theme::ACCENT);
        }
        let x = rect.left() + clip.time / duration * rect.width();
        painter.line_segment([egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())], egui::Stroke::new(2.0, Color32::WHITE));
        if response.clicked() || response.dragged() {
            if let Some(p) = response.interact_pointer_pos() { clip.time = ((p.x - rect.left()) / rect.width()).clamp(0.0, 1.0) * duration; scrub = true; }
        }
        let mut remove = None;
        egui::ScrollArea::vertical().id_salt(("animation-keys", e.index())).max_height(220.0).show(ui, |ui| {
            for (i, key) in clip.keys.iter_mut().enumerate() {
                ui.push_id(i, |ui| { ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut key.time).speed(0.01).range(0.0..=3600.0).suffix(" с"));
                    ui.label("XYZ");
                    for v in [&mut key.pose.position.x, &mut key.pose.position.y, &mut key.pose.position.z] { ui.add(egui::DragValue::new(v).speed(0.05)); }
                    if ui.small_button("×").clicked() { remove = Some(i); }
                }); });
            }
        });
        if let Some(i) = remove { clip.keys.remove(i); }
        clip.keys.sort_by(|a, b| a.time.total_cmp(&b.time));
        if scrub { if let Some(pose) = clip.sample(clip.time) { self.active_mut().world.insert(e, pose); } }
        self.active_mut().world.insert(e, clip);
        ui.weak("Поверните/переместите объект, задайте время и запишите позу. Вращение и масштаб записываются вместе с XYZ. Воспроизведение запускается в Play.");
    }
}
