---
outline: [2, 3]
---

# Игровые системы SDK

Эта страница описывает всё, что нужно для полноценной игры: экранный интерфейс со своими шрифтами, меню и настройки, сохранения, работу с файлами, сцены, камеры и их шейдеры, звук с реверберацией, частицы, декали, деревья и траву. Все примеры — рабочий Rust-код скриптов (`impl Script` или макрос `script!`). Полный пример игры на этих системах — `games/last_service` (см. [LAST SERVICE](../guide/last-service)).

## Экранный интерфейс: `ctx.ui()`

Интерфейс рисуется **каждый кадр** заново (immediate mode): скрипт в `update` описывает, что на экране, и сразу узнаёт, что нажал игрок в прошлом кадре. Координаты — в виртуальном экране высотой **720** единиц; ширина зависит от пропорций окна: `ctx.ui_width()` (1280 при 16:9). Прямоугольник — `[x, y, ширина, высота]` от левого верхнего угла.

| Метод | Что рисует |
|---|---|
| `panel(rect, цвет, радиус)` | закрашенная панель |
| `frame(rect, цвет, обводка, радиус)` | панель с рамкой |
| `text(текст, [x, y], размер, цвет, шрифт, Align::Left/Center/Right)` | строка текста |
| `paragraph(текст, [x, y], ширина, размер, цвет, шрифт)` | абзац с переносом |
| `image(путь, rect, оттенок)` | картинка из assets (PNG/JPG; до 256 px — пиксельно-чёткая) |
| `button(id, rect, текст) -> bool` | кнопка; `true` в кадре после клика |
| `hovered(id) -> bool` | курсор над кнопкой |
| `slider(id, rect, значение 0..1) -> f32` | ползунок; возвращает новое значение |
| `toggle(id, rect, подпись, вкл) -> bool` | переключатель «ВКЛ/ВЫКЛ» |
| `set_style(ButtonStyle)` | шрифт, размер, цвета, скругление и выравнивание следующих кнопок |
| `mouse() -> [f32; 2]` | курсор в единицах интерфейса |

Пока на экране есть кнопки или ползунки, курсор освобождается автоматически; когда меню исчезло — вызовите `ctx.lock_cursor(true)`.

### Свои шрифты

Укажите путь к `.ttf` / `.otf` из папки `assets` в параметре `font` или в `ButtonStyle::font` — шрифт подгружается при первом использовании. Символы, которых в шрифте нет (например, кириллица в латинском пиксельном шрифте), берутся из стандартного шрифта.

```rust
use kvas_engine::prelude::*;

const FONT: &str = "ui/fonts/terminal.ttf";

script! {
    /// Главное меню: логотип, кнопки, громкость.
    pub struct MainMenu {
        volume: f32 = 0.8,
        open: Local<bool> = Local(true),
    }

    update(self, ctx) {
        if !*self.open { return; }
        let w = ctx.ui_width();
        let ui = ctx.ui();
        ui.panel([0.0, 0.0, w, 720.0], [0.0, 0.0, 0.0, 0.6], 0.0);
        ui.image("ui/logo.png", [w / 2.0 - 180.0, 60.0, 360.0, 200.0], [1.0; 4]);
        ui.set_style(ButtonStyle { font: FONT.into(), size: 28.0, ..Default::default() });
        if ui.button("play", [w / 2.0 - 150.0, 320.0, 300.0, 48.0], "ИГРАТЬ") {
            *self.open = false;
            ctx.lock_cursor(true);
        }
        ui.text("ГРОМКОСТЬ", [w / 2.0 - 150.0, 400.0], 18.0, [1.0; 4], FONT, Align::Left);
        let v = ui.slider("volume", [w / 2.0 - 150.0, 430.0, 300.0, 18.0], self.volume);
        if ui.button("quit", [w / 2.0 - 150.0, 480.0, 300.0, 48.0], "ВЫХОД") {
            ctx.quit_game();
        }
        self.volume = v;
        ctx.set_volumes(v, v, v);
    }
}
```

### HUD поверх игры

```rust
// Полоска здоровья, иконки предметов и подсказка.
let ui = ctx.ui();
ui.frame([20.0, 670.0, 204.0, 18.0], [0.0, 0.0, 0.0, 0.5], [1.0, 1.0, 1.0, 0.4], 3.0);
ui.panel([22.0, 672.0, 200.0 * health, 14.0], [0.8, 0.15, 0.1, 1.0], 2.0);
for (i, icon) in ["ui/icons/key.png", "ui/icons/map.png"].iter().enumerate() {
    ui.image(icon, [240.0 + i as f32 * 40.0, 664.0, 32.0, 32.0], [1.0; 4]);
}
ctx.hud_prompt("[E] Открыть"); // встроенная подсказка по центру
```

Для простых подсказок есть и встроенный HUD: `hud_prompt`, `hud_message(текст, секунды)`, `hud_objective`, `hud_crosshair`.

## Сохранения и настройки

Данные игры лежат в её папке: у собранной игры — `%APPDATA%/Kvas/<имя игры>`, при игре в редакторе — `<проект>/.kvas/data`. Скрипт не может случайно выйти за её пределы.

| Метод | |
|---|---|
| `ctx.save_game(слот, заголовок, &json) -> Result` | записать слот (атомарно: сначала во временный файл) |
| `ctx.load_game(слот) -> Option<Value>` | прочитать слот |
| `ctx.save_slots() -> Vec<SaveInfo>` | все слоты, новые первыми (`slot`, `title`, `time`, `scene`) |
| `ctx.has_save(слот)` | есть ли слот |
| `ctx.storage().delete_save(слот)` | удалить |
| `ctx.storage().save_as(слот, заголовок, &T)` / `load_as::<T>(слот)` | типизированно, любой `Serialize` |
| `ctx.setting_f32 / setting_bool / setting_text(ключ, по умолчанию)` | настройка, хранящаяся между запусками |
| `ctx.set_setting(ключ, json)` | записать настройку |

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
struct Progress { chapter: u32, keys: Vec<String>, position: [f32; 3] }

// Сохранить на контрольной точке.
let p = Progress { chapter: 2, keys: vec!["red".into()], position: ctx.position().into() };
ctx.storage().save_as("autosave", "Глава 2. Склад", &p)?;

// Меню загрузки.
for (i, slot) in ctx.save_slots().iter().enumerate() {
    if ctx.ui().button(&slot.slot, [80.0, 200.0 + i as f32 * 44.0, 400.0, 40.0], &slot.title) {
        if let Some(p) = ctx.storage().load_as::<Progress>(&slot.slot) {
            ctx.teleport(p.position.into(), true);
        }
    }
}

// Настройки.
let sens = ctx.setting_f32("sensitivity", 0.1);
ctx.set_setting("sensitivity", 0.15.into());
```

## Файлы

`ctx.storage()` — файлы внутри папки игры (пути только относительные, `..` запрещено):

```rust
let fs = ctx.storage();
fs.write_text("screens/notes.txt", "первая запись")?;
fs.append_line("logs/session.txt", &format!("t = {}", ctx.elapsed()))?;
let text = fs.read_text("screens/notes.txt")?;
let names = fs.list("screens");     // имена; папки заканчиваются на '/'
if fs.exists("mods") { /* ... */ }
fs.write_bytes("cache/data.bin", &[1, 2, 3])?;
```

Внешняя файловая система компьютера (любые пути — для модов, импорта и экспорта, инструментов) — `kvas_engine::storage::external`:

```rust
use kvas_engine::storage::external;
let docs = external::documents_dir();
external::write_text(&format!("{docs}/my_game_export.json"), &json)?;
for path in external::list("C:/Games/MyMods") { /* ... */ }
let bytes = external::read_bytes("D:/textures/custom.png")?;
```

## Сцены

```rust
ctx.load_scene("scenes/level2.scene.json"); // после кадра: старые объекты и скрипты уходят
ctx.add_scene("scenes/hud.scene.json");     // добавить объекты другой сцены к текущей
ctx.reload_scene();                         // начать текущую сцену заново
let now = ctx.scene_path();
```

Время, привязки клавиш, состояние курсора и папка сохранений сохраняются; физика и экземпляры скриптов создаются заново. Чтобы передать данные в следующую сцену — сохраните их в слот или настройку.

## Камеры

| Метод | |
|---|---|
| `ctx.active_camera()` | камера, из которой смотрит игра (первая с `primary`) |
| `ctx.set_active_camera(e)` / `set_active_camera_named("Камера 2")` | переключить камеру со следующего кадра |
| `ctx.place_camera(e, позиция, цель)` | поставить и навести камеру (катсцены) |
| `ctx.set_camera_fov(e, градусы)` | угол обзора (прицел, бег) |
| `ctx.camera_position()` / `camera_forward()` | где активная камера и куда смотрит |
| `ctx.add_camera_effect(e, шейдер, params) -> индекс` | добавить шейдер камере |
| `ctx.set_camera_effect(e, индекс, вкл, params)` | менять шейдер (например, силу «урона») |

У каждой камеры свой список шейдеров (`Camera.effects`): в Инспекторе камеры — раздел «Шейдеры камеры». Они работают, пока камера активна, и идут после общих эффектов сцены. Кнопка **«Смотреть из этой камеры»** показывает сцену её глазами прямо в редакторе (Esc или движение камеры редактора — вернуться).

```rust
// Камеры наблюдения: Q/E листают, у каждой свой шейдер «security_camera».
script! {
    pub struct SecurityMonitor { index: Local<usize> = Local(0) }
    update(self, ctx) {
        let cams = ctx.find_all("Security cam");
        if cams.is_empty() { return; }
        if ctx.key_pressed(Key::E) { *self.index = (*self.index + 1) % cams.len(); }
        if ctx.key_pressed(Key::Q) { *self.index = (*self.index + cams.len() - 1) % cams.len(); }
        ctx.set_active_camera(cams[*self.index]);
    }
}
```

Миниатюры шейдеров камеры в панели «Проект» показывают тестовую сцену, пропущенную через шейдер.

## Звук: микс и реверберация

| Метод | |
|---|---|
| `play_sound(клип, громкость)` / `play_sound_at(клип, громкость, точка)` | 2D / 3D-звук |
| `play_music(клип, громкость) -> SoundId` | зацикленная музыка (своя громкость в миксе) |
| `stop_sound(id)`, `set_sound_volume(id, v)` | управление |
| `set_volumes(общая, музыка, эффекты)` | громкости микса |
| `set_reverb(доля 0..1, комната_мс)` | эхо помещения для новых звуков: 20 мс — кладовка, 40 — комната, 120 — зал |

`AudioSource` даёт пространственный звук (стерео по положению слушателя, затухание по дальности, направленный конус). Слушатель — активная камера.

```rust
// Вход в здание: короткое эхо; на улице — почти сухо и далеко.
let inside = ctx.position().x > 0.0 && ctx.position().x < 12.0;
if inside { ctx.set_reverb(0.35, 38.0); } else { ctx.set_reverb(0.12, 110.0); }
```

## Частицы

`ParticleEmitter` (Инспектор или `ParticleEmitter::preset(...)`): `fire`, `smoke`, `sparks`, `snow`, `rain`, `heavy_rain`, `drizzle`, `magic`, `dust`, `fog`, `mist`, `steam`, `embers`, `fireflies`, `leaves`, `bubbles`, `campfire_smoke`, `dust_storm`, `explosion`.

Новые поля:

| Поле | |
|---|---|
| `stretch` | частицы вытягиваются вдоль движения (струи дождя, искры) |
| `collide_floor`, `floor_height` | гибнут на высоте пола |
| `splash` | сколько брызг рождается в точке падения |
| `follow_camera` | эмиттер держится над камерой (дождь вокруг игрока, не надо покрывать весь мир) |
| `soft` | мягкие частицы: плавно исчезают там, где пересекают поверхность (метры) |

```rust
let rain = ctx.spawn("Rain");
ctx.insert(rain, Transform::at(Vec3::new(0.0, 9.0, 0.0)));
ctx.insert(rain, ParticleEmitter { floor_height: 0.0, soft: 0.2, ..ParticleEmitter::preset("heavy_rain") });
```

## Декали

`Decal` — коробка, которая проецирует изображение на всё, что внутри неё, вдоль своей оси −Y (+Y смотрит из поверхности). Неважна форма и UV поверхности: декаль ложится на ступени, сферы, бочки. Масштаб X/Z — размер, Y — глубина проекции.

| Поле | |
|---|---|
| `texture`, `normal_map`, `orm_map` | изображение (альфа вырезает форму), карта нормалей, ORM |
| `shape` | процедурная форма без картинки: Blob, Scorch, BulletHole, Cracks, Puddle, Grunge, PaintSplat, Moss, Rust, Leaks, Ring |
| `blend` | Normal, Multiply, Screen, **Lighten** (осветление), Darken, Overlay, Add, SoftLight, ReliefOnly (только рельеф), ColorDodge, ColorBurn, Tint |
| `color`, `opacity`, `roughness`, `metallic`, `emission` | вид |
| `normal_strength` | сила рельефа (карты или процедурной формы) |
| `angle_fade` | не рисовать на гранях, отвёрнутых от проекции |
| `order` | кто сверху; `max_distance` — дальше не рисуется (оптимизация) |

Лужи (`shape = Puddle`) отражают то, что на экране (SSR по глубине сцены), и рябят от капель.

**Размещение в редакторе:** меню «Создать» → «Декаль»; выделенную декаль — **Ctrl+клик** по поверхности (встаёт по нормали); картинку из «Проекта» — перетаскивание с **Alt** создаёт декаль на поверхности.

**Тысячи декалей:** каждая — одна коробка, рисуется только видимая часть экрана; декали одной картинки идут подряд (минимум переключений), далёкие отбрасываются по `max_distance`.

```rust
use kvas_engine::decal::{preset, Decal, DecalBlend};
// След от пули там, куда попал луч.
if let Some(hit) = ctx.raycast(eye, dir, 80.0) {
    let e = ctx.spawn("Bullet hole");
    let mut t = Transform::at(hit.point);
    t.rotation = Vec3::ZERO; // поверните +Y по hit.normal для стен
    t.scale = Vec3::new(0.15, 0.3, 0.15);
    ctx.insert(e, t);
    ctx.insert(e, Decal { seed: ctx.frame() as i32, ..preset("bullet_hole") });
    ctx.despawn_after(e, 30.0);
}
```

Готовые пресеты: `blood`, `scorch`, `bullet_hole`, `cracks`, `puddle`, `dirt`, `paint`, `moss`, `rust`, `leaks`, `graffiti_glow`, `light_spot`.

## Деревья и трава

`Tree` строит дерево процедурно: `style` (Broadleaf, Conifer, Palm, Bush), `height`, `trunk_radius`, `crookedness`, `levels`, `branches`, `branch_length`, `branch_angle`, `leaf_density`, `leaf_size`, цвета коры и двух оттенков листвы, `wind`, `seed`. Кора и листья качаются на GPU, края листьев сглажены (alpha-to-coverage).

`Grass`: `round` — поляна-эллипс с мягким краем, `flowers` — доля полевых цветов.

```rust
let oak = ctx.spawn("Oak");
ctx.insert(oak, Transform::at(Vec3::new(4.0, 0.0, -2.0)));
ctx.insert(oak, Tree { height: 6.0, seed: 7, ..Default::default() });
```

## Шейдеры: что видно «за» прозрачным объектом

Прозрачные объекты (альфа < 1, а также шейдеры с `water` или `glass` в имени) рисуются после непрозрачной сцены и могут читать её:

| Функция в WGSL | |
|---|---|
| `scene_depth_behind(in.clip, in.world_pos)` | сколько метров до непрозрачной поверхности позади (пена, мягкие края, толщина воды) |
| `scene_point_behind(in.clip, in.world_pos)` | точка этой поверхности в мире (каустика на дне) |
| `scene_color_behind(in.clip)` | её цвет — для преломления, искажения, режимов наложения |

Пример — стекло с преломлением:

```wgsl
@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let bend = vec2<f32>(sin(in.world_pos.y * 30.0), 0.0) * 6.0;
    let behind = scene_color_behind(vec4<f32>(in.clip.xy + bend, in.clip.zw));
    return vec4<f32>(behind * vec3<f32>(0.85, 0.95, 1.0), 1.0);
}
```

Шейдер с комментарием `// kvas:alpha_to_coverage` и альфой-покрытием даёт гладкие края вырезанных листьев; с `// kvas:decal` — рисуется как декаль.

## Сглаживание

`settings.post.antialias` (0..1, по умолчанию 1) — FXAA поверх MSAA ×4: убирает «лесенки» на тонкой геометрии и резких процедурных узорах. Текстуры имеют мип-уровни и анизотропную фильтрацию ×8.

## Коллайдеры по форме модели

У `Collider` есть формы по самой модели объекта (`MeshRenderer.mesh`):

* `ShapeKind::Mesh` — точные треугольники модели. Для неподвижной геометрии уровня: стены, пол, стойка, шкафы.
* `ShapeKind::Hull` — выпуклая оболочка модели. Для того, что двигается (`RigidBody`): стулья, ящики, бочки.

`size` для них — множитель к модели (обычно `1, 1, 1`), масштаб объекта учитывается. Модель читается один раз и кешируется.

```rust
// Стена из модели — сталкивается ровно по своей форме.
scene.world.insert(wall, Collider { shape: ShapeKind::Mesh, ..Default::default() });
// Стул, который можно толкнуть.
scene.world.insert(chair, Collider { shape: ShapeKind::Hull, ..Default::default() });
scene.world.insert(chair, RigidBody { mass: 6.0, ..Default::default() });
```

Луч только по зонам-триггерам (например, найти предмет за тонким бортиком стойки) — `QueryOptions { only_triggers: true, .. }`; затем второй луч по твёрдому проверяет, не загорожен ли он.
