//! In-editor API reference built from the engine's own rustdoc comments: every public struct,
//! enum, trait, field, variant, function and method with its signature and full documentation.
//! The sources are embedded at build time (see `build.rs`), so the reference always matches the
//! engine the editor was built with. Used by the API window and by the code editor's hints.

use crate::editor::Editor;
use eframe::egui::{self, Color32, RichText};

include!(concat!(env!("OUT_DIR"), "/engine_sources.rs"));

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Module,
    Struct,
    Enum,
    Trait,
    Field,
    Variant,
    Function,
    Method,
    Const,
    Type,
    Macro,
}

impl Kind {
    pub fn label(self) -> &'static str {
        match self {
            Kind::Module => "mod",
            Kind::Struct => "struct",
            Kind::Enum => "enum",
            Kind::Trait => "trait",
            Kind::Field => "field",
            Kind::Variant => "variant",
            Kind::Function => "fn",
            Kind::Method => "method",
            Kind::Const => "const",
            Kind::Type => "type",
            Kind::Macro => "macro",
        }
    }
    fn color(self) -> Color32 {
        match self {
            Kind::Struct | Kind::Enum | Kind::Trait | Kind::Type => Color32::from_rgb(110, 190, 240),
            Kind::Function | Kind::Method | Kind::Macro => Color32::from_rgb(230, 190, 110),
            Kind::Field | Kind::Variant | Kind::Const => Color32::from_rgb(160, 210, 140),
            Kind::Module => Color32::from_rgb(200, 160, 230),
        }
    }
}

/// One documented public item.
#[derive(Clone, Debug)]
pub struct Item {
    pub kind: Kind,
    /// Module path inside the engine, e.g. `kvas_engine::script`.
    pub module: String,
    /// Type the item belongs to (methods, fields, variants); empty for free items.
    pub owner: String,
    pub name: String,
    /// Declaration with whitespace collapsed (no body).
    pub signature: String,
    /// The full rustdoc text (markdown).
    pub docs: String,
}

impl Item {
    /// `Ctx::play_sound`, `Transform`, `script::short_name`.
    pub fn path(&self) -> String {
        if self.owner.is_empty() {
            self.name.clone()
        } else {
            format!("{}::{}", self.owner, self.name)
        }
    }
    /// First sentence/line of the docs.
    pub fn summary(&self) -> &str {
        self.docs.lines().find(|l| !l.trim().is_empty()).unwrap_or("")
    }
}

fn ident(s: &str) -> String {
    s.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect()
}

/// Type name of an `impl` header: `impl<T> Trait for Name<T> {` gives `("Name", true)`.
fn impl_owner(line: &str) -> (String, bool) {
    let rest = line.trim_start_matches("impl").trim_start();
    // Skip generic parameters of the impl itself.
    let rest = if rest.starts_with('<') {
        let mut depth = 0;
        let mut cut = 0;
        for (i, c) in rest.char_indices() {
            match c {
                '<' => depth += 1,
                '>' => {
                    depth -= 1;
                    if depth == 0 {
                        cut = i + 1;
                        break;
                    }
                }
                _ => {}
            }
        }
        rest[cut..].trim_start()
    } else {
        rest
    };
    match rest.find(" for ") {
        Some(i) => (ident(rest[i + 5..].trim_start().trim_start_matches("&mut ").trim_start_matches('&')), true),
        None => (ident(rest), false),
    }
}

/// Parses one source file. Line-based on purpose: it reads the engine's own formatting
/// (rustfmt) and needs no compiler.
fn parse(module: &str, text: &str, out: &mut Vec<Item>) {
    let lines: Vec<&str> = text.lines().collect();
    let mut module_doc = Vec::new();
    let mut doc: Vec<String> = Vec::new();
    // (owner, kind of block, is a trait impl)
    let mut block: Option<(String, Kind, bool)> = None;
    // Inside `choice_enum!(` (the engine's enum macro): `Name { A, B } default A);`.
    let mut in_choice = false;
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let t = line.trim();
        let indent = line.len() - line.trim_start().len();
        i += 1;
        if let Some(d) = t.strip_prefix("//!") {
            module_doc.push(d.strip_prefix(' ').unwrap_or(d).to_string());
            continue;
        }
        if let Some(d) = t.strip_prefix("///") {
            doc.push(d.strip_prefix(' ').unwrap_or(d).to_string());
            continue;
        }
        if t.starts_with("#[") || t.is_empty() && !doc.is_empty() {
            if t.starts_with("#[cfg(test)]") {
                break; // tests are at the end of the file
            }
            continue;
        }
        if t.starts_with("choice_enum!(") {
            in_choice = true;
            continue;
        }
        if in_choice {
            in_choice = false;
            let name = ident(t);
            if let (Some(a), Some(b)) = (t.find('{'), t.find('}')) {
                let docs = doc.join("
");
                out.push(Item { kind: Kind::Enum, module: module.to_string(), owner: String::new(), name: name.clone(), signature: format!("pub enum {name}"), docs });
                for v in t[a + 1..b].split(',').map(str::trim).filter(|v| !v.is_empty()) {
                    out.push(Item { kind: Kind::Variant, module: module.to_string(), owner: name.clone(), name: v.to_string(), signature: format!("{name}::{v}"), docs: String::new() });
                }
            }
            doc.clear();
            continue;
        }
        if indent == 0 && t.starts_with('}') {
            block = None;
            doc.clear();
            continue;
        }
        // A declaration may span several lines; collect it up to its body or `;`.
        let mut decl = t.to_string();
        let opens_item = ["pub fn ", "pub const fn ", "pub unsafe fn ", "fn ", "pub struct ", "pub enum ", "pub trait ", "pub const ", "pub type ", "impl", "pub(crate) fn "]
            .iter()
            .any(|p| t.starts_with(p));
        if opens_item && t.contains('(') {
            let mut j = i;
            while !decl.contains('{') && !decl.ends_with(';') && j < lines.len() && j < i + 12 {
                decl.push(' ');
                decl.push_str(lines[j].trim());
                j += 1;
            }
        }
        let signature = decl.split('{').next().unwrap_or(&decl).trim().trim_end_matches(';').split_whitespace().collect::<Vec<_>>().join(" ");
        let docs = doc.join("\n");
        let mut push = |kind: Kind, owner: &str, name: String| {
            if !name.is_empty() {
                out.push(Item { kind, module: module.to_string(), owner: owner.to_string(), name, signature: signature.clone(), docs: docs.clone() });
            }
        };
        if indent == 0 {
            if t.starts_with("impl") && (t.starts_with("impl ") || t.starts_with("impl<")) {
                let (owner, is_trait) = impl_owner(t);
                block = Some((owner, Kind::Method, is_trait));
            } else if let Some(rest) = t.strip_prefix("pub struct ") {
                let name = ident(rest);
                push(Kind::Struct, "", name.clone());
                if t.ends_with('{') {
                    block = Some((name, Kind::Field, false));
                }
            } else if let Some(rest) = t.strip_prefix("pub enum ") {
                let name = ident(rest);
                push(Kind::Enum, "", name.clone());
                block = Some((name, Kind::Variant, false));
            } else if let Some(rest) = t.strip_prefix("pub trait ") {
                let name = ident(rest);
                push(Kind::Trait, "", name.clone());
                block = Some((name, Kind::Trait, false));
            } else if let Some(rest) = t.strip_prefix("pub fn ").or_else(|| t.strip_prefix("pub const fn ")) {
                push(Kind::Function, "", ident(rest));
            } else if let Some(rest) = t.strip_prefix("pub const ") {
                push(Kind::Const, "", ident(rest));
            } else if let Some(rest) = t.strip_prefix("pub type ") {
                push(Kind::Type, "", ident(rest));
            } else if let Some(rest) = t.strip_prefix("macro_rules! ") {
                if lines.get(i.saturating_sub(2)).is_some_and(|l| l.contains("macro_export")) || !docs.is_empty() {
                    push(Kind::Macro, "", format!("{}!", ident(rest)));
                }
            }
        } else if indent == 4 {
            if let Some((owner, kind, _)) = block.clone() {
                match kind {
                    Kind::Method => {
                        if let Some(rest) = t.strip_prefix("pub fn ").or_else(|| t.strip_prefix("pub const fn ")) {
                            push(Kind::Method, &owner, ident(rest));
                        }
                    }
                    // Trait definitions list their methods without `pub`.
                    Kind::Trait => {
                        if let Some(rest) = t.strip_prefix("fn ") {
                            push(Kind::Method, &owner, ident(rest));
                        }
                    }
                    Kind::Field => {
                        if let Some(rest) = t.strip_prefix("pub ") {
                            if rest.contains(':') && !rest.starts_with("fn ") {
                                push(Kind::Field, &owner, ident(rest));
                            }
                        }
                    }
                    Kind::Variant => {
                        if t.chars().next().is_some_and(|c| c.is_ascii_uppercase()) {
                            push(Kind::Variant, &owner, ident(t));
                        }
                    }
                    _ => {}
                }
            }
        }
        doc.clear();
    }
    if !module_doc.is_empty() {
        out.push(Item { kind: Kind::Module, module: module.to_string(), owner: String::new(), name: module.rsplit("::").next().unwrap_or(module).to_string(), signature: format!("mod {module}"), docs: module_doc.join("\n") });
    }
}

/// Every documented public item of the engine (parsed once).
pub fn index() -> &'static [Item] {
    static INDEX: std::sync::OnceLock<Vec<Item>> = std::sync::OnceLock::new();
    INDEX.get_or_init(|| {
        let mut out = Vec::new();
        for (rel, text) in ENGINE_SOURCES {
            let path = rel.trim_end_matches(".rs").replace('/', "::");
            let module = match path.as_str() {
                "lib" => "kvas_engine".to_string(),
                p => format!("kvas_engine::{}", p.trim_end_matches("::mod")),
            };
            parse(&module, text, &mut out);
        }
        out
    })
}

/// Items whose name starts with (then contains) `word`; after `ctx.` only `Ctx` methods.
pub fn complete(word: &str, limit: usize) -> Vec<&'static Item> {
    let (owner, short) = match word.rsplit_once('.') {
        Some((obj, s)) if obj.ends_with("ctx") => (Some("Ctx"), s),
        Some((_, s)) => (None, s),
        None => (None, word),
    };
    let short = short.to_lowercase();
    if short.len() < 2 {
        return Vec::new();
    }
    let ok = |it: &&Item| owner.is_none_or(|o| it.owner == o) && it.kind != Kind::Module;
    let mut found: Vec<&Item> = index().iter().filter(ok).filter(|it| it.name.to_lowercase().starts_with(&short)).collect();
    found.extend(index().iter().filter(ok).filter(|it| !it.name.to_lowercase().starts_with(&short) && it.name.to_lowercase().contains(&short)));
    // Script API first: it is what scripts call most.
    found.sort_by_key(|it| (it.owner != "Ctx", !it.name.to_lowercase().starts_with(&short)));
    found.truncate(limit);
    found
}

#[derive(Default)]
pub struct ApiState {
    pub open: bool,
    pub query: String,
    /// Path of the shown item (`Ctx::play_sound`).
    pub selected: Option<String>,
}

impl ApiState {
    /// Opens the reference on the item named `word` (the word under the cursor).
    pub fn show(&mut self, word: &str) {
        self.open = true;
        let short = word.rsplit(['.', ':']).next().unwrap_or(word);
        if let Some(it) = complete(word, 1).first().filter(|it| it.name == short) {
            self.selected = Some(it.path());
            self.query = short.to_string();
        } else {
            self.query = short.to_string();
        }
    }
}

/// Markdown of rustdoc rendered with headings, code blocks and inline code.
fn doc_text(ui: &mut egui::Ui, docs: &str) {
    let mut code = false;
    for line in docs.lines() {
        if line.trim_start().starts_with("```") {
            code = !code;
            continue;
        }
        if code {
            ui.label(RichText::new(line).monospace().color(Color32::from_rgb(200, 210, 190)));
        } else if line.trim().is_empty() {
            ui.add_space(4.0);
        } else if let Some(h) = line.strip_prefix('#') {
            ui.label(RichText::new(h.trim_start_matches('#').trim()).strong().size(15.0));
        } else {
            ui.label(line.replace('`', ""));
        }
    }
}

impl Editor {
    pub fn api_window(&mut self, ctx: &egui::Context) {
        if !self.api.open {
            return;
        }
        let mut open = true;
        egui::Window::new("Справочник API (rustdoc)").open(&mut open).default_size([980.0, 640.0]).show(ctx, |ui| {
            let api = &mut self.api;
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut api.query).hint_text("Поиск: play_sound, Transform, raycast, ctx.ui ...").desired_width(420.0));
                ui.weak(format!("{} элементов", index().len()));
            });
            ui.separator();
            let query = api.query.to_lowercase();
            egui::Panel::left("api-list").resizable(true).default_size(330.0).show(ui, |ui| {
                egui::ScrollArea::vertical().id_salt("api-list-scroll").show(ui, |ui| {
                    let mut shown = 0;
                    for it in index().iter().filter(|it| matches!(it.kind, Kind::Struct | Kind::Enum | Kind::Trait | Kind::Function | Kind::Macro | Kind::Module | Kind::Method | Kind::Const | Kind::Type)) {
                        let path = it.path();
                        if !query.is_empty() && !path.to_lowercase().contains(&query) && !it.summary().to_lowercase().contains(&query) {
                            continue;
                        }
                        if query.is_empty() && it.kind == Kind::Method {
                            continue; // methods are listed on their type's page
                        }
                        shown += 1;
                        if shown > 400 {
                            ui.weak("… уточните поиск");
                            break;
                        }
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(it.kind.label()).monospace().size(11.0).color(it.kind.color()));
                            let selected = api.selected.as_deref() == Some(path.as_str());
                            if ui.selectable_label(selected, RichText::new(&path).monospace()).on_hover_text(it.summary()).clicked() {
                                api.selected = Some(path.clone());
                            }
                        });
                    }
                });
            });
            egui::ScrollArea::vertical().id_salt("api-page").show(ui, |ui| {
                let Some(sel) = api.selected.clone() else {
                    ui.heading("Документация движка");
                    ui.label("Слева — все публичные типы, функции и модули движка с их описанием из исходников (rustdoc).");
                    ui.label("В редакторе кода подсказки берутся отсюда же; кнопка 📖 у подсказки или F1 на слове открывает её полную справку.");
                    ui.label("Начните со Script и Ctx: всё, что может скрипт, — методы Ctx.");
                    return;
                };
                let Some(it) = index().iter().find(|it| it.path() == sel) else { return };
                ui.label(RichText::new(format!("{} · {}", it.kind.label(), it.module)).weak());
                ui.heading(RichText::new(it.path()).monospace());
                ui.label(RichText::new(&it.signature).monospace().color(Color32::from_rgb(230, 200, 140)));
                ui.add_space(6.0);
                if it.docs.is_empty() {
                    ui.weak("(без описания)");
                } else {
                    doc_text(ui, &it.docs);
                }
                // A type's page lists its fields, variants and methods.
                let members: Vec<&Item> = index().iter().filter(|m| m.owner == it.name && it.owner.is_empty() && matches!(it.kind, Kind::Struct | Kind::Enum | Kind::Trait)).collect();
                for (kind, title) in [(Kind::Field, "Поля"), (Kind::Variant, "Варианты"), (Kind::Method, "Методы")] {
                    let list: Vec<&&Item> = members.iter().filter(|m| m.kind == kind).collect();
                    if list.is_empty() {
                        continue;
                    }
                    ui.add_space(10.0);
                    ui.label(RichText::new(format!("{title} ({})", list.len())).strong().size(16.0));
                    for m in list {
                        ui.separator();
                        if ui.link(RichText::new(&m.signature).monospace()).clicked() && kind == Kind::Method {
                            api.selected = Some(m.path());
                        }
                        if !m.docs.is_empty() {
                            doc_text(ui, &m.docs);
                        }
                    }
                }
            });
        });
        self.api.open = open;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_has_script_api_with_docs() {
        let idx = index();
        let play = idx.iter().find(|it| it.owner == "Ctx" && it.name == "play_sound").expect("Ctx::play_sound");
        assert!(!play.docs.is_empty() && play.signature.starts_with("pub fn play_sound"), "{play:?}");
        assert!(idx.iter().any(|it| it.kind == Kind::Struct && it.name == "Transform"));
        assert!(idx.iter().any(|it| it.kind == Kind::Field && it.owner == "Transform" && it.name == "position"));
        assert!(idx.iter().any(|it| it.kind == Kind::Trait && it.name == "Script"));
        assert!(idx.iter().any(|it| it.kind == Kind::Method && it.owner == "Script" && it.name == "update"));
        assert!(idx.iter().any(|it| it.kind == Kind::Variant && it.owner == "BodyKind"));
        assert!(complete("ctx.play_so", 5).iter().all(|it| it.owner == "Ctx"));
        assert!(idx.len() > 1500, "{}", idx.len());
    }
}
