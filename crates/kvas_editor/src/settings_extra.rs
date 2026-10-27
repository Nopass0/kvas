//! Settings backed by actual runtime, build and viewport behavior.
use crate::{editor::Editor, settings::Page};
use eframe::egui;

impl Editor {
    pub fn settings_extra(&mut self, ui: &mut egui::Ui, page: Page) {
        match page {
            Page::Simulation => {
                let cfg = &mut self.project.config;
                ui.weak("Применяется при следующем запуске Play и в собранной игре.");
                let mut hz = 1.0 / cfg.timing.fixed_dt;
                if ui.add(egui::Slider::new(&mut hz, 15.0..=240.0).text("Частота физики (Гц)")).changed() {
                    cfg.timing.fixed_dt = 1.0 / hz; self.settings.dirty = true;
                }
                self.settings.dirty |= ui.add(egui::Slider::new(&mut cfg.timing.max_fixed_steps, 1..=32).text("Шагов за кадр")).changed();
                self.settings.dirty |= ui.add(egui::Slider::new(&mut cfg.timing.max_frame_dt, 0.016..=0.5).text("Предел dt (с)")).changed();
                self.settings.dirty |= ui.add(egui::Slider::new(&mut cfg.time_scale, 0.0..=4.0).text("Скорость времени")).changed();
                ui.weak("Лимит шагов предотвращает зависания при накоплении времени физики.");
            }
            Page::Audio => {
                self.settings.dirty |= ui.add(egui::Slider::new(&mut self.project.config.master_volume, 0.0..=1.0).text("Громкость игры")).changed();
                self.scene.audio.master_volume = self.project.config.master_volume;
                if let Some(play) = &mut self.play { play.runtime.scene.audio.master_volume = self.project.config.master_volume; }
                ui.weak("Клипы, волны и предпрослушивание: вкладка «Звук». Затухание настраивается в AudioSource.");
            }
            Page::Build => self.settings_build(ui),
            Page::Graphics => {
                ui.weak("Настройки текущей сцены. Сохраните сцену кнопкой ниже или Ctrl+S в основном окне.");
                if ui.button("Сохранить сцену").clicked() { self.save_scene(false); }
                self.begin_edit();
                self.scene_settings_ui(ui);
            }
            Page::Viewport => {
                let mut changed = ui.checkbox(&mut self.show_grid, "Сетка").changed();
                changed |= ui.checkbox(&mut self.show_colliders, "Коллайдеры").changed();
                changed |= ui.checkbox(&mut self.local_space, "Локальные оси").changed();
                changed |= ui.checkbox(&mut self.snap.grid, "Привязка к сетке").changed();
                changed |= ui.add(egui::Slider::new(&mut self.snap.move_step, 0.01..=10.0).logarithmic(true).text("Шаг перемещения")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.snap.rotate_step, 1.0..=90.0).text("Шаг вращения")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.snap.scale_step, 0.01..=1.0).text("Шаг масштаба")).changed();
                if changed { self.sync_prefs(); self.prefs.save(); }
            }
            Page::Shortcuts => {
                egui::Grid::new("shortcut-reference").striped(true).show(ui, |ui| {
                    for (key, action) in [("W / E / R", "Перемещение / вращение / масштаб"), ("Ctrl+S", "Сохранить сцену"), ("Ctrl+Z / Ctrl+Y", "Отмена / повтор"), ("Ctrl+D / Delete", "Дублировать / удалить выделение"), ("Ctrl / Shift + click", "Переключить / расширить выделение"), ("F", "Фокус на объекте"), ("Ctrl+P", "Запустить / остановить игру"), ("Ctrl+Alt+S", "Настройки"), ("Shift+A / Space", "Добавить ноду"), ("Home", "Показать весь граф"), ("RMB + WASDQE", "Полёт камеры"), ("Alt+LMB / MMB", "Орбита / панорамирование")] {
                        ui.monospace(key); ui.label(action); ui.end_row();
                    }
                });
            }
            _ => {}
        }
    }

    fn settings_build(&mut self, ui: &mut egui::Ui) {
        let before = self.project.config.build.clone();
        let b = &mut self.project.config.build;
        egui::Grid::new("build-settings").num_columns(2).show(ui, |ui| {
            for (label, value) in [("Заголовок", &mut b.title), ("Имя exe", &mut b.exe_name), ("Версия", &mut b.version), ("Компания", &mut b.company), ("Иконка", &mut b.icon)] {
                ui.label(label); ui.text_edit_singleline(value); ui.end_row();
            }
            ui.label("Размер окна"); ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut b.width).range(320..=7680));
                ui.label("×"); ui.add(egui::DragValue::new(&mut b.height).range(240..=4320));
            }); ui.end_row();
        });
        ui.checkbox(&mut b.release, "Оптимизированная сборка (release)");
        ui.checkbox(&mut b.fullscreen, "Полный экран");
        ui.checkbox(&mut b.maximized, "Развернуть окно");
        ui.checkbox(&mut b.resizable, "Разрешить изменение размера");
        ui.checkbox(&mut b.vsync, "VSync");
        self.settings.dirty |= before != *b;
        if ui.button("Открыть окно сборки").clicked() { self.open_build_dialog(); }
    }
}
