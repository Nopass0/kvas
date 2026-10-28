# Kvas: руководство с примерами

Это подробный путеводитель по редактору и движку. Справочник по Rust API лежит в
[ENGINE_API.md](ENGINE_API.md), список планов и ограничений - в [ROADMAP.md](https://github.com/Nopass0/kvas/blob/main/ROADMAP.md).

Содержание

1. [Быстрый старт](#1-быстрый-старт)
2. [Скрипты](#2-скрипты)
3. [Шейдеры объектов](#3-шейдеры-объектов)
4. [Шейдеры камеры](#4-шейдеры-камеры)
5. [Материалы из узлов](#5-материалы-из-узлов)
6. [Камера, свет, небо, туман](#6-камера-свет-небо-туман)
7. [Частицы, вода, дым, рисование по текстуре](#7-частицы-вода-дым-рисование-по-текстуре)
8. [Звук](#8-звук)
9. [Апскейлинг и производительность](#9-апскейлинг-и-производительность)
10. [Сборка игры](#10-сборка-игры)
11. [Если что-то сломалось](#11-если-что-то-сломалось)

---

## 1. Быстрый старт

1. Откройте редактор, нажмите **New project**. В проекте сразу лежат:
   - `assets/scenes/main.scene.json` - сцена с землёй и кубом;
   - `scripts/src/` - ваши скрипты (по файлу на скрипт);
   - `assets/shaders/library/` - готовые шейдеры (около 40 штук);
   - `assets/shaders/kvas_mesh_api.wgsl`, `kvas_camera_api.wgsl` - объявления для редактора (см. раздел 3);
   - `tools/blender/export_kmat.py` - экспорт материалов из Blender.
2. Создайте объект: **GameObject -> 3D -> Cube** (или перетащите модель из Assets в сцену).
3. Перетащите скрипт из Assets на объект или на его Inspector - скрипт прикрепится.
4. **Play** запускает игру прямо в редакторе, **Ctrl+S** сохраняет сцену.

Перетаскивание работает везде: скрипт на объект, шейдер на `MeshRenderer`, текстура на поле текстуры,
звук на поле звука, модель в сцену, шейдер камеры в список эффектов камеры.

## 2. Скрипты

### 2.1 Один скрипт - один файл

Каждый скрипт лежит в своём файле внутри `scripts/src/` (можно в подпапках:
`scripts/src/enemies/zombie.rs`). **Ничего регистрировать не нужно.** При сборке `scripts/build.rs`
находит все файлы, подключает их как модули и регистрирует каждый тип, у которого есть
`impl Script for Имя`.

Создать скрипт: правый клик в Assets -> **Create -> Script**, или меню **Scripts**.
Редактор создаст файл по шаблону, откроет его и сам пересоберёт библиотеку.
Двойной клик по скрипту открывает файл во **встроенном редакторе Kvas** (Settings -> Code editor позволяет выбрать VS Code, RustRover и др.): вкладки,
подсветка Rust/WGSL, номера строк, поиск и замена (Ctrl+F), переход к строке (Ctrl+G), сохранение (Ctrl+S) с автопересборкой, подсказки по API
(Tab вставляет первую).

Требования к скрипту:

- структура `pub struct Имя` (с `pub`!) и `impl Script for Имя`;
- имя в стиле CamelCase (`DoorOpener`), файл удобно назвать `door_opener.rs`;
- `use kvas_engine::prelude::*;` - всё нужное приходит из prelude.

Проекты старого формата (с `register_scripts!(...)` в `lib.rs`) конвертируются при открытии:
скрипты из `lib.rs` раскладываются по отдельным файлам.

### 2.2 Минимальный скрипт

```rust
use kvas_engine::prelude::*;

script_params! {
    /// Вращает объект. Поля ниже появятся в Inspector.
    pub struct Spin {
        speed: f32 = 90.0,        // градусов в секунду
        axis: Vec3 = Vec3::Y,
    }
}

impl Script for Spin {
    fn update(&mut self, ctx: &mut Ctx) {
        let dt = ctx.dt();
        ctx.transform_mut().rotation += self.axis * self.speed * dt;
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
```

`script_params!` объявляет структуру, `Default` и список полей для Inspector. Поддерживаемые типы:
`f32`, `i32`, `bool`, `String`, `Vec3`, `Color`-подобные массивы, `EntityRef` (ссылка на объект по имени),
`SoundRef` (звук из проекта), а также `Local<T>` - внутреннее состояние, которое в Inspector не показывается.

### 2.3 Жизненный цикл

| Метод | Когда вызывается |
|-------|------------------|
| `start(ctx)` | один раз перед первым `update` |
| `update(ctx)` | каждый кадр |
| `fixed_update(ctx)` | с фиксированным шагом физики (60 Гц) |

### 2.4 Рецепты

**Ввод.** Именованные действия настраиваются в Project Settings -> Input.

```rust
let dir = ctx.action_vector("MoveLeft", "MoveRight", "MoveBack", "MoveForward");
if ctx.action_pressed("Jump") { ctx.add_impulse(Vec3::Y * 6.0); }
if ctx.key_pressed(Key::E) { /* нажали E в этом кадре */ }
```

**Найти другой объект и отправить сообщение.**

```rust
if let Some(player) = ctx.find("Player") {
    let pos = hierarchy::global_matrix(&ctx.scene.world, player).w_axis.truncate();
    if ctx.world_position().distance(pos) < 1.5 {
        let _ = ctx.emit("collected", json!({}));   // слушает, например, GameManager
        let me = ctx.entity;
        ctx.despawn(me);
    }
}
```

**Создать объект во время игры.**

```rust
let e = ctx.spawn_primitive("Box", "cube", ctx.world_position() + Vec3::Y * 2.0);
let world = ctx.world();
world.insert(e, RigidBody::default());
world.insert(e, Collider::default());
```

**Луч (raycast) и столкновения.**

```rust
let on_ground = ctx.raycast(ctx.world_position(), Vec3::NEG_Y, 0.65).is_some();
```

**Звук.** `ctx.play_sound_at("sounds/hit.wav", 1.0, position)` - трёхмерный звук, затухающий с расстоянием.

**Время суток, небо, постобработка** - см. раздел «Звук, время, небо и эффекты в скриптах» в ENGINE_API.md.

### 2.5 Готовые скрипты

В проект кладётся библиотека (меню **Scripts -> Install starter library** добавит её в старый проект):

| Папка | Скрипты |
|-------|---------|
| `movement` | Spinner, Bobber, Patrol, Orbit, FlyCamera, FollowCamera, TopDownMover, Jumper, **FpsPlayer** |
| `gameplay` | Collectible, ScoreCounter, Respawn, Spawner, LookAt |
| `camera` | CameraShake, ZoomOnKey |
| `audio` | AmbientLoop |
| `world` | DayNightCycle, NightLight |
| `ui` | WelcomeMessage |

Каждый файл - короткий образец: откройте и скопируйте нужное.

### 2.6 Ошибки в скриптах не роняют редактор

- Ошибка **компиляции**: сборка пишет сообщение в консоль, редактор продолжает работать со
  старой версией скриптов. Исправьте файл - сборка запустится сама.
- **Паника** внутри `start/update/fixed_update`: сработавший экземпляр отключается, остальные
  работают; в консоли будет сообщение с именем скрипта.
- Скрипт с неверным ABI (собран другой версией движка) не загружается, об этом сообщает консоль.

## 3. Шейдеры объектов

Шейдер - это `.wgsl` файл. Назначьте его на `MeshRenderer -> Shader` (перетащите файл на поле или на объект).
Любое изменение файла применяется сразу. **Если шейдер с ошибкой, объект рисуется обычным шейдером,
а сообщение об ошибке с номером строки появляется в консоли.** Редактор и игра не падают.

### 3.1 Что вы пишете

Две функции: `vs_main` и `fs_main`. Всё остальное (структуры, uniform-ы, освещение, туман) движок
подставляет перед вашим кодом сам.

```wgsl
#include "shaders/kvas_mesh_api.wgsl"   // только для подсказок редактора, см. ниже

@vertex
fn vs_main(v: VertexIn) -> VsOut { return standard_vertex(v); }

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let n = normalize(in.normal);
    let base = object.color.rgb;                       // цвет из MeshRenderer
    let lit = lighting(in.world_pos, n, base, 0.7);    // солнце + лампы + тени + небо
    let col = apply_fog(lit, in.world_pos);            // туман сцены
    return vec4<f32>(to_display_at(col, in.clip), object.color.a);   // тонемаппинг и грейдинг
}
```

### 3.2 Файлы `kvas_mesh_api.wgsl` и `kvas_camera_api.wgsl`

Редакторы WGSL (RustRover, VS Code с wgsl-analyzer) видят только один файл и подчёркивают
`globals`, `object`, `lighting` как неопределённые. Поэтому в **каждый проект** кладутся два
сгенерированных файла в `assets/shaders/`. Добавьте первой строкой шейдера

```wgsl
#include "shaders/kvas_mesh_api.wgsl"
```

и редактор увидит все имена. Движок эту строку игнорирует. Файлы обновляются при каждом открытии
проекта (не правьте их), внутри - подробные комментарии «как писать шейдер» и полный текст prelude.
Для шейдеров камеры используйте `kvas_camera_api.wgsl`.

### 3.3 Что доступно

| Имя | Что это |
|-----|---------|
| `globals.time.x` | секунды с начала игры |
| `globals.camera_pos`, `sun_dir`, `sun_color`, `ambient` | камера и освещение |
| `object.color` | цвет и прозрачность из `MeshRenderer` |
| `object.user0`, `object.user1` | 8 свободных чисел `MeshRenderer.user[0..8]` (поле «user» в Inspector) |
| `object.params.x/y/z` | шероховатость, свечение, повтор текстуры |
| `albedo_tex`, `normal_tex`, `orm_tex` | текстуры материала |
| `in.world_pos`, `in.normal`, `in.uv`, `in.clip` | данные точки поверхности |
| `lighting(p, n, base, roughness)` | простое освещение |
| `lighting_pbr(p, n, albedo, rough, metal, ao)` | PBR-освещение |
| `shadow_factor(p, n)` | тень от солнца (0..1) |
| `env_color(dir)` | цвет неба/окружения в направлении (для отражений) |
| `apply_fog(col, p)` | туман |
| `to_display_at(col, in.clip)` | перевод в экранные цвета |
| `hash21`, `hash31`, `value_noise`, `fbm2(p, octaves)`, `fbm3` | шум |

Параметры из Inspector: чтобы шейдер был настраиваемым, читайте `object.user0.x` и т. д.
и подставляйте разумное значение по умолчанию: `select(object.user0.x, 3.0, object.user0.x <= 0.0)`.

### 3.4 Прозрачность и вершины

- Объект с `color.a < 1` рисуется прозрачным: в `fs_main` верните нужную альфу.
- Колыхание травы, волны и т. п. делаются в `vs_main`: возьмите `standard_vertex(v)` и измените
  `out.clip` / `out.world_pos` (пример - `wind_sway.wgsl`).

### 3.5 Пресеты

Все лежат в `assets/shaders/library/` и открываются двойным кликом.

- Стиль: `toon`, `outline_toon`, `unlit_color`, `rim_light`, `neon_glow`, `tron_grid`
- Материалы: `gold_metal`, `pearl`, `marble`, `wood_planks`, `bricks`, `checker_floor`, `sand_ripples`, `moss_on_top`, `glass`, `ice`
- Эффекты: `hologram`, `force_field`, `dissolve`, `fire`, `lava`, `snow_cover`, `wind_sway`
- Служебные: `height_gradient`, `rainbow_height`, `triplanar_blend`

### 3.6 Подключение общих функций

```wgsl
#include "builtin:lib/color"    // lib_luma, lib_hue_shift, lib_saturation, lib_contrast, lib_vignette...
#include "builtin:lib/noise"    // lib_hash21, lib_noise, lib_fbm
#include "builtin:lib/math"     // lib_remap, lib_pulse, lib_rot2
#include "shaders/my_funcs.wgsl" // ваш файл с функциями (каждый файл подключается один раз)
```

Свои функции можно вынести в отдельный файл и вызывать из любого шейдера - так же, как скрипты в Rust.

## 4. Шейдеры камеры

Накладываются на готовую картинку камеры: **Scene settings -> Camera shaders -> Add**, либо перетащите
файл из Assets на камеру. Шейдер определяет одну функцию:

```wgsl
#include "shaders/kvas_camera_api.wgsl"
#include "builtin:lib/color"

// uv - позиция на экране 0..1, color - текущий цвет пикселя, params - 4 числа из Inspector,
// time - секунды. Вернуть новый цвет.
fn effect(uv: vec2<f32>, color: vec3<f32>, params: vec4<f32>, time: f32) -> vec3<f32> {
    let k = params.x;
    return mix(color, vec3<f32>(lib_luma(color)), k) * lib_vignette(uv, 0.3 * k);
}
```

`scene_color(uv)` читает любой другой пиксель (размытие, искажения), `pixel_size()` - размер пикселя в uv.
Эффекты применяются по порядку списка. Пресеты в `assets/shaders/library/fx/`:
`vhs`, `crt`, `glitch`, `film_grain`, `sepia`, `black_white`, `night_vision`, `thermal`, `posterize`,
`pixelate`, `sharpen`, `edge_outline`, `tilt_shift`, `dream_glow`, `teal_orange`, `cinematic_bars`,
`underwater`, `damage_flash`, `chromatic_pop`, `drunk`.

Из скрипта эффектами можно управлять: `ctx.add_post_effect("shaders/library/fx/damage_flash.wgsl")`,
`ctx.set_post_effect_params(index, [1.0, 0.0, 0.0, 0.0])`, `ctx.enable_post_effect(index, false)`,
`ctx.clear_post_effects()` - например, `damage_flash` включается при получении урона.

## 5. Материалы из узлов

Файл `.kmat` - граф узлов. Создать: Assets -> Create -> **Material** (или GameObject -> Materials).
Двойной клик открывает редактор узлов:

- правый клик по полю - добавить узел; тяните от круглого гнезда к другому, чтобы соединить;
- Delete удаляет выбранный узел; колесо мыши приближает;
- справа - живой предпросмотр и список изображений для узлов «Текстура»;
- результат подключается к узлу **Output** (Base color, Roughness, Metallic, Emission, Alpha, Occlusion, Normal, Height);
- шумы и узоры показывают превью прямо в узле; **Color ramp**, режимы наложения, карты нормалей и высот, свет и углы поверхности - см. [нодовый редактор](nodes/index.md) и [справочник нод](nodes/reference.md).

Назначить материал: перетащить `.kmat` на объект. Из Blender материал можно перенести скриптом
`tools/blender/export_kmat.py` (Scripting -> Run Script в Blender).

## 6. Камера, свет, небо, туман

На компоненте **Camera**: поле зрения или фокусное расстояние, выдержка, диафрагма, глубина резкости
(ручной фокус или **автофокус** на объект под центром кадра), боке, виньетка, хроматическая аберрация,
кинематографический режим. Настройки сцены (**Scene settings**) содержат небо и время суток, облака,
звёзды, туман, тени (разрешение, дальность, мягкость), постобработку (bloom, экспозиция, цветокоррекция)
и дальность прорисовки (`view_distance`) с автоматической подгрузкой и выгрузкой далёких объектов.

Из скриптов: `ctx.set_time_of_day(hour)`, `ctx.sky()` (небо и туман), `ctx.post()` (постобработка),
`ctx.post_preset("name")` - см. ENGINE_API.md.

## 7. Частицы, вода, дым, рисование по текстуре

- **Частицы:** GameObject -> Effects -> Particles; пресеты: огонь, дым, искры, снег, дождь, магия,
  туман и др. (около 25). Все параметры (скорость, размер, цвет по времени, гравитация) в Inspector.
- **Вода:** компонент `VoxelWater` - воксельная жидкость, плавающие тела получают выталкивающую силу
  (`RigidBody` + `Collider` на объекте).
- **Рисование по текстуре:** GameObject -> Paint Surface; в панели Paint выберите слой (до 4 текстур),
  кисть, размер и мягкость - границы смешиваются без швов. Результат хранится в сцене.
- **Модификаторы мешей** (как в Blender): Bend, Twist, Taper, Wave, Noise, Subdivide, Mirror, Array,
  Solidify и др. - компонент `MeshModifiers`, порядок можно менять.

## 8. Звук

Файлы wav/ogg/mp3/flac. Компонент **AudioSource** (2D или 3D с затуханием), панель **Audio** показывает
волну и позволяет прослушать. В скрипте: `ctx.play_sound("sounds/click.wav", 1.0)`,
`ctx.play_sound_at(path, volume, position)`, музыка и петли - через `AmbientLoop`.

## 9. Апскейлинг и производительность

**Scene settings -> Performance:**

| Настройка | Что делает |
|-----------|------------|
| Upscaler: Bilinear / FSR-style | рендер с меньшим разрешением и повышение до экрана; резкость регулируется |
| DLSS / Frame Generation | выбор доступен в списке; требуют NVIDIA Streamline, см. ограничения в ROADMAP.md |
| View distance | дальше этого расстояния объекты не рисуются и выгружаются из памяти |
| Shadows | разрешение, дальность, мягкость, отсечение невидимых |
| MSAA | сглаживание |

Практические советы: заменяйте множество одинаковых объектов одним мешем, используйте
пониженное разрешение рендера + апскейлер, ограничивайте `view_distance` в больших сценах,
собирайте игру в режиме **Release**.

## 10. Сборка игры

Кнопка **Build Game** (панель сверху или меню) открывает окно сборки:

- **иконка приложения** (PNG/JPG/ICO) - попадает в exe, на панель задач и в окно;
- заголовок окна, имя файла, версия, компания;
- размер окна, полный экран, развернуть окно, изменение размера, VSync;
- **оптимизированная (release) сборка** - рекомендуется для выпуска.

Нажмите **Build**. Результат в `<проект>/build/`:

```
mygame.exe        - игра
game.pak, game.000.pkd, ...   - упакованные ресурсы (делятся на тома по размеру)
mygame_scripts.dll - ваши скрипты
icon.png          - иконка окна
README.txt
```

Игра **автономна**: ей не нужны ни редактор, ни Rust, ни исходники. Копируйте всю папку `build` целиком.
После сборки редактор сам запускает игру на несколько секунд и пишет в консоль, прошла ли проверка.

## 11. Если что-то сломалось

| Симптом | Что делать |
|---------|-----------|
| Скрипт не появился в списке | проверьте `pub struct Имя` и `impl Script for Имя`, смотрите ошибки сборки в консоли |
| Скрипт «отключён» | в консоли есть сообщение о панике, исправьте код; перезапустите Play |
| Объект розовый/обычный вместо моего шейдера | ошибка в шейдере: консоль покажет строку |
| Редактор подчёркивает `globals` или `lighting` | добавьте `#include "shaders/kvas_mesh_api.wgsl"` первой строкой |
| Собранная игра не открывается | запустите её из терминала - ошибка печатается; проверьте, что рядом лежат `game.pak` и `*_scripts.dll` |
| Долгий первый запуск в debug | запускайте release (`cargo run --release -p kvas_editor`) |
