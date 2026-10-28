# Скрипты на Rust

Скрипт - обычная структура Rust. Она компилируется в динамическую библиотеку рядом с проектом
и **перезагружается при сохранении файла**, не закрывая редактор.

## Первый скрипт

Кнопка **+ New script** в панели Project создаёт файл из шаблона. Вручную это выглядит так
(`<проект>/scripts/src/spinner.rs`):

<<< ../../crates/kvas_engine/tests/cookbook.rs#spinner{rust}

Скрипт регистрируется один раз в `lib.rs`:

```rust
use kvas_engine::prelude::*;

mod spinner;
use spinner::Spinner;

register_scripts!(Spinner);
```

Затем перетащите скрипт из панели Project на объект в сцене или в иерархии.
Поля из `fields()` появятся в Инспекторе и сохранятся в сцене.

::: tip Одна строка прелюдии
`use kvas_engine::prelude::*;` приносит всё нужное: `Script`, `Ctx`, `Vec3`, `Transform`, `Key`,
`Timer`, `Rng`, математику (`lerp`, `damp`, `approach`...) и все компоненты.
:::

## Жизненный цикл

| Метод | Когда вызывается | Для чего |
|---|---|---|
| `start(&mut self, ctx)` | один раз перед первым кадром экземпляра | настройка, поиск объектов, значения по умолчанию |
| `update(&mut self, ctx)` | каждый кадр | движение, ввод, логика |
| `fixed_update(&mut self, ctx)` | каждый шаг физики (60 Гц) | силы и импульсы |
| `fields(&mut self)` | когда нужно показать или сохранить поля | Инспектор и сохранение сцены |

Горячая перезагрузка создаёт новый экземпляр и вызывает `start` снова: **состояние скрипта сбрасывается**,
а значения полей из `fields()` сохраняются.

## Что даёт `Ctx`

`Ctx` - единственная дверь в движок. Самое нужное:

### Время

| Метод | Описание |
|---|---|
| `dt()` | секунды с прошлого вызова (в `fixed_update` - шаг физики) |
| `elapsed()`, `frame()`, `fps()` | время игры, номер кадра, FPS |
| `every(seconds)` | `true` раз в N секунд |
| `time_of_day()`, `set_time_of_day(h)`, `is_night()` | время суток |

### Позиция и движение

| Метод | Описание |
|---|---|
| `position()` / `position_of(e)` | мировая позиция этого объекта / другого |
| `set_position(p)` | поставить в мировую точку |
| `translate(d)` / `translate_local(d)` | сдвинуть по осям родителя / по своим осям |
| `rotate(deg)` | добавить углы Эйлера (градусы) |
| `forward()`, `right()`, `up()` | направления объекта |
| `look_at(p)` / `look_at_flat(p)` | повернуться к точке (с наклоном / только по горизонтали) |
| `move_towards(p, speed)` | идти к точке, `true` когда пришли |
| `distance_to(e)`, `direction_to(p)` | расстояние и направление |
| `move_to(e, p, seconds, ease)` | плавный твин любого объекта |
| `teleport(p, reset_velocity)` | телепорт с учётом физики |

### Поиск, создание, удаление

| Метод | Описание |
|---|---|
| `find(name)`, `find_all(name)`, `find_with::<T>()` | поиск по имени или по компоненту |
| `tagged(tag)`, `has_tag`, `add_tag`, `nearest_tagged(tag)` | теги и ближайший с тегом |
| `children(e)` | потомки объекта |
| `spawn(name)`, `spawn_primitive(name, "sphere", pos)` | создать объект |
| `despawn(e)`, `despawn_self()`, `despawn_after(e, seconds)` | удалить |
| `set_color(e, rgba)`, `set_visible(e, bool)`, `set_texture(e, path)` | внешний вид |
| `attach_script(e, "Name")`, `var(e, "Script", "field")` | общение со скриптами других объектов |

### Иерархия, теги и общие переменные

| Метод | Описание |
|---|---|
| `parent()`, `parent_of(e)`, `root()` | родитель, родитель другого объекта, корень иерархии |
| `children(e)`, `descendants(e)`, `find_child(name)` | дети, все потомки, потомок по имени |
| `tagged(tag)`, `tagged_within(tag, center, radius)`, `tags_of(e)` | объекты по тегу, по тегу в радиусе (ближайшие первыми), теги объекта |
| `set_global(key, json)`, `global(key)`, `global_f32(key, default)`, `global_bool(..)`, `add_global(key, delta)` | переменные, общие для всех скриптов (счёт, сложность) |
| `var(e, "Script", "field")`, `set_var(e, "Script", "field", json)` | поле скрипта на другом объекте |

### Вершины и деформация

`ctx.make_editable(e)` делает встроенный примитив редактируемым, `ctx.editable(e)` отдаёт `EditableMesh`:

| Метод | Описание |
|---|---|
| `vertex_count()`, `position(i)`, `set_position(i, p)`, `bounds()` | доступ к вершинам |
| `select_sphere(c, r)`, `select_half_space(point, normal)`, `translate_vertices(ids, d)` | выбор и сдвиг вершин |
| `wave`, `twist`, `taper`, `bend`, `inflate`, `smooth` | готовые деформаторы |
| `noise_displace(&Noise, amp, time)` | смещение по нормалям шумом |
| `map_positions(\|i, p, n\| ...)` | любая своя деформация |
| `recalculate_normals()` | пересчитать нормали после деформации |

### Физика

| Метод | Описание |
|---|---|
| `add_impulse(v)`, `set_velocity(v)`, `velocity()` | управление телом этого объекта |
| `raycast(origin, dir, max)` | луч, возвращает `RayHit { entity, point, normal, distance }` |
| `overlap_sphere(center, r)`, `touching(e)` | кто рядом / кто касается |

### Ввод и интерфейс

| Метод | Описание |
|---|---|
| `key_down(Key::W)`, `key_pressed(..)`, `mouse_pressed(MouseButton::Left)` | сырой ввод |
| `action_down("Jump")`, `action_axis("Left", "Right")`, `action_vector(..)` | именованные действия (настройки: File → Project Settings → Input) |
| `hud_message(text, seconds)`, `hud_prompt(text)`, `hud_objective(text)` | подсказки на экране |
| `lock_cursor(true)` | захват курсора для шутера (при запуске игры курсор захватывается сам; Esc освобождает, клик по окну игры — снова захватывает) |

### Сцены

| Метод | Что делает |
|---|---|
| `load_scene("scenes/level2.scene.json")` | после кадра переключает игру на другую сцену: объекты и скрипты текущей сцены удаляются, у новой скрипты стартуют заново |
| `add_scene("scenes/ui.scene.json")` | добавляет все объекты другой сцены к текущей (куски уровня, общий интерфейс) |
| `reload_scene()` | перезапускает текущую сцену из файла (рестарт уровня) |
| `scene_path()` | путь текущей сцены |

```rust
script! {
    /// R — начать уровень заново; упал вниз — следующий уровень.
    pub struct LevelFlow {
        next: String = "scenes/level2.scene.json".into(),
    }

    update(self, ctx) {
        if ctx.key_pressed(Key::R) {
            ctx.reload_scene();
        }
        if ctx.position().y < -20.0 {
            ctx.load_scene(&self.next);
        }
    }
}
```

Время, привязки клавиш и захват курсора переходят в новую сцену; физика и экземпляры скриптов создаются заново.

### Звук, небо, эффекты

Компонент `AudioSource` умеет **направленный звук**: поле `cone_angle` (градусы, 360 - во все стороны) задаёт луч вдоль оси -Z объекта,
громкость вне конуса падает до 10 %. Стереопозиция считается по положению слушателя (первая камера). В редакторе при выделении
источника рисуются радиусы слышимости и конус.

`play_sound("sfx/hit.wav", 1.0)`, `play_music(..)`, `play_sound_at(clip, volume, pos)`,
`sky()`, `set_clouds(0.5)`, `post()`, `post_preset("cinematic")`, `set_exposure(1.2)`.

### Сообщения между скриптами

`ctx.emit("open_door", json!({ "who": "player" }))` рассылает сообщение всем,
`ctx.send_to(entity, "damage", json!(10))` - одному адресату,
`ctx.signals("open_door")` читает пришедшее. Сообщения доставляются в следующем кадре.

::: details Нужно что-то ещё?
`ctx.scene.world` - прямой доступ к ECS: `ctx.get::<T>()`, `ctx.get_mut_of::<T>(entity)`,
`ctx.insert(entity, component)`. Полный список методов - в [Rust API](/ENGINE_API) и в сгенерированной
[документации rustdoc](https://nopass0.github.io/kvas/api/kvas_engine/script/struct.Ctx.html).
:::

## Математика и случайные числа

Эти помощники уже в прелюдии:

```rust
use kvas_engine::prelude::*;

let t = lerp(0.0, 10.0, 0.25);                  // 2.5
let k = remap(5.0, 0.0, 10.0, 100.0, 200.0);    // 150.0
let speed = approach(speed, 8.0, 20.0 * ctx.dt()); // разгон без перелёта
let x = damp(x, target_x, 8.0, ctx.dt());          // сглаживание, не зависящее от FPS
let yaw = approach_degrees(yaw, wanted_yaw, 360.0 * ctx.dt()); // поворот по короткой дуге

let mut rng = Rng::new(42);                      // повторяемый поток
let p = rng.in_sphere(2.0);                      // точка внутри шара
let color = [rng.f32(), rng.f32(), rng.f32(), 1.0];
let pick = rng.pick(&["красный", "синий"]);      // Option<&T>
let lucky = ctx.chance(0.1);                     // без своего Rng: 10 %
```

## Таймеры и твины

```rust
let mut cooldown = Timer::once(1.5).unwrap();
// каждый кадр:
cooldown.tick_secs(ctx.dt());
if cooldown.just_finished() { /* выстрел готов */ }
```

`ctx.move_to(entity, target, seconds, Ease::BackOut)` двигает объект плавно (14 кривых `Ease`: `Linear`, `SmoothStep`, `QuadOut`, `BackOut`, `BounceOut`...).

## Правила, которые защищают горячую перезагрузку

- Библиотека и редактор должны быть собраны **одним** компилятором и **одними** исходниками движка. При несовпадении вместо падения вы увидите сообщение «пересоберите».
- Статические переменные дублируются между библиотекой и редактором: общайтесь с движком только через `Ctx` (лог - `ctx.log("...")`).
- `panic!` в скрипте не роняет редактор: экземпляр отключается, ошибка видна в Консоли.
- Новые типы компонентов в скриптах определять нельзя - используйте встроенные и поля скрипта.

## Дальше

- [Рецепты](/scripting/cookbook) - проверенные скрипты, которые можно копировать.
- [Rust API: руководство](/ENGINE_API) - ECS, физика, иерархия, время, сообщения.
