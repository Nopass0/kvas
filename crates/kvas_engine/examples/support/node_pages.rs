// Detailed node pages for `gen_node_docs` (included with `#[path]`): one page per category,
// every node with its inputs, parameters, outputs and how it is usually wired.

use kvas_engine::material_graph::{all_specs, available, is_output, GraphMode};
use kvas_engine::material_nodes::help_for;
use kvas_engine::material_nodes_v4::{choices, conversion_badge, input_type, output_type, outputs, SocketType, CONVERSIONS};
use std::fmt::Write;

fn type_name(ru: bool, ty: SocketType) -> &'static str {
    match (ty, ru) {
        (SocketType::Float, true) => "число",
        (SocketType::Colour, true) => "цвет",
        (SocketType::Vector, true) => "вектор",
        (SocketType::Float, false) => "number",
        (SocketType::Colour, false) => "colour",
        (SocketType::Vector, false) => "vector",
    }
}

fn default_words(ru: bool, expr: &str) -> String {
    match expr {
        "vec3<f32>(in.uv, 0.0)" => (if ru { "UV объекта" } else { "the object's UV" }).into(),
        "in.world_pos" => (if ru { "позиция в мире" } else { "world position" }).into(),
        "n" => (if ru { "нормаль поверхности" } else { "surface normal" }).into(),
        e => match e.strip_prefix("vec3<f32>(").and_then(|r| r.strip_suffix(')')) {
            Some(inner) => format!("`{inner}`"),
            None => format!("`{e}`"),
        },
    }
}

/// One page per category: every node in detail.
pub fn category_page(ru: bool, category: &str, title: &str, about: &str) -> String {
    let mut md = String::new();
    writeln!(md, "---\noutline: [2, 3]\n---\n\n# {title}\n\n{about}\n").unwrap();
    writeln!(
        md,
        "{}\n",
        if ru {
            "Страница создаётся из кода библиотеки. Типы сокетов: **число** (серый круг), **цвет** (жёлтый круг), **вектор** (фиолетовый ромб). Любой выход можно подключить к любому входу; если типы различаются, на входе выбирается приведение (ПКМ по входу). См. [как соединяются ноды](../wiring)."
        } else {
            "Generated from the node library. Socket types: **number** (grey circle), **colour** (yellow circle), **vector** (purple diamond). Any output can feed any input; when the types differ, pick a conversion on the input (right-click it). See [how nodes connect](../wiring)."
        }
    )
    .unwrap();
    for s in all_specs().filter(|s| s.category == category && !is_output(s.kind)) {
        let mut where_ = Vec::new();
        for (mode, en, r) in [(GraphMode::Material, "material", "материал"), (GraphMode::Shader, "object shader", "шейдер объекта"), (GraphMode::Effect, "camera effect", "эффект камеры")] {
            if available(s.kind, mode) {
                where_.push(if ru { r } else { en });
            }
        }
        writeln!(md, "## {}\n", s.kind).unwrap();
        let help = help_for(s.kind, ru);
        if !help.is_empty() {
            writeln!(md, "{help}\n").unwrap();
        }
        writeln!(md, "*{}: {}*\n", if ru { "Где работает" } else { "Works in" }, where_.join(", ")).unwrap();
        if !s.inputs.is_empty() {
            writeln!(md, "{}", if ru { "**Входы**\n\n| Вход | Тип | Если не подключён |\n|---|---|---|" } else { "**Inputs**\n\n| Input | Type | When not connected |\n|---|---|---|" }).unwrap();
            for (name, default) in s.inputs {
                writeln!(md, "| {name} | {} | {} |", type_name(ru, input_type(name, default)), default_words(ru, default)).unwrap();
            }
            writeln!(md).unwrap();
        }
        let tinted = s.color && s.params.len() >= 3;
        if !s.params.is_empty() {
            writeln!(md, "{}", if ru { "**Параметры** (меняются прямо в ноде или в Инспекторе)\n\n| Параметр | По умолчанию | Диапазон | Варианты |\n|---|---|---|---|" } else { "**Parameters** (on the node or in the Inspector)\n\n| Parameter | Default | Range | Choices |\n|---|---|---|---|" }).unwrap();
            if tinted {
                writeln!(md, "| {} | {:.2}, {:.2}, {:.2} | 0..1 | - |", if ru { "цвет" } else { "colour" }, s.params[0].1, s.params[1].1, s.params[2].1).unwrap();
            }
            for (k, (name, default, min, max)) in s.params.iter().enumerate().skip(if tinted { 3 } else { 0 }) {
                let ch = choices(s.kind, k).map(|c| c.iter().enumerate().map(|(i, n)| format!("{i} {n}")).collect::<Vec<_>>().join(", ")).unwrap_or_else(|| "-".into());
                writeln!(md, "| {} | {default} | {min}..{max} | {ch} |", name.split(" (").next().unwrap_or(name)).unwrap();
            }
            writeln!(md).unwrap();
        }
        let outs = outputs(s.kind);
        if outs.len() > 1 {
            writeln!(md, "{}", if ru { "**Выходы**\n\n| Выход | Тип |\n|---|---|" } else { "**Outputs**\n\n| Output | Type |\n|---|---|" }).unwrap();
            for (k, (name, _)) in outs.iter().enumerate() {
                writeln!(md, "| {name} | {} |", type_name(ru, output_type(s.kind, s.category, k))).unwrap();
            }
            writeln!(md).unwrap();
        } else {
            writeln!(md, "**{}**: {}\n", if ru { "Выход" } else { "Output" }, type_name(ru, output_type(s.kind, s.category, 0))).unwrap();
        }
        let hint = match (output_type(s.kind, s.category, 0), ru) {
            (SocketType::Colour, true) => "Обычно идёт в **Output > Base color** или **Emission**, либо во вход **A/B** ноды «Смешать цвета». В числовой вход приводится к яркости.",
            (SocketType::Colour, false) => "Usually goes to **Output > Base color** or **Emission**, or to **A/B** of Mix colour. Into a number input it is converted to luminance.",
            (SocketType::Vector, true) => "Подключается к входам **Vector / Coords / UV** текстур и шумов (координаты) или к **Normal** выхода. Для одного канала выберите приведение «X/Y/Z».",
            (SocketType::Vector, false) => "Feeds **Vector / Coords / UV** inputs of textures and noises, or **Normal** of the output. For one channel pick the X/Y/Z conversion.",
            (SocketType::Float, true) => "Работает как маска: в **Factor** «Смешать цвета», в **Position** Color ramp, в **Roughness / Metallic / Alpha / Height** выхода или во вход **A/B** «Математики».",
            (SocketType::Float, false) => "Use it as a mask: into **Factor** of Mix colour, **Position** of a Color ramp, **Roughness / Metallic / Alpha / Height** of the output, or **A/B** of Math.",
        };
        writeln!(md, "> {hint}\n").unwrap();
        if let Some(first) = s.inputs.first() {
            let example = match (input_type(first.0, first.1), ru) {
                (SocketType::Vector, true) => format!("Пример: «Координаты текстуры» (World) → «Преобразование координат» (масштаб 4) → **{}** (вход {}) → «Color ramp» → Output > Base color.", s.kind, first.0),
                (SocketType::Vector, false) => format!("Example: Texture coordinate (World) → Mapping (scale 4) → **{}** (input {}) → Color ramp → Output > Base color.", s.kind, first.0),
                (SocketType::Colour, true) => format!("Пример: «Картинка» (Цвет) → **{}** (вход {}) → Output > Base color.", s.kind, first.0),
                (SocketType::Colour, false) => format!("Example: Image texture (Colour) → **{}** (input {}) → Output > Base color.", s.kind, first.0),
                (SocketType::Float, true) => format!("Пример: «Текстура шума» (Коэффициент) → **{}** (вход {}) → «Смешать цвета» (Коэффициент).", s.kind, first.0),
                (SocketType::Float, false) => format!("Example: Noise texture (Factor) → **{}** (input {}) → Mix colour (Factor).", s.kind, first.0),
            };
            writeln!(md, "{example}\n").unwrap();
        }
    }
    md
}

/// How sockets connect: types, conversions, several outputs, shortcuts, an example.
pub fn wiring(ru: bool) -> String {
    let mut md = String::new();
    if ru {
        md.push_str("# Как соединяются ноды\n\nКаждый сокет несёт три числа (vec3). Число хранится во всех трёх каналах, цвет — как RGB, вектор — как XYZ. Поэтому **любой выход подключается к любому входу**, а смысл задают тип и приведение.\n\n");
        md.push_str("## Типы сокетов\n\n| Тип | Значок | Примеры |\n|---|---|---|\n| число | серый круг | Roughness, Factor, Scale, маски шумов |\n| цвет | жёлтый круг | Base color, Colour, Stop 1..4 |\n| вектор | фиолетовый ромб | Vector, Coords, UV, Normal |\n\n");
        md.push_str("## Приведение на входе\n\nПКМ по входу (или список в Инспекторе) выбирает, как входящее значение превращается перед использованием. На связи появляется значок. Цвет, подключённый к числовому входу, по умолчанию берёт «Яркость».\n\n| № | Приведение | Значок |\n|---|---|---|\n");
    } else {
        md.push_str("# How nodes connect\n\nEvery socket carries three numbers (a vec3). A number is stored in all three channels, a colour as RGB, a vector as XYZ. So **any output connects to any input**; the type and the conversion give it meaning.\n\n");
        md.push_str("## Socket types\n\n| Type | Mark | Examples |\n|---|---|---|\n| number | grey circle | Roughness, Factor, Scale, noise masks |\n| colour | yellow circle | Base color, Colour, Stop 1..4 |\n| vector | purple diamond | Vector, Coords, UV, Normal |\n\n");
        md.push_str("## Conversions on an input\n\nRight-click an input (or use the list in the Inspector) to choose how the incoming value is turned before use. A badge shows it on the link. A colour plugged into a number input gets \"Luminance\" automatically.\n\n| # | Conversion | Badge |\n|---|---|---|\n");
    }
    for (i, (name, _)) in CONVERSIONS.iter().enumerate() {
        writeln!(md, "| {i} | {name} | `{}` |", conversion_badge(i as u8)).unwrap();
    }
    if ru {
        md.push_str("\n## Несколько выходов\n\n«Картинка» отдаёт цвет, R, G, B и яркость; «Текстура Вороного» — коэффициент, расстояние, край, значение и цвет ячейки; «Координаты текстуры» — UV, объект, мир, нормаль, отражение и экран; «Текстура кирпича» — цвет, маску, значение кирпича и расстояние до шва. Тяните связь от нужного выхода; выход 0 — основной.\n\n## Горячие клавиши\n\nShift+A или двойной клик — добавить ноду; Ctrl+C / V / D — копировать / вставить / дублировать; Delete или X — удалить; Ctrl+G — рамка-комментарий; Ctrl+L — выделить всё, что питает выделенное; Alt+D — отключить входы; H — превью внутри нод; P — превью выбранной ноды; стрелки — сдвиг; F — весь граф; 1 — масштаб 100%; Ctrl+S — сохранить; Ctrl+E — экспорт в WGSL; K — шпаргалка.\n\n## Пример: мрамор из базовых нод\n\n1. «Координаты текстуры» (World) → «Преобразование координат» (масштаб 3).\n2. «Векторная математика» (Скалярное произведение) с вектором (0.82, 0.31, 0.48) — направление прожилок.\n3. «Текстура шума» (масштаб 0.9, детализация 7, шероховатость 0.58) — турбулентность.\n4. «Математика» (Умножить и сложить): шум × 3 + направление → фаза; «Синус» → «Модуль» — тонкие прожилки около нуля.\n5. «Color ramp» (Плавно): цвет прожилки в 0, фон с 0.16.\n6. «Смешать цвета» (Перекрытие) с цветным шумом — облачность; «Перевод диапазона» прожилок → Roughness; «Рельеф» из прожилок → Normal.\n\nГотовые графы — `materials/library/Game Ready/`: откройте любой и посмотрите, как он собран.\n");
    } else {
        md.push_str("\n## Several outputs\n\nImage texture gives colour, R, G, B and luminance; Voronoi texture gives factor, distance, edge, cell value and cell colour; Texture coordinate gives UV, object, world, normal, reflection and screen; Brick texture gives colour, mask, brick value and distance to the mortar. Drag the link from the output you need; output 0 is the main one.\n\n## Keyboard shortcuts\n\nShift+A or double-click add a node; Ctrl+C / V / D copy / paste / duplicate; Delete or X delete; Ctrl+G frame in a comment; Ctrl+L select everything upstream; Alt+D disconnect inputs; H previews inside nodes; P preview the selected node; arrows nudge; F fit; 1 zoom 100%; Ctrl+S save; Ctrl+E export WGSL; K the shortcut sheet.\n\n## Example: marble from basic nodes\n\n1. Texture coordinate (World) → Mapping (scale 3).\n2. Vector math (Dot product) with (0.82, 0.31, 0.48) — the vein direction.\n3. Noise texture (scale 0.9, detail 7, roughness 0.58) — turbulence.\n4. Math (Multiply add): noise × 3 + direction → phase; Sine → Absolute — thin veins near zero.\n5. Color ramp (Ease): vein colour at 0, base from 0.16.\n6. Mix colour (Overlay) with a coloured noise for clouds; Map range of the veins → Roughness; Bump from the veins → Normal.\n\nReady graphs are in `materials/library/Game Ready/` — open any to see how it is built.\n");
    }
    md
}
