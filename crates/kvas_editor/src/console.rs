use crate::i18n::t;
use eframe::egui;
use std::sync::mpsc::{channel, Receiver, Sender};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Info,
    Warn,
    Error,
}

pub struct Console {
    pub lines: Vec<(Level, String)>,
    tx: Sender<(Level, String)>,
    rx: Receiver<(Level, String)>,
    show: [bool; 3],
    scroll_to_end: bool,
}

impl Default for Console {
    fn default() -> Self {
        let (tx, rx) = channel();
        Self { lines: Vec::new(), tx, rx, show: [true; 3], scroll_to_end: true }
    }
}

impl Console {
    /// For background threads (builds).
    pub fn sender(&self) -> Sender<(Level, String)> {
        self.tx.clone()
    }

    pub fn log(&mut self, level: Level, msg: impl Into<String>) {
        self.lines.push((level, msg.into()));
        if self.lines.len() > 2000 {
            self.lines.drain(..500);
        }
    }
    pub fn info(&mut self, msg: impl Into<String>) {
        self.log(Level::Info, msg);
    }
    pub fn warn(&mut self, msg: impl Into<String>) {
        self.log(Level::Warn, msg);
    }
    pub fn error(&mut self, msg: impl Into<String>) {
        self.log(Level::Error, msg);
    }

    pub fn pump(&mut self) {
        while let Ok((level, msg)) = self.rx.try_recv() {
            self.log(level, msg);
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button(t("Clear")).clicked() {
                self.lines.clear();
            }
            let count = |l: Level| self.lines.iter().filter(|(x, _)| *x == l).count();
            let counts = [count(Level::Info), count(Level::Warn), count(Level::Error)];
            for (i, label) in ["Info", "Warnings", "Errors"].iter().enumerate() {
                ui.toggle_value(&mut self.show[i], format!("{} {}", counts[i], t(label)));
            }
            ui.checkbox(&mut self.scroll_to_end, t("Auto-scroll"));
        });
        ui.separator();
        egui::ScrollArea::vertical().auto_shrink(false).stick_to_bottom(self.scroll_to_end).show(ui, |ui| {
            for (level, msg) in &self.lines {
                let (idx, color) = match level {
                    Level::Info => (0, egui::Color32::from_gray(200)),
                    Level::Warn => (1, egui::Color32::from_rgb(230, 190, 80)),
                    Level::Error => (2, egui::Color32::from_rgb(240, 100, 100)),
                };
                if self.show[idx] {
                    ui.add(egui::Label::new(egui::RichText::new(msg).color(color).monospace()).wrap());
                }
            }
        });
    }
}
