# Руководство по Rust API Kvas

Это руководство входит в rustdoc движка. Все примеры `rust`, кроме явно помеченных
`no_run`, исполняются командой `cargo test -p kvas_engine --doc`.

## Устройство и единицы

- `Scene` владеет ECS-миром, настройками окружения и очередью сообщений `SignalBus`.
- `Runtime` запускает одинаковую симуляцию в редакторе и самостоятельном player.
- `Registry` описывает компоненты для Inspector, JSON и копирования сущностей.
- `Assets` одинаково читает каталог проекта и архив `game.pak`.
- `Script` описывает поведение; `Ctx` обращается к состоянию хоста.

Мир правосторонний: Y направлен вверх, локальный вперёд — минус Z. Позиции и размеры
коллайдеров задаются в метрах. `Transform.rotation` хранит градусы, порядок вращений
Y-X-Z. Угловая скорость физического тела задаётся в радианах в секунду. Время — в
секундах; частоты `Bob`, `Pulse`, `ColorCycle` — в циклах в секунду.

`Entity` содержит индекс и поколение. Он действителен только для своего мира.
Нельзя использовать индекс как постоянный идентификатор, переносить handle между
мирами или сохранять `to_bits()` в проект как ссылку на объект. Загрузка сцены
создаёт новые handles; имена и теги помогают найти объекты после загрузки.

## Минимальная симуляция без окна

```rust
use kvas_engine::{prelude::*, scene::Scene, runtime::Runtime};

let mut scene = Scene::default(); // все встроенные ECS-компоненты зарегистрированы
let entity = scene.spawn("Box"); // Name + Transform
scene.world.insert(entity, MeshRenderer::default());
scene.world.insert(entity, Spin { speed: Vec3::new(0.0, 90.0, 0.0) });
scene.world.insert(entity, Collider::default());
scene.world.insert(entity, RigidBody::default());

let mut runtime = Runtime::new(scene, None);
runtime.tick(1.0 / 60.0);
assert!(runtime.scene.world.get::<Transform>(entity).unwrap().rotation.y > 0.0);
```

`Runtime::tick` принимает время реального кадра. Невалидный, нулевой или отрицательный
интервал пропускается. Слишком большой интервал ограничивается `max_frame_dt`, после
чего применяется `time_scale`. В принятом кадре выполняются:

1. Очистка однокадрового HUD и публикация сообщений прошлого кадра.
2. Создание/удаление экземпляров скриптов и вызовы `start`/`update`.
3. Удаление объектов с истёкшим `Lifetime`.
4. Обновление неба, взаимодействий, модификаторов, tween и процедурных систем.
5. Ноль или несколько `fixed_update` и шагов физики.
6. Обновление мировых матриц.

Физические запросы из скрипта читают последний завершённый шаг физики. Созданный
коллайдер или изменённая маска вступят в силу на следующем фиксированном шаге.

## Пауза, одиночный шаг и ограничение нагрузки

```rust
use kvas_engine::{runtime::{Runtime, RuntimeConfig}, scene::Scene};

let config = RuntimeConfig { fixed_dt: 1.0 / 120.0,
    max_frame_dt: 0.1, max_fixed_steps: 16 };
let mut runtime = Runtime::with_config(Scene::default(), None, config).unwrap();
runtime.set_time_scale(0.5).unwrap();
runtime.tick(0.02);
assert!((runtime.time.dt - 0.01).abs() < 1e-6);
runtime.set_paused(true);
runtime.step_once(); // один кадр длиной fixed_dt, независимо от pause/time_scale
let statistics = runtime.frame_stats();
assert!(statistics.fixed_steps >= 1);
```

При достижении `max_fixed_steps` лишние целые интервалы отбрасываются, дробный
остаток сохраняется. `FrameStats.dropped_time` показывает потерянное время,
`interpolation_alpha` — долю следующего шага; автоматической интерполяции поз нет.
Нулевой `time_scale` замораживает обычные кадры. Пауза сохраняет дробный остаток.

## ECS: регистрация, запросы и владение

Встроенные типы уже зарегистрированы в `Scene::default`. Собственные типы допустимы
в постоянном коде хоста; регистрировать storage из выгружаемой DLL нельзя: его vtable
осталась бы указывать в выгруженный модуль. `Component::NAME` должен быть уникален.
Проверки имени Rust-типа, размера и выравнивания ловят обычные конфликты до type cast.
Они не заменяют требование совместимого компилятора и исходников для native DLL.

```rust
use kvas_ecs::{Component, World};
#[derive(Debug)] struct Health(i32);
struct Damage(i32);
impl Component for Health { const NAME: &'static str = "game.Health"; }
impl Component for Damage { const NAME: &'static str = "game.Damage"; }

let mut world = World::new();
world.register::<Health>(); world.register::<Damage>();
let entity = world.spawn();
assert!(world.try_insert(entity, Health(100)).unwrap().is_none());
world.insert(entity, Damage(7));
world.for_each2_mut::<Health, Damage>(|_, health, damage| health.0 -= damage.0);
assert_eq!(world.get::<Health>(entity).unwrap().0, 93);
assert!(world.get2_mut::<Health, Health>(entity).is_none()); // запрет mutable alias
```

- `iter`, `iter_mut`, `iter2`, `iter3` обходят плотные storage без snapshot сущностей.
- `entities_with*`, `entities_without` создают snapshot, позволяющий менять структуру мира.
- `get2_mut` безопасно заимствует разные storage; одинаковые ключи дают `None`.
- `get_or_insert_with` не вызывает фабрику, если значение есть или handle устарел.
- `retain<T>` удаляет отвергнутые компоненты, сохраняя сущности.
- `try_insert` возвращает заменённый компонент либо отклонённое новое значение.
- `clear` уничтожает сущности, сохраняя регистрации. При исчерпании поколения слот
  выводится из оборота, поэтому старый handle не оживает после переполнения.

Порядок плотного storage меняется при удалении. `iter_entities` и `entities` идут
по индексам. Handles из разных миров могут иметь одинаковые биты; сравнение не
проверяет принадлежность миру.

## Иерархия и координаты

```rust
use kvas_engine::{prelude::*, scene::Scene};
let mut scene = Scene::default();
let parent = scene.spawn("Parent"); let child = scene.spawn("Child");
scene.world.get_mut::<Transform>(parent).unwrap().position = Vec3::X * 10.0;
assert!(hierarchy::set_parent_local(&mut scene.world, child, Some(parent)));
assert_eq!(hierarchy::global_matrix(&scene.world, child).w_axis.x, 10.0);
assert!(hierarchy::set_parent(&mut scene.world, child, None));
assert_eq!(scene.world.get::<Transform>(child).unwrap().position.x, 10.0);
assert!(!hierarchy::set_parent_local(&mut scene.world, child, Some(child)));
```

`set_parent` сохраняет мировую позу и отклоняет вырожденные матрицы и shear, который
невозможно точно представить через TRS. `set_parent_local` сохраняет локальную позу,
поэтому допускает родителя с нулевым масштабом. Обе функции проверяют живые handles
и циклы без изменения мира при отказе. Обходы защищены от циклов, даже если `Parent`
был вставлен напрямую. Загрузчик предупреждает о некорректных связях и отбрасывает их.

`GlobalTransform` — кэш. После ручного редактирования вне `Runtime` вызывайте
`hierarchy::update_globals`; для немедленного чтения используйте `global_matrix`.
Полное обновление корректного дерева использует кэширование цепочек и не имеет
прежнего ограничения глубины 64. `despawn_recursive` включает transient-потомков.

`Transform` предоставляет `look_at`, `transform_point`, `transform_direction`,
`inverse_transform_point`, `try_from_matrix` и `interpolate` (quaternion slerp по
кратчайшей дуге). `look_at` принимает target/up в пространстве родителя.

## Ввод: состояния и фронты

Хост вызывает `Input::update` ровно один раз перед кадром симуляции. `pressed` и
`released` вычисляются сравнением снимков, чтение их не потребляет. Если один кадр
содержит несколько `fixed_update`, они видят одинаковые фронты. Однократные действия
обрабатывайте в `update`, а силы/движение — в `fixed_update`.

```rust
use kvas_engine::input::{Input, Key};
use glam::Vec2;
let mut input = Input::default();
input.try_rebind("Jump", &["Space"]).unwrap();
input.update([Key::Space].into_iter().collect(), Default::default(), Vec2::ZERO, 0.0);
assert!(input.action_pressed("Jump"));
input.update(Default::default(), Default::default(), Vec2::ZERO, 0.0);
assert!(input.action_released("Jump"));
input.reset(); // при потере фокуса: очищает кнопки/фронты/движение и снимает cursor lock
```

`action_vector(left,right,down,up)` ограничивает длину единицей: диагональное движение
не быстрее осевого. `action_released` срабатывает только после отпускания последнего
удерживаемого альтернативного источника. `try_rebind` отклоняет опечатки атомарно;
совместимый `rebind` молча игнорирует неизвестные физические источники. Перепривязки
в памяти не сохраняются автоматически в `project.json`.

## Физика, слои и телепортация

`Collider.size` — полный размер: размеры box, диаметр sphere, диаметр и полная высота
capsule/cylinder. Коллайдер без собственного `RigidBody` присоединяется к ближайшему
телу-предку; без такого предка он статический. Изменения форм, размеров, трения,
trigger-флага, слоёв и перепривязки синхронизируются на физическом шаге. Удаление
компонентов удаляет соответствующие Rapier-объекты; изменения Joint пересоздают связь.

```rust
use kvas_engine::{prelude::*, runtime::Runtime, scene::Scene};
let mut scene = Scene::default(); let ground = scene.spawn("Ground");
scene.world.insert(ground, Collider { size: Vec3::new(10.0, 1.0, 10.0),
    memberships: 1, filter: u32::MAX, ..Default::default() });
let mut runtime = Runtime::new(scene, None); runtime.tick(1.0 / 60.0);
let hit = runtime.physics.raycast_with_options(Vec3::Y * 5.0, Vec3::NEG_Y, 20.0,
    QueryOptions { filter: 1, ..Default::default() }).unwrap();
assert_eq!(hit.entity, ground);
```

Для взаимодействия двух объектов должны одновременно выполняться
`(a.memberships & b.filter) != 0` и `(b.memberships & a.filter) != 0`.
`QueryOptions` применяет то же правило. Обычные raycast игнорируют triggers;
`overlap_sphere` включает их. Настраиваемые варианты позволяют изменить это.
Исключение сущности-владельца тела исключает все его составные коллайдеры.
`touching(entity)` возвращает активные контакты и sensor-overlaps последнего шага.

Импульсы и запросы скоростей потребляются следующим dynamic-шагом. Чтобы перенести
уже созданное dynamic-тело, используйте `ctx.teleport`/`teleport_of`: они преобразуют
мировую позицию в локальную и ставят `RigidBody.teleport`. В коде хоста можно изменить
Transform и установить этот флаг самостоятельно. С `reset_velocity = true` очищаются
импульс и обе скорости. `ccd = true` помогает быстрым телам не проходить сквозь тонкие
стены. CCD требует дополнительной работы физического solver.

## Теги, время жизни, видимость и tween

```rust
use kvas_engine::{prelude::*, scene::Scene, runtime::Runtime};
let mut scene = Scene::default(); let entity = scene.spawn("Lift");
let mut tags = Tags::default(); tags.insert("platform"); tags.insert("interactive");
scene.world.insert(entity, tags);
let mut tween = MoveTween::to(Vec3::Y * 3.0, 2.0).unwrap();
tween.mode = TweenMode::PingPong; tween.ease = Ease::SineInOut;
scene.world.insert(entity, tween);
scene.world.insert(entity, Visibility { visible: true });
scene.world.insert(entity, Lifetime::new(30.0).unwrap());
let mut runtime = Runtime::new(scene, None); runtime.tick(0.1);
assert!(runtime.scene.world.get::<Transform>(entity).unwrap().position.y > 0.0);
```

- Теги регистрозависимы, разделяются пробелами и не должны содержать пробельные символы.
- `Lifetime` удаляет всё поддерево, учитывает паузу и масштаб времени.
- `Visibility` наследуется потомками и скрывает meshes/shadows; физика, скрипты и
  источники света продолжают работать.
- `MoveTween` работает в локальном пространстве, поддерживает delay, Once/Repeat/
  PingPong и 14 easing-кривых. Для физической платформы используйте Kinematic body.
- `restart` возвращается к захваченному origin; `restart_from_current` захватывает
  текущую позицию на следующем обновлении. Нулевой duration один раз ставит target.

## Таймеры и локальные очереди

```rust
use kvas_engine::prelude::{Timer, TimerMode, EventQueue};
use std::time::Duration;
let mut timer = Timer::new(Duration::from_millis(100), TimerMode::Repeating);
timer.tick(Duration::from_millis(250));
assert_eq!(timer.times_finished(), 2);
assert_eq!(timer.elapsed(), Duration::from_millis(50));
let mut queue = EventQueue::new(1);
queue.push("ready").unwrap();
assert_eq!(queue.push("overflow"), Err("overflow"));
assert_eq!(queue.pop(), Some("ready"));
```

Таймеры не используют wall clock. `tick_secs` игнорирует невалидные dt, очищая фронт
завершения. Одноразовый таймер сохраняет `finished`, но `just_finished` истинно только
после завершающего tick. Повторяющийся считает все пройденные интервалы. Нулевой
интервал завершает таймер один раз за неприостановленный tick, без бесконечного цикла.
`EventQueue<T>` — локальный FIFO; для общения разных скриптов используйте SignalBus.

## Сообщения между скриптами

```rust
use kvas_engine::{scene::Scene, runtime::Runtime};
let mut scene = Scene::default();
scene.signals.send("door.open", serde_json::json!({"key": 3}), None, None).unwrap();
let mut runtime = Runtime::new(scene, None); runtime.tick(0.01);
assert_eq!(runtime.scene.signals.channel("door.open").count(), 1);
runtime.tick(0.01);
assert!(runtime.scene.signals.incoming().is_empty());
```

Скрипт вызывает `ctx.emit(channel, json)` для broadcast или `ctx.send_to(entity,
channel, json)` для адресного сообщения. `ctx.signals(channel)` возвращает собственный
snapshot подходящих сообщений, чтобы во время обработки можно было менять сцену.
Очередь ограничена (по умолчанию 256 в каждом из двух буферов); ошибки отправки нельзя
игнорировать, если событие критично для игровой логики. Чтение не потребляет сообщения,
поэтому каждый слушатель видит свой broadcast. Отправитель к моменту доставки может
быть уже удалён; проверяйте `ctx.is_alive`, если нужен его текущий компонент.

## Скрипты и горячая перезагрузка

```rust
use kvas_engine::prelude::*;
#[derive(Default)] struct Spinner;
impl Script for Spinner {
    fn update(&mut self, ctx: &mut Ctx) {
        let delta = ctx.dt();
        ctx.transform_mut().rotation.y += 90.0 * delta;
    }
}
// В cdylib проекта экспортируйте register_scripts!(Spinner).
// Для постоянно слинкованного хоста/теста DLL не нужна:
use kvas_engine::script::{ScriptDef, ScriptHost};
let host = ScriptHost::from_defs(vec![ScriptDef::of::<Spinner>("Spinner")]).unwrap();
assert!(host.create("Spinner").is_some());
```

Сначала вызывается `start`, затем `update` или `fixed_update`. Удаление владельца/slot
в `start` предотвращает дальнейший callback. Паники фабрики, fields и callbacks
изолируются; неудачный экземпляр отключается, другие продолжают работу. Это работает
с unwinding: native crash или `panic=abort` невозможно превратить в обычную ошибку.
`restart_instances` пересоздаёт экземпляры из сохранённых props. Ошибки отсутствующих
типов/инициализации не спамят каждый кадр, повторная попытка делается после явного
restart/reload или нового прикрепления slot.

Экземпляр, полученный `ScriptHost::create`, удерживает native library до собственного
уничтожения. `names()` заимствует строки на время жизни хоста; `names_owned()` позволяет
сохранить имена после reload. Перед обменом Rust-контейнерами проверяется C-совместимый
ABI probe. Требуются одинаковые compiler, engine sources, зависимости, target и
совместимые build flags; это native Rust API, а не стабильный C ABI или sandbox.
После обновления движка пересоберите host и все scripts DLL. Старый модуль без нового
probe отклоняется с требованием rebuild. Состояние обычных полей скрипта при reload
сбрасывается; сериализованные props применяются при создании нового экземпляра.

## Assets, архивы и сохранение

```rust,no_run
use kvas_engine::{assets::Assets, scene::Scene, registry::Registry};
use std::path::Path;
let assets = Assets::open_pak(Path::new("game.pak"))?;
let bytes = assets.read_bytes("models/character.glb")?; // Cow::Borrowed для .pak
assert!(assets.exists("models/character.glb"));
let models = assets.list_prefix("models");
let registry = Registry::default();
let (scene, warnings) = Scene::load(&registry, Path::new("main.scene.json"))?;
scene.save(&registry, Path::new("saved.scene.json"))?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

Пути нормализуются относительно assets root; поддерживаются внутренние `..`, но
переход выше root, абсолютные/drive-пути и NUL отклоняются. Чтение loose-файла также
проверяет canonical path, чтобы symlink не вывел за корень. Listing/pack пропускают
скрытые записи и symlinks. `try_list` сохраняет ошибки обхода, `list` возвращает пустой
список при ошибке. Упаковщик использует строгий вариант и не выдаёт успешный пустой
архив вместо ошибки отсутствующего каталога.

Архив проверяет имена, дубли и границы таблицы/данных. `pack` сортирует entries,
исключает старый выходной архив из исходного каталога и позволяет inline extras
переопределять обычные файлы; последний duplicate extra побеждает. Проект, сцена и
архив записываются во временный соседний файл с последующей заменой. Ошибка до замены
оставляет прежний destination целым. Это не межпроцессная блокировка и не гарантия
сохранения directory entry при внезапном отключении питания.

`Scene::validate` сообщает о проблемах persistence, отсутствующих Transform,
неживых родителях и циклах. `try_to_json`/`save` отклоняют встроенные значения, которые
не могут пройти JSON round-trip (например NaN/Infinity), вместо сохранения null.
Совместимый `to_json` при ошибке возвращает пустую строку: для записи файлов выбирайте
`save` или явно обрабатывайте `try_to_json`.

## Меши, spatial и renderer

`MeshData` поддерживает validate, пересчёт normals, affine transform с inverse-transpose
нормалями и исправлением winding при зеркале, append с проверкой u32 indices и
surface_area. При append материал остаётся от первого mesh; разные текстуры обычно
нужно рисовать раздельно. Импорт glTF поддерживает triangle list/strip/fan, внешние
buffers/textures через Assets, percent-encoded URI и mirrored node transforms. Point/
line primitives не превращаются в треугольники. Скелетная анимация не реализована.
Внешние source dependencies отслеживаются GPU-кэшем для hot reload.

`Ray`, `Aabb`, `Frustum` работают без GPU. Frustum использует глубину 0..1, как wgpu;
не бесконечную дальнюю плоскость. AABB visibility — консервативная проверка, которая
может сохранить лишний объект возле угла frustum. `View` в kvas_render добавляет
pixel picking, world-to-screen и проверяемый perspective constructor. Пиксельные
координаты отсчитываются от левого верхнего угла viewport.

`Renderer::new_headless` и `capture` создают RGBA8 кадр без окна. Readback блокирующий,
с timeout, подходит для экспорта/проверок. Возвращаемое изображение сохраняется обычным
`image.save(path)`. Capture использует точный Frame.size независимо от render_scale,
максимум 4096 на сторону и ограничения device. Обычный render по-прежнему возвращает
egui texture. `Renderer::stats` считает mesh/shadow calls, triangles, hidden/culled
objects и debug lines; это не измерение GPU-времени.

```text
cargo run -p kvas_render --example capture_scene -- output.png
cargo run -p kvas_render --example capture_scene -- demo.png games/demo
cargo test -p kvas_render --test capture_gpu -- --ignored
```

В редакторе доступно `File → Export View PNG...`: экспортируется текущая камера
без grid/gizmo/HUD, с выбранным режимом Lit/Normals и т. п. Счётчики draw calls и
triangles видны по наведению на FPS/object counter. При потере фокуса окна ввод и
cursor lock сбрасываются; одиночный шаг Play использует фиксированный интервал.

GPU-тесты явно opt-in: для них нужен доступный native wgpu adapter. Обычные тесты
проверяют CPU-логику, сериализацию, lifecycle и WGSL, а отдельные demo/FPS integration
тесты собирают и загружают настоящие DLL. Успешная сборка сама по себе не доказывает
работоспособность графического окна на другой ОС или другом драйвере.

## Звук, время, небо и эффекты в скриптах

Все вызовы идут через `Ctx`; звук ставится в очередь, а воспроизводит его оболочка
(редактор или player) через крейт `kvas_audio`.

- Звук: `play_sound(клип, громкость)`, `play_music` (цикл), `play_sound_at(клип, громкость, позиция)`
  (затухание по расстоянию и стерео-панорама), `play_sound_panned`, `play_sound_pitched`,
  `stop_sound(id)`, `set_sound_volume`, `stop_all_sounds`, `set_master_volume`. Компонент
  `AudioSource` играет клип прямо с объекта. Поддерживаются wav, ogg, mp3, flac, m4a.
- Время: `dt()`, `elapsed()`, `frame()`, `fps()`, `every(секунды)`.
- Небо: `time_of_day()`, `set_time_of_day(час)`, `set_day_speed`, `set_clouds`, `is_night()`,
  `sky()` для всех параметров.
- Вид: `post()` (bloom, lens_flare, blur, motion_blur, vignette, chromatic_aberration,
  film_grain, sharpen, tint), `post_preset("cinematic")`, `shadows()` (softness, strength,
  bias, range), `set_exposure`.
- Вода: компонент `VoxelWater` со стилями `Voxel`, `Smooth`, `Realistic`.
