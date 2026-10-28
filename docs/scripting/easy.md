# Писать скрипты легко

Эта страница - про то, как сделать скрипт с минимумом кода, красиво выставить переменные в Инспектор и легко отлаживать.

## Один блок - один скрипт: `script!`

Макрос `script!` объявляет **переменные с значениями по умолчанию** и **колбэки** в одном месте.
Он сам создаёт `Default`, `fields()` для Инспектора и реализацию `Script`.

```rust
use kvas_engine::prelude::*;

script! {
    /// Вращает объект и считает кадры.
    pub struct Spinner {
        speed: f32 = 90.0,            // в Инспекторе: число
        axis: Vec3 = Vec3::Y,         // в Инспекторе: три поля XYZ
        enabled: bool = true,         // флажок
        label: String = "spin".into(),// текст
        frames: Local<u32> = Local(0),// Local - переменная скрипта, не показывается и не сохраняется
    }

    start(self, ctx) {
        ctx.log(format!("{} стартовал", self.label));
    }

    update(self, ctx) {
        if self.enabled {
            ctx.rotate(self.axis * self.speed * ctx.dt());
        }
        *self.frames += 1;
    }
}

register_scripts!(Spinner);
```

Колбэки: `start`, `update`, `fixed_update` (любые, в любом порядке). Первое слово в скобках - имя `self`, второе - имя контекста.

### Какие типы можно выставлять в Инспектор

| Тип | Виджет |
|---|---|
| `f32`, `i32`, `u32` | числовое поле |
| `bool` | флажок |
| `Vec3` | три поля X Y Z |
| `String` | текстовое поле |
| `Color` | выбор цвета |
| `EntityRef` | выбор объекта по имени |
| `MeshRef`, `TextureRef`, `SoundRef` | выбор файла нужного типа |
| любое `enum` с `Choice` | выпадающий список |
| `Local<T>` | скрыто |

## Если структура уже написана: `fields!`

```rust
#[derive(Default)]
struct Door { speed: f32, open: bool, name: String }

impl Script for Door {
    fn update(&mut self, ctx: &mut Ctx) { /* ... */ }
    fn fields(&mut self) -> Vec<Field<'_>> { fields!(self; speed, open, name) }
}
```

Один список вместо `Field::f32("speed", &mut self.speed)` на каждое поле. Имя поля в Инспекторе и в сцене совпадает с именем переменной.

## Отладка

Всё попадает в Консоль редактора: предупреждения жёлтым, ошибки красным.

| Метод | Что делает |
|---|---|
| `ctx.log("текст")` | обычная строка |
| `ctx.warn("текст")` | жёлтое предупреждение |
| `ctx.error("текст")` | красная ошибка, скрипт продолжает работать |
| `ctx.watch("speed", self.speed)` | печатает `speed = 4.0` (через `Debug`) |
| `ctx.watch_every(1.0, "pos", ctx.position())` | то же, но не чаще раза в секунду - безопасно в `update` |

```rust
update(self, ctx) {
    ctx.watch_every(0.5, "позиция", ctx.position());
    if ctx.distance_to(self.target).is_none() {
        ctx.warn("цель пропала");
    }
}
```

::: tip Паника не страшна
`panic!` в скрипте не роняет редактор: экземпляр отключается, а в Консоли появляется сообщение с именем скрипта и объекта.
Исправьте код, сохраните файл - скрипт перезагрузится.
:::

## Тесты вместо запусков

Скрипты запускаются без окна, поэтому логику удобно проверять тестами (см. [рецепты](./cookbook#как-проверить-свой-скрипт-без-редактора)).

## Сглаживание и плавность

Все функции есть в `prelude`:

| Функция | Для чего |
|---|---|
| `lerp(a, b, t)`, `inverse_lerp`, `remap`, `remap_clamped` | интерполяция и перевод диапазонов |
| `smoothstep(e0, e1, x)`, `ease_smooth(t)`, `ease_smoother(t)` | S-образные кривые |
| `approach(cur, target, max_step)` | движение к цели без перелёта |
| `damp(cur, target, lambda, dt)` | экспоненциальное сглаживание, не зависит от FPS |
| `smooth_damp(cur, target, velocity, time, dt)` | пружина без перелёта: возвращает `(значение, скорость)` |
| `approach_degrees`, `delta_degrees`, `wrap_degrees` | углы по короткой дуге |

```rust
// Камера плавно догоняет цель, скорость хранится между кадрами.
update(self, ctx) {
    let (x, v) = smooth_damp(ctx.position().x, self.target_x, *self.vx, 0.3, ctx.dt());
    *self.vx = v;
    ctx.translate(Vec3::new(x - ctx.position().x, 0.0, 0.0));
}
```
