//! Sound panel: browse the project's clips, see their waveform, audition them with volume, pitch
//! and stereo position, and attach them to objects.

use crate::editor::Editor;
use crate::i18n::t;
use crate::icons::{self, Icon};
use crate::theme::{self, pal};
use eframe::egui::{self, Color32, RichText, Sense};
use kvas_audio::ClipInfo;
use kvas_engine::audio::SoundId;
use kvas_engine::prelude::*;
use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Instant;

pub struct AudioPanel {
    selected: Option<String>,
    infos: HashMap<String, Option<ClipInfo>>,
    pending: Vec<String>,
    tx: Sender<(String, Option<ClipInfo>)>,
    rx: Receiver<(String, Option<ClipInfo>)>,
    volume: f32,
    pitch: f32,
    pan: f32,
    looped: bool,
    playing: Option<(SoundId, Instant)>,
    search: String,
    edit: kvas_audio::edit::ClipEdit,
    edit_job: Option<Receiver<Result<String, String>>>,
}

impl Default for AudioPanel {
    fn default() -> Self {
        let (tx, rx) = channel();
        Self { selected: None, infos: HashMap::new(), pending: Vec::new(), tx, rx, volume: 1.0, pitch: 1.0, pan: 0.0, looped: false, playing: None, search: String::new(), edit: Default::default(), edit_job: None }
    }
}

fn is_audio(path: &str) -> bool {
    matches!(path.rsplit('.').next().map(str::to_ascii_lowercase).as_deref(), Some("wav" | "ogg" | "mp3" | "flac" | "m4a"))
}

fn clock(seconds: f32) -> String {
    let s = seconds.max(0.0) as u32;
    format!("{}:{:02}", s / 60, s % 60)
}

impl Editor {
    fn request_clip_info(&mut self, clip: &str) {
        if self.sound.infos.contains_key(clip) || self.sound.pending.iter().any(|c| c == clip) {
            return;
        }
        self.sound.pending.push(clip.to_string());
        let (assets, tx, name) = (self.assets.clone(), self.sound.tx.clone(), clip.to_string());
        std::thread::spawn(move || {
            let info = kvas_audio::analyze_asset(&assets, &name, 600);
            let _ = tx.send((name, info));
        });
    }

    fn stop_preview(&mut self) {
        if let Some((id, _)) = self.sound.playing.take() {
            self.scene.audio.stop(id);
            if let Some(a) = &mut self.audio {
                a.update_requests(&mut self.scene, &self.assets);
            }
        }
    }

    fn start_preview(&mut self, clip: &str) {
        self.stop_preview();
        if self.audio.is_none() {
            self.audio = kvas_audio::AudioEngine::new();
        }
        let p = &self.sound;
        let id = self.scene.audio.play_with(clip, p.volume, p.pitch, p.pan, p.looped);
        if let Some(a) = &mut self.audio {
            a.update_requests(&mut self.scene, &self.assets);
        }
        self.sound.playing = Some((id, Instant::now()));
    }

    pub fn audio_ui(&mut self, ui: &mut egui::Ui) {
        if let Some(result) = self.sound.edit_job.as_ref().and_then(|rx| rx.try_recv().ok()) {
            self.sound.edit_job = None;
            match result {
                Ok(path) => { self.sound.selected = Some(path.clone()); self.sound.edit = Default::default(); self.reload_assets(); self.console.info(format!("WAV сохранён: {path}")); }
                Err(e) => self.console.error(e),
            }
        }
        while let Ok((clip, info)) = self.sound.rx.try_recv() {
            self.sound.pending.retain(|c| *c != clip);
            self.sound.infos.insert(clip, info);
        }
        let clips: Vec<String> = {
            let q = self.sound.search.to_lowercase();
            self.asset_list.iter().filter(|a| is_audio(a) && (q.is_empty() || a.to_lowercase().contains(&q))).cloned().collect()
        };
        let p = pal();

        // Toolbar: master volume and mute for the whole editor preview / game.
        ui.horizontal(|ui| {
            let muted = self.scene.audio.muted;
            if icons::button(ui, Icon::Volume, t("Mute all sound"), muted).clicked() {
                self.scene.audio.muted = !muted;
                if let Some(play) = &mut self.play {
                    play.runtime.scene.audio.muted = !muted;
                }
            }
            ui.label(RichText::new(t("Master")).color(p.text_dim));
            let mut master = self.scene.audio.master_volume;
            if ui.add(egui::Slider::new(&mut master, 0.0..=1.0).show_value(false)).changed() {
                self.scene.audio.master_volume = master;
                if let Some(play) = &mut self.play {
                    play.runtime.scene.audio.master_volume = master;
                }
            }
            ui.add(egui::TextEdit::singleline(&mut self.sound.search).hint_text(t("Search")).desired_width(140.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.weak(format!("{} {}", clips.len(), t("clips")));
            });
        });
        ui.separator();

        egui::Panel::left("audio-list").resizable(true).default_size(210.0).show(ui, |ui| {
            egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                if clips.is_empty() {
                    ui.add_space(12.0);
                    ui.weak(t("No sounds yet. Import wav, ogg, mp3, flac or m4a files in the Project panel."));
                }
                for clip in &clips {
                    let selected = self.sound.selected.as_deref() == Some(clip.as_str());
                    let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 24.0), Sense::click());
                    crate::ui_kit::tile(ui, rect, resp.id, resp.hovered(), selected);
                    ui.painter().text(rect.left_center() + egui::vec2(26.0, 0.0), egui::Align2::LEFT_CENTER, clip.rsplit('/').next().unwrap_or(clip), egui::FontId::proportional(12.5), p.text);
                    icons::Icon::Music.image(14.0, p.text_dim).paint_at(ui, egui::Rect::from_center_size(rect.left_center() + egui::vec2(12.0, 0.0), egui::Vec2::splat(14.0)));
                    if resp.on_hover_text(clip).clicked() {
                        self.sound.selected = Some(clip.clone());
                    }
                }
            });
        });

        let Some(clip) = self.sound.selected.clone().filter(|c| clips.contains(c)) else {
            ui.add_space(20.0);
            ui.vertical_centered(|ui| ui.weak(t("Select a sound on the left.")));
            return;
        };
        self.request_clip_info(&clip);
        let info = self.sound.infos.get(&clip).cloned().flatten();
        ui.vertical(|ui| {
            ui.label(RichText::new(clip.rsplit('/').next().unwrap_or(&clip)).family(theme::bold_family()).size(15.0));
            match (&info, self.sound.pending.contains(&clip)) {
                (Some(i), _) => {
                    let kind = if i.channels >= 2 { t("stereo") } else { t("mono") };
                    ui.weak(format!("{}   {}   {} Hz", clock(i.seconds), kind, i.sample_rate));
                }
                (None, true) => {
                    ui.weak(t("Analyzing..."));
                }
                (None, false) => {
                    ui.colored_label(theme::red(), t("This file cannot be decoded."));
                }
            }
            // Waveform.
            let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 84.0), Sense::click());
            ui.painter().rect_filled(rect, 8.0, p.field);
            if let Some(i) = &info {
                let mid = rect.center().y;
                let half = rect.height() * 0.5 - 6.0;
                let n = i.peaks.len();
                let w = (rect.width() - 8.0) / n as f32;
                for (k, peak) in i.peaks.iter().enumerate() {
                    let x = rect.left() + 4.0 + k as f32 * w;
                    let h = (peak.sqrt() * half).max(1.0);
                    ui.painter().line_segment([egui::pos2(x, mid - h), egui::pos2(x, mid + h)], egui::Stroke::new(w.max(1.0), theme::ACCENT.gamma_multiply(0.85)));
                }
                if let Some((_, started)) = self.sound.playing {
                    let len = (i.seconds / self.sound.pitch.max(0.05)).max(0.01);
                    let t_play = started.elapsed().as_secs_f32();
                    if self.sound.looped || t_play < len {
                        let frac = (t_play % len) / len;
                        let x = rect.left() + 4.0 + frac * (rect.width() - 8.0);
                        ui.painter().line_segment([egui::pos2(x, rect.top() + 3.0), egui::pos2(x, rect.bottom() - 3.0)], egui::Stroke::new(1.5, Color32::WHITE));
                        ui.ctx().request_repaint();
                    } else {
                        self.sound.playing = None;
                    }
                }
            }
            if resp.clicked() {
                self.start_preview(&clip);
            }
            ui.horizontal(|ui| {
                let playing = self.sound.playing.is_some();
                if playing {
                    if icons::button(ui, Icon::Square, t("Stop"), true).clicked() {
                        self.stop_preview();
                    }
                } else if icons::button(ui, Icon::Play, t("Play"), false).clicked() {
                    self.start_preview(&clip);
                }
                ui.checkbox(&mut self.sound.looped, t("Loop"));
                ui.separator();
                ui.label(t("Volume"));
                ui.add(egui::Slider::new(&mut self.sound.volume, 0.0..=1.5).show_value(false));
                ui.label(t("Pitch"));
                ui.add(egui::Slider::new(&mut self.sound.pitch, 0.25..=3.0).show_value(false));
                ui.label(t("Pan"));
                ui.add(egui::Slider::new(&mut self.sound.pan, -1.0..=1.0).show_value(false)).on_hover_text(t("Left ... right"));
            });
            ui.collapsing("Редактор клипа — обрезка, фейды, нормализация", |ui| {
                let edit = &mut self.sound.edit;
                egui::Grid::new("clip-edit-fields").num_columns(2).show(ui, |ui| {
                    for (name, value) in [("Начало (с)", &mut edit.start), ("Конец (с), 0 = весь", &mut edit.end), ("Fade in (с)", &mut edit.fade_in), ("Fade out (с)", &mut edit.fade_out), ("Усиление", &mut edit.gain)] {
                        ui.label(name); ui.add(egui::DragValue::new(value).speed(0.01).range(0.0..=36000.0)); ui.end_row();
                    }
                });
                ui.checkbox(&mut edit.normalize, "Нормализация пика до 98%");
                if ui.add_enabled(self.sound.edit_job.is_none(), egui::Button::new("Экспортировать новый WAV")).clicked() {
                    let (tx, rx) = channel(); self.sound.edit_job = Some(rx);
                    let assets = self.assets.clone(); let edit = self.sound.edit;
                    let source = clip.clone(); let root = self.project.assets_dir();
                    std::thread::spawn(move || {
                        let result = (|| -> Result<String, String> {
                            let wav = kvas_audio::edit::edit_asset(&assets, &source, edit)?;
                            let stem: String = std::path::Path::new(&source).file_stem().unwrap_or_default().to_string_lossy().chars().filter(|c| c.is_alphanumeric() || *c == '_').take(48).collect();
                            let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_err(|e| e.to_string())?.as_nanos();
                            let path = format!("sounds/edits/{stem}_{stamp}.wav");
                            std::fs::create_dir_all(root.join("sounds/edits")).map_err(|e| e.to_string())?;
                            std::fs::write(root.join(&path), wav).map_err(|e| e.to_string())?;
                            Ok(path)
                        })();
                        let _ = tx.send(result);
                    });
                }
                if self.sound.edit_job.is_some() { ui.spinner(); }
                ui.weak("Исходник сохраняется. Новый клип появится в списке; его можно прослушать и прикрепить к объекту.");
            });
            ui.horizontal(|ui| {
                let has_sel = self.selection.is_some();
                if ui.add_enabled(has_sel, egui::Button::new(t("Attach to selected object"))).on_hover_text(t("Adds an Audio Source component that plays this clip")).clicked() {
                    self.attach_sound(&clip);
                }
                if ui.button(t("Create sound object")).clicked() {
                    self.create_sound_object(&clip);
                }
            });
        });
    }

    fn attach_sound(&mut self, clip: &str) {
        let Some(e) = self.selection else { return };
        self.begin_edit();
        let world = &mut self.active_mut().world;
        let source = AudioSource { clip: clip.to_string(), ..Default::default() };
        world.insert(e, source);
        self.status = crate::i18n::tf("Attached {}", &[clip]);
    }

    fn create_sound_object(&mut self, clip: &str) {
        let name = clip.rsplit('/').next().unwrap_or(clip).rsplit_once('.').map_or(clip, |(n, _)| n).to_string();
        let source = AudioSource { clip: clip.to_string(), ..Default::default() };
        let e = self.create_entity(&name, move |w, e| w.insert(e, source));
        self.selection = Some(e);
    }
}
