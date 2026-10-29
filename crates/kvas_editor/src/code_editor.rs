//! Built-in code editor: tabs, syntax colours, line numbers, find / replace, go to line,
//! autocomplete hints for the Kvas API and live saving (scripts rebuild, shaders reload).

use crate::editor::Editor;
use crate::i18n::t;
use crate::theme::{self, pal};
use crate::wgsl_edit::{self, Lang};
use eframe::egui::{self, text::CCursor, text_edit::TextEditState, Color32, FontId, Key, Modifiers, RichText};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Instant, SystemTime};

static REQUESTS: Mutex<Vec<(PathBuf, usize)>> = Mutex::new(Vec::new());

/// Asks the built-in editor to open a file at a line (callable from anywhere).
pub fn request(path: &Path, line: usize) {
    if let Ok(mut r) = REQUESTS.lock() {
        r.push((path.to_path_buf(), line));
    }
}

struct Doc {
    path: PathBuf,
    text: String,
    saved: String,
    stamp: Option<SystemTime>,
    jump: Option<usize>,
    lang: Lang,
}

impl Doc {
    fn dirty(&self) -> bool {
        self.text != self.saved
    }
    fn name(&self) -> String {
        self.path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()
    }
}

#[derive(Default)]
pub struct CodeEditor {
    pub open: bool,
    docs: Vec<Doc>,
    active: usize,
    show_find: bool,
    find: String,
    replace: String,
    case: bool,
    goto: Option<String>,
    focus_find: bool,
    last_check: Option<Instant>,
    /// Pending cursor selection (char range) for the next frame (find / jump).
    select: Option<(usize, usize)>,
    cursor_line: usize,
    cursor_col: usize,
    cursor_idx: usize,
}

fn stamp(p: &Path) -> Option<SystemTime> {
    std::fs::metadata(p).and_then(|m| m.modified()).ok()
}

fn matches(text: &str, needle: &str, case: bool) -> Vec<usize> {
    if needle.is_empty() {
        return Vec::new();
    }
    let (hay, pat) = if case { (text.to_string(), needle.to_string()) } else { (text.to_lowercase(), needle.to_lowercase()) };
    if hay.len() != text.len() {
        return Vec::new(); // case folding changed byte lengths; skip rather than misplace
    }
    hay.match_indices(&pat).map(|(i, _)| i).collect()
}

fn byte_to_char(text: &str, b: usize) -> usize {
    text[..b.min(text.len())].chars().count()
}

impl CodeEditor {
    pub fn open_file(&mut self, path: &Path, line: usize) {
        self.open = true;
        if let Some(i) = self.docs.iter().position(|d| d.path == path) {
            self.active = i;
            self.docs[i].jump = Some(line);
            return;
        }
        match std::fs::read_to_string(path) {
            Ok(text) => {
                self.docs.push(Doc { path: path.to_path_buf(), saved: text.clone(), text, stamp: stamp(path), jump: Some(line), lang: Lang::of(&path.to_string_lossy()) });
                self.active = self.docs.len() - 1;
            }
            Err(_) => crate::assets_panel::open_external(path),
        }
    }

    fn save(&mut self, i: usize) -> bool {
        let d = &mut self.docs[i];
        match std::fs::write(&d.path, &d.text) {
            Ok(()) => {
                d.saved = d.text.clone();
                d.stamp = stamp(&d.path);
                true
            }
            Err(_) => false,
        }
    }
}

impl Editor {
    /// The word under the cursor in the open Rust/WGSL file (for F1 help), if the editor is open.
    pub fn code_word(&self) -> Option<String> {
        let ce = &self.code;
        let d = ce.docs.get(ce.active).filter(|_| ce.open)?;
        let byte = d.text.char_indices().nth(ce.cursor_idx).map_or(d.text.len(), |(b, _)| b);
        let end = d.text[byte..].char_indices().find(|(_, c)| !(c.is_alphanumeric() || *c == '_')).map_or(d.text.len(), |(i, _)| byte + i);
        let (_, word) = word_at(&d.text, end);
        (!word.is_empty()).then_some(word)
    }

    pub fn code_editor_ui(&mut self, ctx: &egui::Context) {
        // Requests from "open file" actions anywhere in the editor.
        let pending: Vec<(PathBuf, usize)> = REQUESTS.lock().map(|mut r| std::mem::take(&mut *r)).unwrap_or_default();
        for (p, line) in pending {
            self.code.open_file(&p, line);
        }
        if !self.code.open {
            return;
        }
        // Pick up changes made outside (e.g. by another editor) when the tab has no edits.
        if self.code.last_check.is_none_or(|t| t.elapsed().as_millis() > 700) {
            self.code.last_check = Some(Instant::now());
            for d in &mut self.code.docs {
                let now = stamp(&d.path);
                if now != d.stamp && !d.dirty() {
                    if let Ok(text) = std::fs::read_to_string(&d.path) {
                        d.saved = text.clone();
                        d.text = text;
                    }
                    d.stamp = now;
                }
            }
        }
        let mut open = true;
        let mut saved_path: Option<PathBuf> = None;
        let mut close_tab: Option<usize> = None;
        let ce = &mut self.code;
        let api = &mut self.api;
        let project_root = self.project.root.clone();
        egui::Window::new(t("Code editor")).id(egui::Id::unique("kvas-code-editor")).open(&mut open).default_size([920.0, 620.0]).collapsible(false).show(ctx, |ui| {
            if ce.docs.is_empty() {
                ui.centered_and_justified(|ui| ui.weak(t("Double-click a script or shader to open it here.")));
                return;
            }
            ce.active = ce.active.min(ce.docs.len() - 1);
            // Shortcuts.
            let (save, find, goto) = ui.input_mut(|i| (i.consume_key(Modifiers::COMMAND, Key::S), i.consume_key(Modifiers::COMMAND, Key::F), i.consume_key(Modifiers::COMMAND, Key::G)));
            if find {
                ce.show_find = true;
                ce.focus_find = true;
            }
            if goto {
                ce.goto = Some(String::new());
            }
            // Tabs.
            ui.horizontal_wrapped(|ui| {
                for i in 0..ce.docs.len() {
                    let label = format!("{}{}", ce.docs[i].name(), if ce.docs[i].dirty() { " •" } else { "" });
                    let selected = ce.active == i;
                    if ui.selectable_label(selected, label).on_hover_text(ce.docs[i].path.display().to_string()).clicked() {
                        ce.active = i;
                    }
                    if ui.small_button("×").clicked() {
                        close_tab = Some(i);
                    }
                    ui.add_space(6.0);
                }
            });
            ui.horizontal(|ui| {
                if ui.button(format!("💾 {}", t("Save"))).on_hover_text("Ctrl+S").clicked() || save {
                    let i = ce.active;
                    if ce.save(i) {
                        saved_path = Some(ce.docs[i].path.clone());
                    }
                }
                if ui.button(format!("🔍 {}", t("Find"))).on_hover_text("Ctrl+F").clicked() {
                    ce.show_find = !ce.show_find;
                    ce.focus_find = ce.show_find;
                }
                if ui.button(t("Go to line")).on_hover_text("Ctrl+G").clicked() {
                    ce.goto = Some(String::new());
                }
                ui.weak(ce.docs[ce.active].path.strip_prefix(&project_root).unwrap_or(&ce.docs[ce.active].path).display().to_string());
            });
            if let Some(mut g) = ce.goto.clone() {
                ui.horizontal(|ui| {
                    ui.label(t("Line"));
                    let r = ui.add(egui::TextEdit::singleline(&mut g).desired_width(70.0));
                    r.request_focus();
                    if r.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
                        if let Ok(n) = g.trim().parse::<usize>() {
                            let i = ce.active;
                            ce.docs[i].jump = Some(n);
                        }
                        ce.goto = None;
                    } else if ui.input(|i| i.key_pressed(Key::Escape)) {
                        ce.goto = None;
                    } else {
                        ce.goto = Some(g);
                    }
                });
            }
            if ce.show_find {
                find_bar(ui, ce);
            }
            ui.separator();
            let i = ce.active;
            editor_area(ui, ce, i, api);
            ui.separator();
            let d = &ce.docs[ce.active];
            ui.horizontal(|ui| {
                ui.weak(format!("{} {}, {} {}", t("Ln"), ce.cursor_line, t("Col"), ce.cursor_col));
                ui.weak(match d.lang {
                    Lang::Rust => "Rust",
                    Lang::Wgsl => "WGSL",
                    Lang::Plain => "Text",
                });
                ui.weak(format!("{} {}", d.text.lines().count(), t("lines")));
                if d.dirty() {
                    ui.colored_label(theme::red(), t("Unsaved changes"));
                }
            });
        });
        if let Some(i) = close_tab {
            self.code.docs.remove(i);
            if self.code.docs.is_empty() {
                self.code.open = false;
            }
        }
        if !open {
            self.code.open = false;
        }
        // Saving a script rebuilds it, saving a shader reloads it (the renderer watches file stamps).
        if let Some(p) = saved_path {
            if p.extension().is_some_and(|e| e == "rs") {
                self.scripts.start_build(&self.project, self.console.sender());
            } else {
                self.renderer.clear_thumbnails();
                self.reload_assets();
            }
        }
    }
}

fn find_bar(ui: &mut egui::Ui, ce: &mut CodeEditor) {
    let i = ce.active;
    let hits = matches(&ce.docs[i].text, &ce.find, ce.case);
    ui.horizontal(|ui| {
        let r = ui.add(egui::TextEdit::singleline(&mut ce.find).hint_text(t("Find")).desired_width(200.0));
        if ce.focus_find {
            r.request_focus();
            ce.focus_find = false;
        }
        let enter = r.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
        ui.checkbox(&mut ce.case, t("Aa")).on_hover_text(t("Match case"));
        ui.weak(format!("{} {}", hits.len(), t("matches")));
        let cursor_char = ce.cursor_idx;
        let next_hit = |forward: bool| -> Option<usize> {
            let text = &ce.docs[i].text;
            let cur = text.char_indices().nth(cursor_char).map_or(text.len(), |(b, _)| b);
            if forward {
                hits.iter().copied().find(|h| *h >= cur).or(hits.first().copied())
            } else {
                hits.iter().rev().copied().find(|h| *h + 1 < cur).or(hits.last().copied())
            }
        };
        let (mut go_next, mut go_prev) = (enter, false);
        if ui.button("↓").on_hover_text(t("Next")).clicked() {
            go_next = true;
        }
        if ui.button("↑").on_hover_text(t("Previous")).clicked() {
            go_prev = true;
        }
        let target = if go_next { next_hit(true) } else if go_prev { next_hit(false) } else { None };
        if let Some(b) = target {
            let a = byte_to_char(&ce.docs[i].text, b);
            ce.select = Some((a, a + ce.find.chars().count()));
        }
        if ui.button(t("Close")).clicked() {
            ce.show_find = false;
        }
    });
    ui.horizontal(|ui| {
        ui.add(egui::TextEdit::singleline(&mut ce.replace).hint_text(t("Replace with")).desired_width(200.0));
        if ui.button(t("Replace all")).clicked() && !ce.find.is_empty() {
            let d = &mut ce.docs[i];
            let mut out = String::new();
            let mut last = 0;
            for h in matches(&d.text, &ce.find, ce.case) {
                if h >= last {
                    out.push_str(&d.text[last..h]);
                    out.push_str(&ce.replace);
                    last = h + ce.find.len();
                }
            }
            out.push_str(&d.text[last..]);
            d.text = out;
        }
    });
}

/// Line-number gutter + highlighted text + hints.
fn editor_area(ui: &mut egui::Ui, ce: &mut CodeEditor, i: usize, api: &mut crate::api_docs::ApiState) {
    let dark = !theme::is_light();
    let lang = ce.docs[i].lang;
    let id = ui.make_persistent_id(("code-text", ce.docs[i].path.clone()));
    let font = FontId::monospace(13.5);
    let mut layouter = |ui: &egui::Ui, s: &dyn egui::TextBuffer, wrap: f32| ui.fonts_mut(|f| f.layout_job(wgsl_edit::highlight_lang(s.as_str(), dark, FontId::monospace(13.5), wrap, lang)));
    let hints_h = 70.0;
    let height = (ui.available_height() - hints_h - 28.0).max(140.0);

    // Tab accepts the first suggestion (computed last frame from the cursor word).
    let suggestion: Option<(String, usize, usize)> = ui.memory(|m| m.data.get_temp(id.with("sugg")));
    if suggestion.is_some() && ui.input_mut(|inp| inp.consume_key(Modifiers::NONE, Key::Tab)) {
        if let Some((name, start, end)) = suggestion.clone() {
            let text = &mut ce.docs[i].text;
            if end <= text.len() && start <= end {
                text.replace_range(start..end, &name);
                ce.select = Some((byte_to_char(text, start + name.len()), byte_to_char(text, start + name.len())));
            }
        }
    }

    let mut cursor_byte = None;
    let line_count = ce.docs[i].text.lines().count().max(1) + 1;
    let jump = ce.docs[i].jump.take();
    egui::ScrollArea::both().id_salt(id.with("scroll")).max_height(height).auto_shrink([false, false]).show(ui, |ui| {
        ui.horizontal_top(|ui| {
            // Gutter.
            let gutter: String = (1..=line_count).map(|n| format!("{n}\n")).collect();
            ui.add(egui::Label::new(RichText::new(gutter.trim_end()).font(font.clone()).color(pal().text_dim)).selectable(false));
            let text = &mut ce.docs[i].text;
            let out = egui::TextEdit::multiline(text).id(id).font(font.clone()).code_editor().frame(egui::Frame::NONE).desired_width(f32::INFINITY).lock_focus(true).layouter(&mut layouter).show(ui);
            // Apply requested selection / line jump.
            let mut want: Option<(usize, usize)> = ce.select.take();
            if let Some(line) = jump {
                let start: usize = text.split('\n').take(line.saturating_sub(1)).map(|l| l.chars().count() + 1).sum();
                want = Some((start, start));
            }
            if let Some((a, b)) = want {
                let mut state = TextEditState::load(ui.ctx(), id).unwrap_or_default();
                state.cursor.set_char_range(Some(egui::text::CCursorRange::two(CCursor::new(a), CCursor::new(b))));
                state.store(ui.ctx(), id);
                let row = out.galley.pos_from_cursor(CCursor::new(a));
                ui.scroll_to_rect(row.translate(out.galley_pos.to_vec2()).expand2(egui::vec2(0.0, 60.0)), Some(egui::Align::Center));
                ui.memory_mut(|m| m.request_focus(id));
            }
            if let Some(r) = out.cursor_range {
                let idx = r.primary.index.0;
                cursor_byte = Some(text.char_indices().nth(idx).map_or(text.len(), |(b, _)| b));
                let before: String = text.chars().take(idx).collect();
                ce.cursor_line = before.matches('\n').count() + 1;
                ce.cursor_col = before.rsplit('\n').next().map_or(0, |l| l.chars().count()) + 1;
                ce.cursor_idx = idx;
            }
        });
    });

    // Hints.
    ui.separator();
    let text = ce.docs[i].text.clone();
    let (start, word) = cursor_byte.map(|p| word_at(&text, p)).unwrap_or((0, String::new()));
    let short = word.rsplit('.').next().unwrap_or(&word).to_string();
    let shown: Vec<(String, String, String)> = if short.len() < 2 || lang == Lang::Plain {
        Vec::new()
    } else if lang == Lang::Wgsl {
        wgsl_edit::API.iter().filter(|a| a.0.contains(short.as_str())).take(5).map(|a| (a.0.to_string(), a.1.to_string(), a.2.to_string())).collect()
    } else {
        // Everything public in the engine, from its rustdoc (methods of `Ctx` first after `ctx.`).
        crate::api_docs::complete(&word, 6).into_iter().map(|it| (it.name.clone(), it.signature.clone(), format!("{}  ·  {}", it.path(), it.summary()))).collect()
    };
    let mut insert = None;
    egui::ScrollArea::vertical().id_salt(id.with("hints")).max_height(hints_h).show(ui, |ui| {
        if shown.is_empty() {
            ui.weak(t("Hints appear as you type (Tab inserts the first). Ctrl+F find, Ctrl+G go to line, Ctrl+S save."));
        }
        for (name, sig, doc) in &shown {
            ui.horizontal_wrapped(|ui| {
                if lang != Lang::Wgsl && ui.small_button("📖").on_hover_text(t("Full documentation (F1)")).clicked() {
                    api.show(doc.split("  ·  ").next().unwrap_or(name));
                }
                if ui.link(RichText::new(name).monospace().strong()).clicked() {
                    insert = Some(name.clone());
                }
                ui.label(RichText::new(sig).monospace().weak());
                ui.label(RichText::new(doc).size(11.5).color(Color32::from_gray(150)));
            });
        }
    });
    let word_start = start + word.len() - short.len();
    if let (Some(first), Some(p)) = (shown.first(), cursor_byte) {
        ui.memory_mut(|m| m.data.insert_temp(id.with("sugg"), (first.0.clone(), word_start, p)));
    } else {
        ui.memory_mut(|m| m.data.remove::<(String, usize, usize)>(id.with("sugg")));
    }
    if let (Some(name), Some(p)) = (insert, cursor_byte) {
        let text = &mut ce.docs[i].text;
        text.replace_range(word_start..p, &name);
    }
}

fn word_at(text: &str, pos: usize) -> (usize, String) {
    let pos = pos.min(text.len());
    let start = text[..pos].char_indices().rev().take_while(|(_, c)| c.is_alphanumeric() || *c == '_' || *c == '.').last().map_or(pos, |(b, _)| b);
    (start, text[start..pos].to_string())
}
