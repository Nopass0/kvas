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
| `lock_cursor(true)` | захват курсора для шутера |

### Звук, небо, эффекты

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
