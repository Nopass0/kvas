//! Writes the generated pages of the documentation site (Russian and English) from the code itself:
//! the node reference and the catalogue of built-in materials. They can never drift from the library.
//!
//! `cargo run -p kvas_engine --example gen_node_docs`   (run from the repository root)

use kvas_engine::material_graph::{all_specs, available, is_output, GraphMode};
use kvas_engine::material_nodes::help_for;
use kvas_engine::material_nodes_ext::CATEGORIES;
use kvas_engine::material_presets::library;
use std::fmt::Write;

struct Text {
    russian: bool,
}

impl Text {
    fn category(&self, category: &str) -> (&'static str, &'static str) {
        let ru = self.russian;
        let pick = |en: (&'static str, &'static str), ru_: (&'static str, &'static str)| if ru { ru_ } else { en };
        match category {
            "Input" => pick(("Input", "Constants, textures, time, UV."), ("Ввод", "Константы, текстуры, время, UV.")),
            "Geometry" => pick(("Geometry", "Position, normal, height, direction masks, vertex offsets."), ("Геометрия", "Позиция, нормаль, высота, маски по направлению, смещение вершин.")),
            "Light" => pick(("Light", "Sun, shadows, highlights, rim light, glow (Emission)."), ("Свет", "Солнце, тени, блики, контровой свет, свечение (Emission).")),
            "Normal" => pick(("Normals", "Normal maps, bump from height, normal blending."), ("Нормали", "Карты нормалей, бамп из высоты, смешивание нормалей.")),
            "Noise" => pick(("Noise", "Fractal, Perlin, Voronoi, cracks, clouds, grunge. They show a preview by default."), ("Шумы", "Фрактальные, Перлин, Вороной, трещины, облака, грязь. По умолчанию показывают превью.")),
            "Procedural" => pick(("Patterns", "Checker, bricks, hexagons, planks, weave, gradients and shapes."), ("Узоры", "Шашки, кирпичи, соты, доски, плетение, градиенты и фигуры.")),
            "Colour" => pick(("Colour", "Color ramp, blend modes, HSV, colour correction."), ("Цвет", "Color ramp, режимы наложения, HSV, цветокоррекция.")),
            "Math" => pick(("Math", "Arithmetic, curves, comparisons, transitions."), ("Математика", "Арифметика, кривые, сравнения, переходы.")),
            "Vector" => pick(("Vectors", "Components, dot and cross products, rotations, coordinates."), ("Векторы", "Компоненты, скалярное и векторное произведение, повороты, координаты.")),
            "Utility" => pick(("Utility", "Reroute, comments, number from the Inspector."), ("Утилиты", "Reroute, комментарии, число из Инспектора.")),
            "Effect" => pick(("Camera", "Only in camera effect graphs (.kfx)."), ("Камера", "Только в графах эффектов камеры (.kfx).")),
            _ => ("", ""),
        }
    }
}

fn reference(t: &Text) -> String {
    let mut md = String::new();
    let total = all_specs().filter(|s| !is_output(s.kind)).count();
    writeln!(md, "---\noutline: [2, 2]\n---\n").unwrap();
    if t.russian {
        writeln!(md, "# Справочник нод\n\nСтраница создаётся автоматически из библиотеки нод (`cargo run -p kvas_engine --example gen_node_docs`). Всего нод: **{total}**.\n").unwrap();
        writeln!(md, "Буквы показывают, где ноду можно использовать: **M** - материал (`.kmat`), **S** - шейдер объекта (`.kshader`), **E** - эффект камеры (`.kfx`).\n").unwrap();
        writeln!(md, "Параметры показаны как `имя (мин..макс)`. Подсказки - те же, что видны в редакторе.\n").unwrap();
    } else {
        writeln!(md, "# Node reference\n\nThis page is generated from the node library (`cargo run -p kvas_engine --example gen_node_docs`). Nodes in total: **{total}**.\n").unwrap();
        writeln!(md, "The letters show where a node can be used: **M** - material (`.kmat`), **S** - object shader (`.kshader`), **E** - camera effect (`.kfx`).\n").unwrap();
        writeln!(md, "Parameters are shown as `name (min..max)`. The descriptions are the tooltips of the editor (some older nodes only have Russian or no text yet).\n").unwrap();
    }
    let header = if t.russian { "| Нода | Где | Входы | Параметры | Описание |\n|---|---|---|---|---|" } else { "| Node | Where | Inputs | Parameters | Description |\n|---|---|---|---|---|" };
    for category in CATEGORIES {
        let specs: Vec<_> = all_specs().filter(|s| s.category == *category && !is_output(s.kind)).collect();
        if specs.is_empty() {
            continue;
        }
        let (title, about) = t.category(category);
        writeln!(md, "## {title}\n\n{about}\n\n{header}").unwrap();
        for s in specs {
            let mut where_ = String::new();
            for (mode, letter) in [(GraphMode::Material, 'M'), (GraphMode::Shader, 'S'), (GraphMode::Effect, 'E')] {
                if available(s.kind, mode) {
                    where_.push(letter);
                }
            }
            let tinted = s.color && s.params.len() >= 3;
            let inputs = s.inputs.iter().map(|i| i.0).collect::<Vec<_>>().join(", ");
            let params = s.params.iter().skip(if tinted { 3 } else { 0 }).map(|p| format!("{} ({}..{})", p.0.split(" (").next().unwrap_or(p.0), p.2, p.3)).collect::<Vec<_>>().join(", ");
            let colour = if t.russian { "цвет" } else { "colour" };
            let params = if tinted { format!("{colour}{}{}", if params.is_empty() { "" } else { ", " }, params) } else { params };
            let help = help_for(s.kind, t.russian);
            writeln!(md, "| **{}** | {} | {} | {} | {} |", s.kind, where_, if inputs.is_empty() { "-".into() } else { inputs }, if params.is_empty() { "-".into() } else { params }, help.replace('|', "\\|")).unwrap();
        }
        writeln!(md).unwrap();
    }
    writeln!(md, "## {}\n", if t.russian { "Выходные ноды" } else { "Output nodes" }).unwrap();
    writeln!(md, "{}", if t.russian { "| Граф | Нода | Входы |\n|---|---|---|" } else { "| Graph | Node | Inputs |\n|---|---|---|" }).unwrap();
    for (mode, ru, en) in [(GraphMode::Material, "Материал", "Material"), (GraphMode::Shader, "Шейдер объекта", "Object shader"), (GraphMode::Effect, "Эффект камеры", "Camera effect")] {
        if let Some(s) = all_specs().find(|s| s.kind == mode.output_kind()) {
            writeln!(md, "| {} | **{}** | {} |", if t.russian { ru } else { en }, s.kind, s.inputs.iter().map(|i| i.0).collect::<Vec<_>>().join(", ")).unwrap();
        }
    }
    md
}

fn materials(t: &Text) -> String {
    let all = library();
    let mut md = String::new();
    if t.russian {
        writeln!(md, "# Каталог встроенных материалов\n\nВ каждый проект кладётся папка `assets/materials/library/` с **{}** материалами. Страница создаётся автоматически.\n", all.len()).unwrap();
        writeln!(md, "![Все материалы](/images/materials.png)\n").unwrap();
        writeln!(md, "Каждый материал - обычный граф нод: откройте двойным кликом и меняйте цвета, масштаб шума, шероховатость. Чтобы применить - перетащите `.kmat` на объект или в поле `shader` компонента MeshRenderer.\n").unwrap();
    } else {
        writeln!(md, "# Built-in material catalogue\n\nEvery new project gets `assets/materials/library/` with **{}** materials. This page is generated.\n", all.len()).unwrap();
        writeln!(md, "![All materials](/images/materials.png)\n").unwrap();
        writeln!(md, "Each material is an ordinary node graph: double-click it to change colours, noise scale or roughness. To use one, drag the `.kmat` onto an object or into the `shader` field of the MeshRenderer.\n").unwrap();
    }
    let mut categories: Vec<&str> = Vec::new();
    for p in &all {
        if !categories.contains(&p.category) {
            categories.push(p.category);
        }
    }
    for category in categories {
        let names: Vec<_> = all.iter().filter(|p| p.category == category).map(|p| format!("`{}`", p.name)).collect();
        writeln!(md, "## {category}\n\n{}\n", names.join(", ")).unwrap();
    }
    md
}

fn write(path: &str, text: String) {
    let path = std::path::Path::new(path);
    std::fs::create_dir_all(path.parent().unwrap()).expect("create folder");
    std::fs::write(path, text).expect("write page");
    println!("wrote {}", path.display());
}

fn main() {
    let (ru, en) = (Text { russian: true }, Text { russian: false });
    write("docs/nodes/reference.md", reference(&ru));
    write("docs/en/nodes/reference.md", reference(&en));
    write("docs/nodes/materials.md", materials(&ru));
    write("docs/en/nodes/materials.md", materials(&en));
}
