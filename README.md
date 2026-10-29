<div align="center">

<img src="docs/public/logo.svg" width="84" alt="Kvas">

# Kvas Engine

**An open-source 3D game engine written in Rust, with a Unity-style editor.**

ECS · wgpu renderer · Rapier physics · visual materials · hot-reloaded Rust scripts

[Documentation](https://nopass0.github.io/kvas/en/) · [Quick start](#quick-start) · [Roadmap](ROADMAP.md) · [Contributing](CONTRIBUTING.md) · [Русская версия](https://nopass0.github.io/kvas/)

<img src="docs/images/demo.png" alt="Kvas demo scene" width="860">

</div>

Kvas is an actively developed engine and editor. The repository includes the Rust workspace, sample projects, a bilingual documentation site, and tools for building standalone games. Some roadmap items are experimental or not implemented; this README and [ROADMAP.md](ROADMAP.md) distinguish current functionality from plans.

## Features

- **Editor:** scene hierarchy, inspector, transform gizmos, project browser, material/node editor, texture painting, animation and audio panels, undo/redo, English and Russian UI.
- **Rendering:** wgpu, PBR materials, shadows, HDRI and procedural skies, particles, post-processing, camera effects, and a custom spatial upscaler inspired by FSR. This is not AMD FSR SDK integration.
- **Materials and shaders:** node-based `.kmat` materials, WGSL object shaders and camera effects, reusable shader libraries.
- **Gameplay:** sparse-set ECS, Rapier physics, scene serialization, asset management and Rust scripts with hot reload.
- **Player builds:** package a project into a standalone player and asset archive. See [building and distribution](docs/en/guide/getting-started.md).

Check [known limitations and roadmap](ROADMAP.md) before relying on platform support or vendor-specific features. DLSS and official FSR SDK integration, hardware ray tracing, skeletal animation, and broad cross-platform support are not currently claimed as complete.

## Quick start

### Requirements

- Rust toolchain via [rustup](https://rustup.rs) (stable; see `rust-toolchain.toml` if present).
- A graphics adapter supported by wgpu. Windows with Vulkan or DirectX 12 is the primary development target; other platforms need validation.
- Git and (only for the documentation site) Node.js 22 or later with npm.

### Clone, build, and run

```bash
git clone https://github.com/Nopass0/kvas.git
cd kvas
cargo run -p kvas_editor -- games/demo
```

Run the project manager without a project argument:

```bash
cargo run -p kvas_editor
```

Open the demo and immediately enter Play mode:

```bash
cargo run -p kvas_editor -- games/demo --play
```

Build the player once so the editor can use it for packaged projects:

```bash
cargo build -p kvas_player
```

Build a game without opening the editor UI:

```bash
cargo run -p kvas_editor -- --build games/demo
```

The output is written to the project's `build/` directory. Use `--release` for optimized editor/player binaries, for example `cargo run --release -p kvas_editor -- games/demo`.

## Validate changes

Before committing, install the repository hook once and run the same checks locally:

```powershell
cargo install cargo-llvm-cov --locked
git config core.hooksPath .githooks
./tools/pre-commit.ps1
```

The hook checks formatting, compilation, the complete workspace test suite, and Rust doctests. Pull requests run the same required validation in GitHub Actions. See [CONTRIBUTING.md](CONTRIBUTING.md) for documentation, performance, and review requirements. GPU-only tests need a suitable adapter and are not silently treated as covered by CPU CI.

## Documentation

- [English documentation site](https://nopass0.github.io/kvas/en/)
- [Russian documentation site](https://nopass0.github.io/kvas/)
- [Project and crate map](docs/PROJECT_MAP.md)
- [Development and verification guide](DEVELOPMENT.md)
- [Contribution guide](CONTRIBUTING.md)
- [Agent instructions](AGENTS.md)
- [Feature roadmap and current limitations](ROADMAP.md)
- [Rust API documentation](https://nopass0.github.io/kvas/api/kvas_engine/index.html)

Run the docs site locally:

```bash
cd docs
npm ci
npm run dev
```

GitHub Pages is built and deployed by `.github/workflows/docs.yml`. Enable **Settings → Pages → Build and deployment → GitHub Actions** in a fork to publish your own copy.

## Workspace layout

| Path | Purpose |
|---|---|
| `crates/kvas_ecs` | ECS primitives and world queries |
| `crates/kvas_engine` | Runtime, scenes, assets, physics, scripting, materials and engine API |
| `crates/kvas_render` | wgpu renderer, WGSL support and rendering utilities |
| `crates/kvas_audio` | Audio playback and spatial audio |
| `crates/kvas_editor` | Editor application and tools |
| `crates/kvas_player` | Standalone game player |
| `games/` | Demo, showcase, regression and sample game projects |
| `docs/` | VitePress documentation in English and Russian |
| `tools/` | Project generation, verification and content tools |
| `third_party/` | Vendored dependencies; check each directory's license and local policy |

## Community

Use GitHub Issues for reproducible bugs and scoped feature proposals. Use Discussions for questions and design exploration when enabled. Please include engine revision, operating system, GPU/backend, exact steps, expected and observed behavior, and relevant logs. Do not attach secrets, private user data, or assets without redistribution rights.

By participating, you agree to follow the [Code of Conduct](CODE_OF_CONDUCT.md). Security-sensitive reports should follow [SECURITY.md](SECURITY.md).

## License

See [LICENSE](LICENSE) for the project's license and any additional third-party notices that apply to specific assets or dependencies.
