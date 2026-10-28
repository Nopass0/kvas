# Рецепты скриптов

Каждый рецепт - готовый скрипт. Код на этой странице **подтягивается прямо из тестов**
(`crates/kvas_engine/tests/cookbook.rs`): он компилируется и запускается в CI, поэтому не устаревает.

Чтобы использовать рецепт: скопируйте структуру в файл `<проект>/scripts/src/`, добавьте тип в
`register_scripts!(...)` и перетащите скрипт на объект. Числовые поля с `0` по умолчанию задаются в Инспекторе
(у `Chaser` это `speed` и `stop_distance`).

## Вращение с настройкой в Инспекторе

Поле, объявленное в `fields()`, видно в Инспекторе и сохраняется в сцене.

<<< ../../crates/kvas_engine/tests/cookbook.rs#spinner{rust}

## Преследователь

Идёт к объекту `Player`, поворачивается к нему и останавливается на дистанции.
`look_at_flat` меняет только курс, `move_towards` возвращает `true`, когда точка достигнута.

<<< ../../crates/kvas_engine/tests/cookbook.rs#chaser{rust}

## Патруль по точкам

Хранит маршрут и индекс следующей точки. Состояние (`next`) сбрасывается при горячей перезагрузке - это нормально.

<<< ../../crates/kvas_engine/tests/cookbook.rs#patrol{rust}

## Спавнер шаров

`Timer::repeating` отсчитывает интервал, `Rng` даёт повторяемую случайность, `despawn_after` убирает мусор.

<<< ../../crates/kvas_engine/tests/cookbook.rs#spawner{rust}

## Камера от третьего лица

Экспоненциальное сглаживание `1 - exp(-k * dt)` даёт одинаковое ощущение на 30 и 144 FPS.

<<< ../../crates/kvas_engine/tests/cookbook.rs#camera_follow{rust}

## Монета

Проверяет, кто находится рядом, по тегу `player` (тег добавляется в Инспекторе или `ctx.add_tag`).

<<< ../../crates/kvas_engine/tests/cookbook.rs#pickup{rust}

## Выстрел лучом

`raycast` возвращает `RayHit`. Силу можно толкнуть через компонент `RigidBody` попавшего объекта.

<<< ../../crates/kvas_engine/tests/cookbook.rs#shooter{rust}

## Сообщения: кнопка и дверь

Скрипты не знают друг друга: кнопка рассылает сигнал, дверь его слушает.
Сигналы доставляются в следующем кадре.

<<< ../../crates/kvas_engine/tests/cookbook.rs#signals{rust}

## Как проверить свой скрипт без редактора

Скрипты - обычные типы, поэтому их можно запускать в тесте, как делает этот cookbook:

```rust
let host = ScriptHost::from_defs(vec![ScriptDef::of::<Chaser>("Chaser")]).unwrap();
let mut runtime = Runtime::new(scene, Some(host));
for _ in 0..240 {
    runtime.tick(1.0 / 60.0);
}
assert!(runtime.scene.world.get::<Transform>(chaser).unwrap().position.x > 5.0);
```
