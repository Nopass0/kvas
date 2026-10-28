//! Layered texture authoring: paint/erase, lasso/rectangle, history, native JSON and PNG export.
use crate::{editor::Editor, image_document::ImageDocument, theme};
use eframe::egui;

#[derive(Clone, Copy, PartialEq)]
enum Tool { Brush, Erase, Lasso, Rectangle }
pub struct ImageEditorState {
    pub open: bool,
    doc: ImageDocument,
    layer: usize,
    tool: Tool,
    color: [u8; 4],
    radius: f32,
    hardness: f32,
    pressure: bool,
    selection: Vec<[f32; 2]>,
    last: Option<[f32; 2]>,
    rect_start: Option<[f32; 2]>,
    texture: Option<egui::TextureHandle>,
    refresh: bool,
    undo: Vec<ImageDocument>,
    redo: Vec<ImageDocument>,
    path: String,
    status: String,
}
impl Default for ImageEditorState {
    fn default() -> Self { Self { open: false, doc: ImageDocument::new(512, 512), layer: 0, tool: Tool::Brush, color: [80, 145, 235, 255], radius: 16.0, hardness: 0.7, pressure: true, selection: Vec::new(), last: None, rect_start: None, texture: None, refresh: true, undo: Vec::new(), redo: Vec::new(), path: "textures/painted.png".into(), status: String::new() } }
}
impl ImageEditorState {
    fn snapshot(&mut self) {
        self.undo.push(self.doc.clone()); self.redo.clear();
        let bytes = self.doc.width as usize * self.doc.height as usize * 4;
        while self.undo.len() > 1 && self.undo.iter().map(|d| bytes * d.layers.len()).sum::<usize>() > 64 * 1024 * 1024 { self.undo.remove(0); }
    }
    fn history(&mut self, undo: bool) {
        let (from, to) = if undo { (&mut self.undo, &mut self.redo) } else { (&mut self.redo, &mut self.undo) };
        if let Some(doc) = from.pop() { to.push(std::mem::replace(&mut self.doc, doc)); self.layer = self.layer.min(self.doc.layers.len() - 1); self.refresh = true; }
    }
}

impl Editor {
    pub fn image_editor_window(&mut self, ctx: &egui::Context) {
        if !self.image_editor.open { return; }
        let mut s = std::mem::take(&mut self.image_editor);
        let mut open = true;
        egui::Window::new("Изображение — слои и кисти").open(&mut open).default_size([960.0, 720.0]).show(ctx, |ui| {
            if !ctx.egui_wants_keyboard_input() {
                if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::Z)) { s.history(true); }
                if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::Y)) { s.history(false); }
            }
            ui.horizontal_wrapped(|ui| {
                for (tool, name) in [(Tool::Brush, "Кисть"), (Tool::Erase, "Ластик"), (Tool::Lasso, "Лассо"), (Tool::Rectangle, "Прямоугольник")] { ui.selectable_value(&mut s.tool, tool, name); }
                ui.color_edit_button_srgba_unmultiplied(&mut s.color);
                ui.add(egui::DragValue::new(&mut s.radius).range(1.0..=200.0).prefix("px "));
                ui.add(egui::Slider::new(&mut s.hardness, 0.0..=1.0).text("Жёсткость"));
                ui.checkbox(&mut s.pressure, "Давление");
            });
            ui.horizontal(|ui| {
                if ui.button("Отмена").clicked() { s.history(true); }
                if ui.button("Повтор").clicked() { s.history(false); }
                if ui.button("Снять выделение").clicked() { s.selection.clear(); }
                if ui.button("Заливка слоя / выделения").clicked() { s.snapshot(); s.doc.fill(s.layer, s.color, &s.selection); s.refresh = true; }
                if ui.button("Новый 512²").clicked() { s.snapshot(); s.doc = ImageDocument::new(512, 512); s.layer = 0; s.selection.clear(); s.refresh = true; }
            });
            ui.horizontal(|ui| {
                ui.label("Файл в assets"); ui.text_edit_singleline(&mut s.path);
                if ui.button("Открыть").clicked() {
                    let result = (|| -> Result<ImageDocument, String> {
                        let bytes = self.assets.read(&s.path).map_err(|e| e.to_string())?;
                        if s.path.ends_with(".kimage") { let d: ImageDocument = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?; d.validate()?; return Ok(d); }
                        let image = image::load_from_memory(&bytes).map_err(|e| e.to_string())?.to_rgba8();
                        if image.width() > 1024 || image.height() > 1024 { return Err("Редактор поддерживает изображения до 1024×1024; исходник не изменён.".into()); }
                        let mut d = ImageDocument::new(image.width(), image.height()); d.layers[0].pixels = image.into_raw(); Ok(d)
                    })();
                    match result { Ok(doc) => { s.snapshot(); s.doc = doc; s.layer = 0; s.selection.clear(); s.refresh = true; s.status = "Открыто".into(); }, Err(e) => s.status = e }
                }
                if ui.button("PNG + слои").clicked() {
                    let result = (|| -> Result<(), String> {
                        s.doc.validate()?;
                        let rel = kvas_engine::assets::normalize_path(&s.path).map_err(|e| e.to_string())?;
                        let path = self.project.assets_dir().join(rel).with_extension("png");
                        std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
                        let pixels = s.doc.composite();
                        image::save_buffer(&path, &pixels, s.doc.width, s.doc.height, image::ColorType::Rgba8).map_err(|e| e.to_string())?;
                        std::fs::write(path.with_extension("kimage"), serde_json::to_vec(&s.doc).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
                        Ok(())
                    })();
                    match result { Ok(()) => { self.reload_assets(); self.renderer.refresh_changed(); s.status = "PNG и редактируемые слои .kimage сохранены.".into(); }, Err(e) => s.status = e }
                }
            });
            if s.refresh {
                let image = egui::ColorImage::from_rgba_unmultiplied([s.doc.width as usize, s.doc.height as usize], &s.doc.composite());
                match &mut s.texture { Some(texture) => texture.set(image, egui::TextureOptions::LINEAR), None => s.texture = Some(ctx.load_texture("image-editor-canvas", image, egui::TextureOptions::LINEAR)) }
                s.refresh = false;
            }
            ui.horizontal_top(|ui| {
                ui.vertical(|ui| {
                    ui.set_width(190.0); ui.heading("Слои");
                    if ui.add_enabled(s.doc.layers.len() < 8, egui::Button::new("+ Слой")).clicked() { s.snapshot(); s.doc.add_layer(); s.layer = s.doc.layers.len() - 1; s.refresh = true; }
                    if ui.add_enabled(s.doc.layers.len() > 1, egui::Button::new("Удалить слой")).clicked() { s.snapshot(); s.doc.layers.remove(s.layer); s.layer = s.layer.min(s.doc.layers.len() - 1); s.refresh = true; }
                    for i in (0..s.doc.layers.len()).rev() {
                        ui.push_id(i, |ui| {
                            ui.horizontal(|ui| { s.refresh |= ui.checkbox(&mut s.doc.layers[i].visible, "").changed(); ui.selectable_value(&mut s.layer, i, &s.doc.layers[i].name); });
                            if s.layer == i {
                                ui.text_edit_singleline(&mut s.doc.layers[i].name);
                                s.refresh |= ui.add(egui::Slider::new(&mut s.doc.layers[i].opacity, 0.0..=1.0).text("Opacity")).changed();
                            }
                        });
                    }
                    ui.horizontal(|ui| {
                        if ui.add_enabled(s.layer + 1 < s.doc.layers.len(), egui::Button::new("Выше")).clicked() { s.snapshot(); s.doc.layers.swap(s.layer, s.layer + 1); s.layer += 1; s.refresh = true; }
                        if ui.add_enabled(s.layer > 0, egui::Button::new("Ниже")).clicked() { s.snapshot(); s.doc.layers.swap(s.layer, s.layer - 1); s.layer -= 1; s.refresh = true; }
                    });
                });
                let available = ui.available_size().min(egui::vec2(720.0, 540.0));
                let scale = (available.x / s.doc.width as f32).min(available.y / s.doc.height as f32).max(0.05);
                let size = egui::vec2(s.doc.width as f32, s.doc.height as f32) * scale;
                let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click_and_drag());
                // Checkerboard makes alpha and erasing visible.
                for y in 0..(rect.height() / 16.0).ceil() as i32 { for x in 0..(rect.width() / 16.0).ceil() as i32 {
                    let cell = egui::Rect::from_min_size(rect.min + egui::vec2(x as f32, y as f32) * 16.0, egui::vec2(16.0, 16.0)).intersect(rect);
                    ui.painter().rect_filled(cell, 0.0, egui::Color32::from_gray(if (x + y) % 2 == 0 { 50 } else { 70 }));
                } }
                if let Some(texture) = &s.texture { ui.painter().image(texture.id(), rect, egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)), egui::Color32::WHITE); }
                let position = response.interact_pointer_pos().filter(|p| rect.contains(*p)).map(|p| [(p.x - rect.left()) / scale, (p.y - rect.top()) / scale]);
                if response.drag_started() || response.clicked() {
                    s.last = None;
                    match s.tool { Tool::Brush | Tool::Erase => s.snapshot(), Tool::Lasso => s.selection.clear(), Tool::Rectangle => s.rect_start = position }
                }
                if response.dragged() || response.clicked() {
                    if let Some(p) = position { match s.tool {
                        Tool::Brush | Tool::Erase => {
                            let pressure = if s.pressure { ui.input(|i| i.events.iter().rev().find_map(|e| match e { egui::Event::Touch { force: Some(f), .. } => Some(*f), _ => None }).unwrap_or(1.0)).clamp(0.05, 1.0) } else { 1.0 };
                            let previous = s.last.unwrap_or(p);
                            let distance = ((p[0] - previous[0]).powi(2) + (p[1] - previous[1]).powi(2)).sqrt();
                            let steps = (distance / (s.radius * 0.25).max(0.5)).ceil().max(1.0) as usize;
                            for step in 0..=steps.min(4096) {
                                let t = step as f32 / steps.max(1) as f32;
                                let center = [previous[0] + (p[0] - previous[0]) * t, previous[1] + (p[1] - previous[1]) * t];
                                s.doc.dab(s.layer, center, s.radius * pressure, s.color, s.tool == Tool::Erase, s.hardness, &s.selection);
                            }
                            s.last = Some(p); s.refresh = true;
                        }
                        Tool::Lasso => { if s.selection.len() < 4096 { s.selection.push(p); } }
                        Tool::Rectangle => if let Some(a) = s.rect_start { s.selection = vec![a, [p[0], a[1]], p, [a[0], p[1]]]; },
                    } }
                }
                if response.drag_stopped() { s.last = None; s.rect_start = None; }
                if s.selection.len() >= 2 {
                    let mut points: Vec<_> = s.selection.iter().map(|p| rect.min + egui::vec2(p[0], p[1]) * scale).collect(); points.push(points[0]);
                    ui.painter().add(egui::Shape::line(points, egui::Stroke::new(1.5, theme::ACCENT)));
                }
                if let Some(p) = ui.input(|i| i.pointer.hover_pos()).filter(|p| rect.contains(*p)) { ui.painter().circle_stroke(p, s.radius * scale, egui::Stroke::new(1.0, egui::Color32::WHITE)); }
            });
            ui.label(&s.status);
            ui.weak("1024² максимум, 8 слоёв, alpha-over; история кистей/структуры до 64 MiB. Pressure используется только при наличии touch-force событий платформы; стилус аппаратно не проверен.");
        });
        s.open = open; self.image_editor = s;
    }
}
