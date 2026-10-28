//! Layered texture painting: brushes with pressure, eraser, bucket, gradient, eyedropper,
//! lasso / rectangle selection, layer blend modes, zoom and pan, history, native `.kimage` and PNG.
use crate::image_document::{Blend, Dab, ImageDocument, MAX_LAYERS};
use crate::{editor::Editor, theme};
use eframe::egui::{self, Key, Modifiers};

#[derive(Clone, Copy, PartialEq)]
enum Tool {
    Brush,
    Erase,
    Bucket,
    Gradient,
    Picker,
    Lasso,
    Rectangle,
}

impl Tool {
    const ALL: [(Tool, &'static str, &'static str); 7] = [
        (Tool::Brush, "Кисть", "B"),
        (Tool::Erase, "Ластик", "E"),
        (Tool::Bucket, "Заливка", "G"),
        (Tool::Gradient, "Градиент", "Shift+G"),
        (Tool::Picker, "Пипетка", "I, или Alt с кистью"),
        (Tool::Lasso, "Лассо", "L"),
        (Tool::Rectangle, "Прямоугольник", "M"),
    ];
}

/// Ready brush settings: name, hardness, opacity, flow, spacing.
const PRESETS: [(&str, f32, f32, f32, f32); 5] =
    [("Жёсткая круглая", 0.95, 1.0, 1.0, 0.15), ("Мягкая круглая", 0.0, 1.0, 0.6, 0.12), ("Аэрограф", 0.0, 0.6, 0.12, 0.08), ("Карандаш", 1.0, 1.0, 1.0, 0.05), ("Маркер", 0.7, 0.7, 0.35, 0.1)];

pub struct ImageEditorState {
    pub open: bool,
    doc: ImageDocument,
    layer: usize,
    tool: Tool,
    color: [u8; 4],
    secondary: [u8; 4],
    radius: f32,
    hardness: f32,
    opacity: f32,
    flow: f32,
    spacing: f32,
    tolerance: f32,
    pressure_size: bool,
    pressure_opacity: bool,
    selection: Vec<[f32; 2]>,
    last: Option<[f32; 2]>,
    rect_start: Option<[f32; 2]>,
    texture: Option<egui::TextureHandle>,
    refresh: bool,
    undo: Vec<ImageDocument>,
    redo: Vec<ImageDocument>,
    path: String,
    status: String,
    zoom: f32,
    pan: egui::Vec2,
    fit: bool,
    new_size: [u32; 2],
}

impl Default for ImageEditorState {
    fn default() -> Self {
        Self {
            open: false,
            doc: ImageDocument::new(512, 512),
            layer: 0,
            tool: Tool::Brush,
            color: [80, 145, 235, 255],
            secondary: [255, 255, 255, 255],
            radius: 16.0,
            hardness: 0.7,
            opacity: 1.0,
            flow: 0.8,
            spacing: 0.12,
            tolerance: 0.15,
            pressure_size: true,
            pressure_opacity: false,
            selection: Vec::new(),
            last: None,
            rect_start: None,
            texture: None,
            refresh: true,
            undo: Vec::new(),
            redo: Vec::new(),
            path: "textures/painted.png".into(),
            status: String::new(),
            zoom: 1.0,
            pan: egui::Vec2::ZERO,
            fit: true,
            new_size: [1024, 1024],
        }
    }
}

impl ImageEditorState {
    fn snapshot(&mut self) {
        self.undo.push(self.doc.clone());
        self.redo.clear();
        let bytes = self.doc.width as usize * self.doc.height as usize * 4;
        while self.undo.len() > 1 && self.undo.iter().map(|d| bytes * d.layers.len()).sum::<usize>() > 256 * 1024 * 1024 {
            self.undo.remove(0);
        }
    }
    fn history(&mut self, undo: bool) {
        let (from, to) = if undo { (&mut self.undo, &mut self.redo) } else { (&mut self.redo, &mut self.undo) };
        if let Some(doc) = from.pop() {
            to.push(std::mem::replace(&mut self.doc, doc));
            self.layer = self.layer.min(self.doc.layers.len() - 1);
            self.refresh = true;
        }
    }

    fn shortcuts(&mut self, ctx: &egui::Context) {
        if ctx.egui_wants_keyboard_input() {
            return;
        }
        ctx.input_mut(|i| {
            let mut key = |m: Modifiers, k: Key| i.consume_key(m, k);
            if key(Modifiers::COMMAND | Modifiers::SHIFT, Key::Z) || key(Modifiers::COMMAND, Key::Y) {
                self.history(false);
            } else if key(Modifiers::COMMAND, Key::Z) {
                self.history(true);
            }
            if key(Modifiers::COMMAND, Key::D) {
                self.selection.clear();
            }
            if key(Modifiers::COMMAND, Key::J) {
                self.snapshot();
                if self.doc.duplicate_layer(self.layer) {
                    self.layer += 1;
                }
                self.refresh = true;
            }
            if key(Modifiers::COMMAND, Key::E) && self.layer > 0 {
                self.snapshot();
                self.doc.merge_down(self.layer);
                self.layer -= 1;
                self.refresh = true;
            }
            if key(Modifiers::COMMAND, Key::Num0) {
                self.fit = true;
            }
            if key(Modifiers::NONE, Key::Delete) || key(Modifiers::NONE, Key::Backspace) {
                self.snapshot();
                self.doc.clear(self.layer, &self.selection);
                self.refresh = true;
            }
            for (k, tool) in [(Key::B, Tool::Brush), (Key::E, Tool::Erase), (Key::G, Tool::Bucket), (Key::I, Tool::Picker), (Key::L, Tool::Lasso), (Key::M, Tool::Rectangle)] {
                if key(Modifiers::NONE, k) {
                    self.tool = tool;
                }
            }
            if key(Modifiers::SHIFT, Key::G) {
                self.tool = Tool::Gradient;
            }
            if key(Modifiers::NONE, Key::X) {
                std::mem::swap(&mut self.color, &mut self.secondary);
            }
            if key(Modifiers::NONE, Key::OpenBracket) {
                self.radius = (self.radius / 1.2).max(1.0);
            }
            if key(Modifiers::NONE, Key::CloseBracket) {
                self.radius = (self.radius * 1.2).min(400.0);
            }
        });
    }
}

impl Editor {
    pub fn image_editor_window(&mut self, ctx: &egui::Context) {
        if !self.image_editor.open {
            return;
        }
        let mut s = std::mem::take(&mut self.image_editor);
        let mut open = true;
        egui::Window::new("Изображение — слои и кисти").open(&mut open).default_size([1180.0, 780.0]).show(ctx, |ui| {
            s.shortcuts(ctx);
            // Tools and brush settings.
            ui.horizontal_wrapped(|ui| {
                for (tool, name, keys) in Tool::ALL {
                    ui.selectable_value(&mut s.tool, tool, name).on_hover_text(keys);
                }
                ui.separator();
                ui.color_edit_button_srgba_unmultiplied(&mut s.color).on_hover_text("Основной цвет");
                if ui.small_button("⇄").on_hover_text("Поменять цвета (X)").clicked() {
                    std::mem::swap(&mut s.color, &mut s.secondary);
                }
                ui.color_edit_button_srgba_unmultiplied(&mut s.secondary).on_hover_text("Второй цвет (конец градиента)");
            });
            ui.horizontal_wrapped(|ui| {
                ui.add(egui::DragValue::new(&mut s.radius).range(1.0..=400.0).prefix("Размер ").suffix(" px")).on_hover_text("[ и ] — меньше / больше");
                ui.add(egui::Slider::new(&mut s.hardness, 0.0..=1.0).text("Жёсткость"));
                ui.add(egui::Slider::new(&mut s.opacity, 0.0..=1.0).text("Непрозрачность"));
                ui.add(egui::Slider::new(&mut s.flow, 0.01..=1.0).text("Нажим"));
                ui.add(egui::Slider::new(&mut s.spacing, 0.02..=1.0).text("Шаг"));
                if s.tool == Tool::Bucket {
                    ui.add(egui::Slider::new(&mut s.tolerance, 0.0..=1.0).text("Допуск"));
                }
                ui.checkbox(&mut s.pressure_size, "Перо → размер");
                ui.checkbox(&mut s.pressure_opacity, "Перо → прозрачность");
                egui::ComboBox::from_id_salt("brush-presets").selected_text("Кисти").show_ui(ui, |ui| {
                    for (name, hardness, opacity, flow, spacing) in PRESETS {
                        if ui.selectable_label(false, name).clicked() {
                            (s.hardness, s.opacity, s.flow, s.spacing) = (hardness, opacity, flow, spacing);
                            s.tool = Tool::Brush;
                        }
                    }
                });
            });
            ui.horizontal(|ui| {
                if ui.button("Отмена").on_hover_text("Ctrl+Z").clicked() {
                    s.history(true);
                }
                if ui.button("Повтор").on_hover_text("Ctrl+Y").clicked() {
                    s.history(false);
                }
                if ui.button("Снять выделение").on_hover_text("Ctrl+D").clicked() {
                    s.selection.clear();
                }
                if ui.button("Залить").clicked() {
                    s.snapshot();
                    s.doc.fill(s.layer, s.color, &s.selection);
                    s.refresh = true;
                }
                if ui.button("Очистить").on_hover_text("Delete").clicked() {
                    s.snapshot();
                    s.doc.clear(s.layer, &s.selection);
                    s.refresh = true;
                }
                ui.separator();
                ui.add(egui::DragValue::new(&mut s.new_size[0]).range(1..=crate::image_document::MAX_SIZE));
                ui.label("×");
                ui.add(egui::DragValue::new(&mut s.new_size[1]).range(1..=crate::image_document::MAX_SIZE));
                if ui.button("Новый").clicked() {
                    s.snapshot();
                    s.doc = ImageDocument::new(s.new_size[0], s.new_size[1]);
                    s.layer = 0;
                    s.selection.clear();
                    s.refresh = true;
                    s.fit = true;
                }
                if ui.button("Вписать").on_hover_text("Ctrl+0").clicked() {
                    s.fit = true;
                }
                ui.weak(format!("{:.0}%", s.zoom * 100.0));
            });
            ui.horizontal(|ui| {
                ui.label("Файл в assets");
                ui.text_edit_singleline(&mut s.path);
                if ui.button("Открыть").clicked() {
                    let result = (|| -> Result<ImageDocument, String> {
                        let bytes = self.assets.read(&s.path).map_err(|e| e.to_string())?;
                        if s.path.ends_with(".kimage") {
                            let d: ImageDocument = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
                            d.validate()?;
                            return Ok(d);
                        }
                        // Prefer the layered version saved next to a PNG.
                        let layered = s.path.rsplit_once('.').map(|(stem, _)| format!("{stem}.kimage"));
                        if let Some(d) = layered.and_then(|p| self.assets.read(&p).ok()).and_then(|b| serde_json::from_slice::<ImageDocument>(&b).ok()).filter(|d| d.validate().is_ok()) {
                            return Ok(d);
                        }
                        let image = image::load_from_memory(&bytes).map_err(|e| e.to_string())?.to_rgba8();
                        let max = crate::image_document::MAX_SIZE;
                        if image.width() > max || image.height() > max {
                            return Err(format!("Редактор поддерживает изображения до {max}×{max}; исходник не изменён."));
                        }
                        let mut d = ImageDocument::new(image.width(), image.height());
                        d.layers[0].pixels = image.into_raw();
                        Ok(d)
                    })();
                    match result {
                        Ok(doc) => {
                            s.snapshot();
                            s.doc = doc;
                            s.layer = 0;
                            s.selection.clear();
                            s.refresh = true;
                            s.fit = true;
                            s.status = "Открыто".into();
                        }
                        Err(e) => s.status = e,
                    }
                }
                if ui.button("Сохранить PNG + слои").clicked() {
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
                    match result {
                        Ok(()) => {
                            self.reload_assets();
                            self.renderer.refresh_changed();
                            s.status = "PNG и редактируемые слои .kimage сохранены.".into();
                        }
                        Err(e) => s.status = e,
                    }
                }
            });
            if s.refresh {
                let image = egui::ColorImage::from_rgba_unmultiplied([s.doc.width as usize, s.doc.height as usize], &s.doc.composite());
                let options = if s.zoom > 2.0 { egui::TextureOptions::NEAREST } else { egui::TextureOptions::LINEAR };
                match &mut s.texture {
                    Some(texture) => texture.set(image, options),
                    None => s.texture = Some(ctx.load_texture("image-editor-canvas", image, options)),
                }
                s.refresh = false;
            }
            ui.horizontal_top(|ui| {
                layers_panel(ui, &mut s);
                canvas(ui, &mut s);
            });
            ui.label(&s.status);
            ui.weak("Колесо — масштаб, средняя кнопка или Пробел — сдвиг. Alt+клик кистью — взять цвет. Ctrl+J — копия слоя, Ctrl+E — слить вниз. Давление пера работает, если система передаёт силу нажатия.");
        });
        s.open = open;
        self.image_editor = s;
    }
}

fn layers_panel(ui: &mut egui::Ui, s: &mut ImageEditorState) {
    ui.vertical(|ui| {
        ui.set_width(230.0);
        ui.heading("Слои");
        ui.horizontal_wrapped(|ui| {
            if ui.add_enabled(s.doc.layers.len() < MAX_LAYERS, egui::Button::new("+ Слой")).clicked() {
                s.snapshot();
                s.doc.add_layer();
                let last = s.doc.layers.len() - 1;
                let layer = s.doc.layers.remove(last);
                s.doc.layers.insert(s.layer + 1, layer);
                s.layer += 1;
                s.refresh = true;
            }
            if ui.add_enabled(s.doc.layers.len() < MAX_LAYERS, egui::Button::new("Копия")).on_hover_text("Ctrl+J").clicked() {
                s.snapshot();
                if s.doc.duplicate_layer(s.layer) {
                    s.layer += 1;
                }
                s.refresh = true;
            }
            if ui.add_enabled(s.layer > 0, egui::Button::new("Слить вниз")).on_hover_text("Ctrl+E").clicked() {
                s.snapshot();
                s.doc.merge_down(s.layer);
                s.layer -= 1;
                s.refresh = true;
            }
            if ui.add_enabled(s.doc.layers.len() > 1, egui::Button::new("Удалить")).clicked() {
                s.snapshot();
                s.doc.layers.remove(s.layer);
                s.layer = s.layer.min(s.doc.layers.len() - 1);
                s.refresh = true;
            }
        });
        egui::ScrollArea::vertical().max_height(420.0).show(ui, |ui| {
            for i in (0..s.doc.layers.len()).rev() {
                ui.push_id(i, |ui| {
                    let selected = s.layer == i;
                    egui::Frame::NONE.fill(if selected { theme::pal().selection } else { egui::Color32::TRANSPARENT }).corner_radius(6.0).inner_margin(4.0).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            s.refresh |= ui.checkbox(&mut s.doc.layers[i].visible, "").on_hover_text("Видимость").changed();
                            if ui.selectable_label(selected, &s.doc.layers[i].name).clicked() {
                                s.layer = i;
                            }
                        });
                        if selected {
                            ui.text_edit_singleline(&mut s.doc.layers[i].name);
                            s.refresh |= ui.add(egui::Slider::new(&mut s.doc.layers[i].opacity, 0.0..=1.0).text("Непрозр.")).changed();
                            let mut blend = s.doc.layers[i].blend;
                            egui::ComboBox::from_id_salt("blend").selected_text(blend.name()).show_ui(ui, |ui| {
                                for b in Blend::ALL {
                                    ui.selectable_value(&mut blend, b, b.name());
                                }
                            });
                            if blend != s.doc.layers[i].blend {
                                s.doc.layers[i].blend = blend;
                                s.refresh = true;
                            }
                            ui.checkbox(&mut s.doc.layers[i].lock_alpha, "Закрепить прозрачность");
                        }
                    });
                });
            }
        });
        ui.horizontal(|ui| {
            if ui.add_enabled(s.layer + 1 < s.doc.layers.len(), egui::Button::new("Выше")).clicked() {
                s.snapshot();
                s.doc.layers.swap(s.layer, s.layer + 1);
                s.layer += 1;
                s.refresh = true;
            }
            if ui.add_enabled(s.layer > 0, egui::Button::new("Ниже")).clicked() {
                s.snapshot();
                s.doc.layers.swap(s.layer, s.layer - 1);
                s.layer -= 1;
                s.refresh = true;
            }
        });
    });
}

fn canvas(ui: &mut egui::Ui, s: &mut ImageEditorState) {
    let (area, response) = ui.allocate_exact_size(ui.available_size().max(egui::vec2(200.0, 200.0)), egui::Sense::click_and_drag());
    let painter = ui.painter_at(area);
    painter.rect_filled(area, 6.0, theme::pal().field);
    let doc_size = egui::vec2(s.doc.width as f32, s.doc.height as f32);
    if s.fit {
        s.fit = false;
        s.zoom = ((area.width() - 20.0) / doc_size.x).min((area.height() - 20.0) / doc_size.y).clamp(0.02, 32.0);
        s.pan = egui::Vec2::ZERO;
    }
    let hover = ui.input(|i| i.pointer.hover_pos()).filter(|p| area.contains(*p));
    // Zoom around the pointer, pan with the middle button or Space.
    if let Some(p) = hover {
        let scroll = ui.input(|i| i.smooth_scroll_delta.y);
        if scroll != 0.0 {
            let old = s.zoom;
            s.zoom = (old * (scroll * 0.002).exp()).clamp(0.02, 32.0);
            let center = area.center() + s.pan;
            s.pan += (p - center) * (1.0 - s.zoom / old);
            s.refresh = true;
        }
    }
    let panning = response.dragged_by(egui::PointerButton::Middle) || (response.dragged_by(egui::PointerButton::Primary) && ui.input(|i| i.key_down(Key::Space)));
    if panning {
        s.pan += ui.input(|i| i.pointer.delta());
    }
    let rect = egui::Rect::from_center_size(area.center() + s.pan, doc_size * s.zoom);
    // Checkerboard makes alpha and erasing visible.
    let cell = 12.0;
    let visible = rect.intersect(area);
    let (cx0, cy0) = (((visible.left() - rect.left()) / cell).floor() as i32, ((visible.top() - rect.top()) / cell).floor() as i32);
    let (cx1, cy1) = (((visible.right() - rect.left()) / cell).ceil() as i32, ((visible.bottom() - rect.top()) / cell).ceil() as i32);
    if (cx1 - cx0) * (cy1 - cy0) < 40_000 {
        for y in cy0..cy1 {
            for x in cx0..cx1 {
                let c = egui::Rect::from_min_size(rect.min + egui::vec2(x as f32, y as f32) * cell, egui::vec2(cell, cell)).intersect(rect);
                painter.rect_filled(c, 0.0, egui::Color32::from_gray(if (x + y) % 2 == 0 { 50 } else { 70 }));
            }
        }
    }
    if let Some(texture) = &s.texture {
        painter.image(texture.id(), rect, egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)), egui::Color32::WHITE);
    }
    painter.rect_stroke(rect, 0.0, egui::Stroke::new(1.0, theme::pal().border_strong), egui::StrokeKind::Outside);
    let zoom = s.zoom;
    let to_doc = move |p: egui::Pos2| [(p.x - rect.left()) / zoom, (p.y - rect.top()) / zoom];
    let position = response.interact_pointer_pos().map(to_doc);
    let alt = ui.input(|i| i.modifiers.alt);
    let tool = if alt && matches!(s.tool, Tool::Brush | Tool::Bucket | Tool::Gradient) { Tool::Picker } else { s.tool };
    if !panning {
        if response.drag_started_by(egui::PointerButton::Primary) || response.clicked() {
            s.last = None;
            match tool {
                Tool::Brush | Tool::Erase | Tool::Gradient => s.snapshot(),
                Tool::Lasso => s.selection.clear(),
                Tool::Rectangle => s.rect_start = position,
                _ => {}
            }
            if tool == Tool::Gradient {
                s.rect_start = position;
            }
            if let Some(p) = position {
                match tool {
                    Tool::Bucket => {
                        s.snapshot();
                        s.doc.flood_fill(s.layer, p, s.color, s.tolerance, &s.selection);
                        s.refresh = true;
                    }
                    Tool::Picker => {
                        if let Some(c) = s.doc.sample(p) {
                            s.color = c;
                        }
                    }
                    _ => {}
                }
            }
        }
        if (response.dragged_by(egui::PointerButton::Primary) || response.clicked()) && matches!(tool, Tool::Brush | Tool::Erase | Tool::Lasso | Tool::Rectangle | Tool::Picker) {
            if let Some(p) = position {
                match tool {
                    Tool::Brush | Tool::Erase => {
                        let pressure = ui
                            .input(|i| i.events.iter().rev().find_map(|e| match e {
                                egui::Event::Touch { force: Some(f), .. } => Some(*f),
                                _ => None,
                            }))
                            .unwrap_or(1.0)
                            .clamp(0.05, 1.0);
                        let radius = s.radius * if s.pressure_size { pressure } else { 1.0 };
                        let strength = s.opacity * s.flow * if s.pressure_opacity { pressure } else { 1.0 };
                        let previous = s.last.unwrap_or(p);
                        let distance = ((p[0] - previous[0]).powi(2) + (p[1] - previous[1]).powi(2)).sqrt();
                        let step = (s.radius * s.spacing).max(0.5);
                        let steps = (distance / step).ceil().max(1.0) as usize;
                        for i in 0..=steps.min(8192) {
                            if s.last.is_some() && i == 0 {
                                continue;
                            }
                            let t = i as f32 / steps.max(1) as f32;
                            let center = [previous[0] + (p[0] - previous[0]) * t, previous[1] + (p[1] - previous[1]) * t];
                            s.doc.dab(s.layer, Dab { center, radius, color: s.color, strength, hardness: s.hardness, erase: tool == Tool::Erase }, &s.selection);
                        }
                        s.last = Some(p);
                        s.refresh = true;
                    }
                    Tool::Lasso => {
                        if s.selection.len() < 8192 {
                            s.selection.push(p);
                        }
                    }
                    Tool::Rectangle => {
                        if let Some(a) = s.rect_start {
                            s.selection = vec![a, [p[0], a[1]], p, [a[0], p[1]]];
                        }
                    }
                    Tool::Picker => {
                        if let Some(c) = s.doc.sample(p) {
                            s.color = c;
                        }
                    }
                    _ => {}
                }
            }
        }
        // Gradient: drawn as a line while dragging, applied on release.
        if tool == Tool::Gradient {
            if let (Some(a), Some(p)) = (s.rect_start, position) {
                let (sa, sb) = (rect.min + egui::vec2(a[0], a[1]) * s.zoom, rect.min + egui::vec2(p[0], p[1]) * s.zoom);
                painter.line_segment([sa, sb], egui::Stroke::new(2.0, theme::ACCENT));
                if response.drag_stopped() {
                    s.doc.gradient(s.layer, a, p, s.color, s.secondary, &s.selection);
                    s.refresh = true;
                }
            }
        }
    }
    if response.drag_stopped() {
        s.last = None;
        s.rect_start = None;
    }
    if s.selection.len() >= 2 {
        let mut points: Vec<_> = s.selection.iter().map(|p| rect.min + egui::vec2(p[0], p[1]) * s.zoom).collect();
        points.push(points[0]);
        painter.add(egui::Shape::dashed_line(&points, egui::Stroke::new(1.5, theme::ACCENT), 6.0, 4.0));
    }
    if let Some(p) = hover {
        if matches!(tool, Tool::Brush | Tool::Erase) {
            painter.circle_stroke(p, s.radius * s.zoom, egui::Stroke::new(1.0, egui::Color32::WHITE));
            painter.circle_stroke(p, s.radius * s.zoom + 1.0, egui::Stroke::new(1.0, egui::Color32::from_black_alpha(160)));
        }
        let d = to_doc(p);
        if d[0] >= 0.0 && d[1] >= 0.0 && d[0] < s.doc.width as f32 && d[1] < s.doc.height as f32 {
            painter.text(area.left_bottom() + egui::vec2(8.0, -8.0), egui::Align2::LEFT_BOTTOM, format!("{} , {}", d[0] as i32, d[1] as i32), egui::FontId::monospace(11.0), theme::pal().text_dim);
        }
    }
}
