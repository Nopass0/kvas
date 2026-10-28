//! GUI canvas uses the same components and layout math as the game/player.
use crate::editor::Editor;
use eframe::egui;
use kvas_engine::prelude::*;

impl Editor {
    pub fn gui_window(&mut self, ctx: &egui::Context) {
        if !self.gui_open { return; }
        let mut open = true;
        egui::Window::new("GUI — интерфейс игры").open(&mut open).default_size([740.0, 500.0]).show(ctx, |ui| {
            ui.horizontal(|ui| {
                for (kind, label) in [(UiKind::Label, "+ Текст"), (UiKind::Button, "+ Кнопка"), (UiKind::Progress, "+ Прогресс")] {
                    if ui.button(label).clicked() { self.create_entity(label.trim_start_matches("+ "), |world, e| { world.insert(e, UiElement { kind, text: label.trim_start_matches("+ ").into(), anchor: Vec3::new(0.5, 0.5, 0.0), ..Default::default() }); }); }
                }
                if ui.button("Сохранить сцену").clicked() { self.save_scene(false); }
            });
            ui.weak("Canvas 640×360. Перетаскивайте элементы; текст, anchors, размер и signal — в инспекторе. world_space привязывает надпись к позиции объекта в мире.");
            let (rect, _) = ui.allocate_exact_size(egui::vec2(640.0, 360.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, 6.0, egui::Color32::from_rgb(22, 25, 32));
            for x in (0..=640).step_by(40) { ui.painter().line_segment([rect.min + egui::vec2(x as f32, 0.0), rect.min + egui::vec2(x as f32, 360.0)], egui::Stroke::new(1.0, egui::Color32::from_gray(45))); }
            let view = kvas_render::game_view(self.active(), 16.0 / 9.0);
            kvas_render::shell::draw_game_ui(ui, self.active(), rect, view.as_ref(), false);
            let mut items: Vec<_> = self.active().world.iter::<UiElement>().filter_map(|(e, item)| kvas_render::shell::ui_bounds(self.active(), e, item, rect, view.as_ref()).map(|r| (e, r, item.world_space))).collect();
            items.reverse();
            for (e, bounds, world_space) in items {
                let response = ui.interact(bounds.intersect(rect), ui.make_persistent_id(("gui-canvas", e.to_bits())), egui::Sense::click_and_drag());
                if response.clicked() || response.drag_started() { self.selection = Some(e); }
                if response.dragged() && !world_space && !self.is_playing() {
                    self.begin_edit();
                    if let Some(item) = self.active_mut().world.get_mut::<UiElement>(e) { item.offset += Vec3::new(response.drag_delta().x, response.drag_delta().y, 0.0); }
                }
                if self.selection == Some(e) { ui.painter().rect_stroke(bounds, 6.0, egui::Stroke::new(1.5, crate::theme::ACCENT), egui::StrokeKind::Inside); }
            }
            ui.weak("Кнопки отправляют signal в следующий tick: ctx.signals(\"ui.click\"). Работает в Play и отдельной игре.");
        });
        self.gui_open = open;
    }
}
