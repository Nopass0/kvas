# Project Map

This page helps contributors find the maintained source for each part of Kvas. Confirm current symbols and ownership in the checkout; this map is a navigation aid, not a substitute for code inspection.

## Rust workspace

| Crate | Responsibility | Start here |
|---|---|---|
| `kvas_ecs` | Entity/component storage and world queries | `crates/kvas_ecs/src/` |
| `kvas_engine` | Runtime, scenes, components, assets, physics, scripting, materials and shared APIs | `crates/kvas_engine/src/` |
| `kvas_render` | wgpu rendering, render resources, WGSL and shader validation | `crates/kvas_render/src/` |
| `kvas_audio` | Playback, spatial audio and audio-facing APIs | `crates/kvas_audio/src/` |
| `kvas_editor` | Editor shell, panels, project workflows, localization and code tools | `crates/kvas_editor/src/` |
| `kvas_player` | Standalone player entry point and packaged-project runtime | `crates/kvas_player/src/` |

Inspect each crate's `Cargo.toml` for its dependency, feature, example, and test boundaries. Integration tests usually live in `tests/`, unit tests alongside source, and examples in `examples/`.

## Projects, docs, and tooling

- `games/demo/`: primary getting-started project.
- `games/showcase/`: larger showcase and rendering example.
- `games/library_check/`: validation project for built-in script examples.
- Other `games/*/` projects: regression fixtures and sample content. Check asset provenance before copying content between projects.
- `docs/`: VitePress docs, English under `docs/en/`, Russian at the root/localized sections, and shared images/assets.
- `tools/`: project verification and content/library generators.
- `.github/workflows/`: pull request CI and documentation publication.
- `.githooks/`: repository-local hooks used when `core.hooksPath` is configured.
- `third_party/`: vendored dependencies; each component has its own license and update policy.

## Core design constraints

- Component/script `Field` reflection feeds the inspector, scene serialization, and undo behavior; extend the common mechanism instead of creating parallel field maps.
- Route project and packaged-game asset reads through the shared asset abstraction.
- Keep editor Play mode and the standalone player on the same runtime behavior.
- Keep the same renderer responsible for editor scene previews and game rendering.
- Play mode operates on a scene copy so Stop can discard runtime changes.

See [AGENTS.md](https://github.com/Nopass0/kvas/blob/main/AGENTS.md) for change constraints and [DEVELOPMENT.md](https://github.com/Nopass0/kvas/blob/main/DEVELOPMENT.md) for build/test commands.
