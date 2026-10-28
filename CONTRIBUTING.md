# Как участвовать

Спасибо, что хотите улучшить Kvas.

## Сборка и проверка

```bash
cargo run -p kvas_editor -- games/demo        # редактор с демо-проектом
cargo test -p kvas_ecs -p kvas_engine         # быстрые тесты
cargo test -p kvas_render --lib               # проверка всех шейдеров нод через naga
./tools/verify.ps1                            # полная проверка (Windows)
```

## Правила

- **DRY / KISS.** Один механизм на одну задачу: рефлексия `Field`, пути через `Assets`, один `Runtime`, один рендерер.
- Публичное API снабжается rustdoc: единицы измерения, когда вызывается, что при ошибке. Примеры в документации - исполняемые.
- Новая нода: добавьте её в `material_nodes_ext.rs` (и подсказку в `HELP`), функции WGSL - в `shader_lib.rs`,
  затем `cargo run -p kvas_engine --example gen_node_docs`. Тест валидирует шейдер во всех режимах.
- Новый встроенный материал: `material_presets.rs`, затем `cargo run -p kvas_engine --example gen_material_library`.
- Рецепты скриптов живут в `crates/kvas_engine/tests/cookbook.rs`, сайт подтягивает код оттуда.
- Строки интерфейса - английские ключи через `t("...")`, перевод в `lang_ru*.rs`.

## Документация

```bash
cd docs && npm install && npm run dev
```

Сайт публикуется автоматически при пуше в `main`.
