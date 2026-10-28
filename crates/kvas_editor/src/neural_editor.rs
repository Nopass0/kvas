//! Bounded CPU supervised training, live loss curve, portable weights and explicit application.
use crate::{editor::Editor, theme};
use eframe::egui;
use kvas_engine::neural::{Network, Example, NeuralAgent};
use std::sync::{mpsc::{self, Receiver}, Arc, atomic::{AtomicBool, Ordering}};

enum TrainingEvent { Progress(usize, f32), Done(Result<Network, String>) }
pub struct NeuralTraining {
    pub open: bool,
    epochs: usize,
    rate: f32,
    dataset: String,
    path: String,
    task: Option<Receiver<TrainingEvent>>,
    cancel: Arc<AtomicBool>,
    model: Option<Network>,
    losses: Vec<(usize, f32)>,
    error: String,
}
impl Default for NeuralTraining {
    fn default() -> Self { Self { open: false, epochs: 500, rate: 0.02, dataset: String::new(), path: "ai/steering.weights.json".into(), task: None, cancel: Arc::new(AtomicBool::new(false)), model: None, losses: Vec::new(), error: String::new() } }
}

impl Editor {
    pub fn neural_training_window(&mut self, ctx: &egui::Context) {
        let mut events = Vec::new();
        if let Some(rx) = &self.training.task { while let Ok(event) = rx.try_recv() { events.push(event); } }
        for event in events {
            match event {
                TrainingEvent::Progress(epoch, loss) => self.training.losses.push((epoch, loss)),
                TrainingEvent::Done(result) => {
                    self.training.task = None;
                    match result { Ok(model) => { self.training.model = Some(model); self.console.info("CPU обучение завершено; веса готовы к сохранению и применению."); }, Err(e) => self.training.error = e }
                }
            }
        }
        if !self.training.open { return; }
        let mut open = true;
        egui::Window::new("AI — обучение поведения (CPU)").open(&mut open).default_size([660.0, 540.0]).show(ctx, |ui| {
            ui.weak("Supervised MLP: 2 входа (смещение X/Z), 8 tanh нейронов, 2 выхода (движение X/Z). Это обучение по примерам, без RL и поиска пути.");
            let busy = self.training.task.is_some();
            ui.add_enabled_ui(!busy, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Эпохи"); ui.add(egui::DragValue::new(&mut self.training.epochs).range(1..=10000));
                    ui.label("Learning rate"); ui.add(egui::DragValue::new(&mut self.training.rate).speed(0.001).range(0.0001..=0.2));
                });
                ui.label("JSON dataset в assets (пусто = встроенные 25 примеров steering)");
                ui.text_edit_singleline(&mut self.training.dataset);
                if ui.button("Обучить с seed 42").clicked() { self.start_neural_training(); }
            });
            if busy {
                let fraction = self.training.losses.last().map_or(0.0, |(n, _)| *n as f32 / self.training.epochs as f32);
                ui.add(egui::ProgressBar::new(fraction).show_percentage());
                if ui.button("Отменить обучение").clicked() { self.training.cancel.store(true, Ordering::Relaxed); }
                ctx.request_repaint();
            }
            if let Some((epoch, loss)) = self.training.losses.last() { ui.monospace(format!("Epoch {epoch}   MSE {loss:.6}")); }
            let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 120.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, 6.0, theme::pal().field);
            let max_loss = self.training.losses.iter().map(|(_, v)| *v).fold(0.00001, f32::max);
            let points: Vec<_> = self.training.losses.iter().map(|(n, v)| egui::pos2(rect.left() + *n as f32 / self.training.epochs as f32 * rect.width(), rect.bottom() - (*v / max_loss).clamp(0.0, 1.0) * rect.height())).collect();
            if points.len() > 1 { ui.painter().add(egui::Shape::line(points, egui::Stroke::new(2.0, theme::ACCENT))); }
            ui.separator(); ui.label("Файл весов в assets"); ui.text_edit_singleline(&mut self.training.path);
            ui.horizontal(|ui| {
                if ui.add_enabled(self.training.model.is_some(), egui::Button::new("Сохранить веса")).clicked() {
                    let result = (|| -> Result<(), String> {
                        let rel = kvas_engine::assets::normalize_path(&self.training.path).map_err(|e| e.to_string())?;
                        let path = self.project.assets_dir().join(&rel);
                        std::fs::create_dir_all(path.parent().ok_or("invalid path")?).map_err(|e| e.to_string())?;
                        self.training.model.as_ref().unwrap().save(&path)?;
                        Ok(())
                    })();
                    match result { Ok(()) => { self.reload_assets(); self.console.info(format!("Веса сохранены: {}", self.training.path)); }, Err(e) => self.training.error = e }
                }
                if ui.button("Загрузить веса").clicked() {
                    match self.assets.read_string(&self.training.path).map_err(|e| e.to_string()).and_then(|s| Network::from_json(&s)) {
                        Ok(model) => { self.training.model = Some(model); self.training.error.clear(); }, Err(e) => self.training.error = e,
                    }
                }
                if ui.add_enabled(self.training.model.is_some() && self.selection.is_some() && !self.is_playing(), egui::Button::new("Применить к объекту")).clicked() {
                    let model = self.training.model.as_ref().unwrap().clone();
                    if model.inputs != 2 || model.outputs != 2 { self.training.error = "NeuralAgent ожидает модель 2 → N → 2".into(); }
                    else if let Some(e) = self.selection {
                        self.begin_edit();
                        let world = &mut self.active_mut().world;
                        if !world.has::<NeuralAgent>(e) { world.insert(e, NeuralAgent::default()); }
                        world.get_mut::<NeuralAgent>(e).unwrap().network = model;
                        self.status = "Веса применены. Укажите target и enabled в NeuralAgent, затем запустите Play.".into();
                    }
                }
            });
            if !self.training.error.is_empty() { ui.colored_label(theme::red(), &self.training.error); }
            ui.collapsing("Формат dataset и ограничения", |ui| {
                ui.monospace("[{\"input\":[-1,0],\"target\":[-1,0]}, {\"input\":[1,0],\"target\":[1,0]}]");
                ui.label("Нормализуйте данные в -1..1. Лимит работы 200 млн операций. Веса в JSON сохраняются также в сцене. NeuralAgent управляет Transform без RigidBody; коллизии и навигацию нужно реализовать отдельно.");
            });
        });
        self.training.open = open;
    }

    fn start_neural_training(&mut self) {
        let data: Result<Vec<Example>, String> = if self.training.dataset.trim().is_empty() { Ok(kvas_engine::neural::steering_examples()) }
            else { self.assets.read_string(&self.training.dataset).map_err(|e| e.to_string()).and_then(|s| serde_json::from_str(&s).map_err(|e| e.to_string())) };
        let data = match data { Ok(d) => d, Err(e) => { self.training.error = e; return; } };
        let epochs = self.training.epochs;
        if data.len().saturating_mul(epochs).saturating_mul(32) > 200_000_000 { self.training.error = "Слишком большой dataset × epochs; уменьшите объём обучения.".into(); return; }
        self.training.error.clear(); self.training.losses.clear();
        self.training.cancel = Arc::new(AtomicBool::new(false));
        let cancel = self.training.cancel.clone(); let rate = self.training.rate;
        let (tx, rx) = mpsc::channel(); self.training.task = Some(rx);
        std::thread::spawn(move || {
            let result = (|| {
                let mut model = Network::new(2, 8, 2, 42)?;
                for epoch in 1..=epochs {
                    if cancel.load(Ordering::Relaxed) { return Err("Обучение отменено; предыдущие веса сохранены.".into()); }
                    let loss = model.train_epoch(&data, rate)?;
                    if epoch == 1 || epoch % 10 == 0 || epoch == epochs { let _ = tx.send(TrainingEvent::Progress(epoch, loss)); }
                }
                Ok(model)
            })();
            let _ = tx.send(TrainingEvent::Done(result));
        });
    }
}
