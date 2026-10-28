//! Writes the node reference page of the documentation site from the node library itself,
//! so the list of nodes, sockets and parameters can never drift from the code.
//!
//! `cargo run -p kvas_engine --example gen_node_docs -- docs/nodes/reference.md`

use kvas_engine::material_graph::{all_specs, available, is_output, GraphMode};
use kvas_engine::material_nodes::help_for;
use kvas_engine::material_nodes_ext::CATEGORIES;
use std::fmt::Write;

fn category_title(category: &str) -> (&'static str, &'static str) {
    match category {
        "Input" => ("Ввод", "Константы, текстуры, время, UV."),
        "Geometry" => ("Геометрия", "Позиция, нормаль, высота, маски по направлению, смещение вершин."),
        "Light" => ("Свет", "Солнце, тени, блики, контровой свет, свечение (Emission)."),
        "Normal" => ("Нормали", "Карты нормалей, бамп из высоты, смешивание нормалей."),
        "Noise" => ("Шумы", "Фрактальные, Перлин, Вороной, трещины, облака, грязь. По умолчанию показывают превью."),
        "Procedural" => ("Узоры", "Шашки, кирпичи, соты, доски, плетение, градиенты и фигуры."),
        "Colour" => ("Цвет", "Color ramp, режимы наложения, HSV, цветокоррекция."),
        "Math" => ("Математика", "Арифметика, кривые, сравнения, переходы."),
        "Vector" => ("Векторы", "Компоненты, скалярное и векторное произведение, повороты, координаты."),
        "Utility" => ("Утилиты", "Reroute, комментарии, число из Инспектора."),
        "Effect" => ("Камера", "Только в графах эффектов камеры (.kfx)."),
        _ => ("", ""),
    }
}

fn main() {
    let out = std::env::args().nth(1).unwrap_or_else(|| "docs/nodes/reference.md".into());
    let mut md = String::new();
    let total = all_specs().filter(|s| !is_output(s.kind)).count();
    writeln!(md, "---\noutline: [2, 2]\n---\n").unwrap();
    writeln!(md, "# Справочник нод\n").unwrap();
    writeln!(md, "Страница создаётся автоматически из библиотеки нод (`cargo run -p kvas_engine --example gen_node_docs`). Всего нод: **{total}**.\n").unwrap();
    writeln!(md, "Значок после имени показывает, где ноду можно использовать: **M** - материал (`.kmat`), **S** - шейдер объекта (`.kshader`), **E** - эффект камеры (`.kfx`).\n").unwrap();
    for category in CATEGORIES {
        let specs: Vec<_> = all_specs().filter(|s| s.category == *category && !is_output(s.kind)).collect();
        if specs.is_empty() {
            continue;
        }
        let (title, about) = category_title(category);
        writeln!(md, "## {title}\n\n{about}\n").unwrap();
        writeln!(md, "| Нода | Где | Входы | Параметры | Описание |\n|---|---|---|---|---|").unwrap();
        for s in specs {
            let mut where_ = String::new();
            for (mode, letter) in [(GraphMode::Material, 'M'), (GraphMode::Shader, 'S'), (GraphMode::Effect, 'E')] {
                if available(s.kind, mode) {
                    where_.push(letter);
                }
            }
            let inputs = s.inputs.iter().map(|i| i.0).collect::<Vec<_>>().join(", ");
            let params = s
                .params
                .iter()
                .skip(if s.color && s.params.len() >= 3 { 3 } else { 0 })
                .map(|p| format!("{} ({}..{})", p.0.split(" (").next().unwrap_or(p.0), p.2, p.3))
                .collect::<Vec<_>>()
                .join(", ");
            let params = if s.color && s.params.len() >= 3 { format!("цвет{}{}", if params.is_empty() { "" } else { ", " }, params) } else { params };
            let help = help_for(s.kind, true);
            let help = if help.is_empty() { help_for(s.kind, false) } else { help };
            writeln!(md, "| **{}** | {} | {} | {} | {} |", s.kind, where_, if inputs.is_empty() { "-".into() } else { inputs }, if params.is_empty() { "-".into() } else { params }, help.replace('|', "\\|")).unwrap();
        }
        writeln!(md).unwrap();
    }
    writeln!(md, "## Выходные ноды\n").unwrap();
    writeln!(md, "| Граф | Нода | Входы |\n|---|---|---|").unwrap();
    for (mode, name) in [(GraphMode::Material, "Материал"), (GraphMode::Shader, "Шейдер объекта"), (GraphMode::Effect, "Эффект камеры")] {
        if let Some(s) = all_specs().find(|s| s.kind == mode.output_kind()) {
            writeln!(md, "| {name} | **{}** | {} |", s.kind, s.inputs.iter().map(|i| i.0).collect::<Vec<_>>().join(", ")).unwrap();
        }
    }
    if let Some(dir) = std::path::Path::new(&out).parent() {
        std::fs::create_dir_all(dir).expect("create output folder");
    }
    std::fs::write(&out, md).expect("write the reference");
    println!("{total} nodes -> {out}");
}
