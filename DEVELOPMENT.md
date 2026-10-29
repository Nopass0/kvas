# Development Guide

This guide describes how to obtain, run, test, document, and build Kvas from source. It complements the user guide at `docs/en/guide/` and repository-wide contributor rules in `AGENTS.md`.

## 1. Get the source

```bash
git clone https://github.com/Nopass0/kvas.git
cd kvas
```

To work on a fork, clone your fork and add the upstream repository as `upstream`. Keep feature work on a topic branch and open a pull request against `main`.

## 2. Install prerequisites

- Install the stable Rust toolchain with [rustup](https://rustup.rs). `rustfmt` and `rustdoc` are used by validation.
- Install Git.
- Install a graphics driver with a wgpu-supported Vulkan or DirectX 12 adapter for running the editor. Headless compilation and most unit tests do not prove that the local GPU path works.
- To build the docs site, install Node.js 22+ and npm.
- Linux contributors may need ALSA and udev development packages plus `pkg-config`; GitHub CI installs `libasound2-dev`, `libudev-dev`, and `pkg-config`.

Check toolchain availability:

```bash
rustc --version
cargo --version
```

## 3. Build and run

```bash
cargo run -p kvas_editor
cargo run -p kvas_editor -- games/demo
cargo run -p kvas_editor -- games/demo --play
cargo build -p kvas_player
```

Build a sample game from the command line:

```bash
cargo run -p kvas_editor -- --build games/demo
```

The packaged output is placed under the project's `build/` directory. Use `--release` for optimized binaries. For editor controls and project authoring, see [Getting Started](docs/en/guide/getting-started.md) and [Editor](docs/en/editor.md).

## 4. Configure the commit gate

Configure the repository-local hook path once per checkout:

```bash
git config core.hooksPath .githooks
```

Then run the pre-commit gate before committing:

```powershell
./tools/pre-commit.ps1
```

The script runs formatting verification, workspace compilation, all workspace tests with LLVM line coverage, and doctests. It writes the local LCOV report to `target/llvm-cov-target/lcov.info`. Install the coverage tool once with `cargo install cargo-llvm-cov --locked`; the script stops with this instruction if it is missing. The Git hook invokes the same script. GitHub Actions repeats it on pull requests and uploads LCOV as an artifact. GPU tests are opt-in because they require a suitable adapter; run them with `./tools/verify.ps1 -Gpu` on supported hardware and include the adapter details in the PR.

If a check fails, fix the cause and rerun the complete gate. Do not use `--no-verify` to bypass it. On non-Windows systems, run the Cargo commands shown by the script directly, or use PowerShell 7 to run the script.

## 5. Tests and coverage

Useful focused checks during iteration:

```bash
cargo test -p kvas_ecs -p kvas_engine
cargo test -p kvas_render --lib
cargo test -p kvas_engine --test cookbook
```

The full pre-commit gate is authoritative for changes that can affect multiple crates. Add a regression test for each behavior fix. Tests should cover error paths and boundaries, not just the happy path. Use GPU-backed checks for GPU-specific behavior and state when hardware validation was unavailable.

Coverage summaries are produced locally and LCOV reports are preserved by GitHub CI as workflow artifacts. Coverage is intended to show what changed tests exercise; the project has not yet established a repository-wide numeric threshold. Do not treat the existence of an artifact as proof that every feature is adequately tested.

## 6. Performance work

Before optimizing, choose and document a representative workload. Capture a reproducible baseline and candidate measurement using the same machine, adapter/backend, Rust profile, scene/assets, and run procedure. Add a benchmark harness if none exists. Include correctness tests, repeated measurements, variance, and before/after results in the PR. Unit tests must not depend on tight timing thresholds. See the performance testing policy in `AGENTS.md` and the benchmark foundation items in `ROADMAP.md`.

## 7. Documentation site and Rust API

The VitePress sources are in `docs/`; English pages live under `docs/en/` and Russian pages are at the documentation root and its localized sections. Run the site locally:

```bash
cd docs
npm ci
npm run dev
npm run build
```

Build the principal Rust API documentation:

```bash
cargo doc --no-deps -p kvas_ecs -p kvas_engine -p kvas_render
```

The node reference, material catalogue, and other generated references have canonical source code and generators. Follow the instructions in `CONTRIBUTING.md`; do not hand-edit generated files. Every feature change should update relevant Rust docs and both locales where a corresponding guide exists.

## 8. Generators and content validation

Consult `CONTRIBUTING.md` for node, material, shader, script-library, and cookbook workflows. After changing generated content, run its generator and owning tests, inspect the generated diff, and include the generated files only when they are tracked source/reference outputs. Do not add `target/`, local `.kvas` state, build packages, local import fixtures, or third-party assets without redistribution clearance.

## 9. Pull request handoff

Before opening a PR:

1. Run the complete pre-commit gate.
2. Run relevant docs build and generators.
3. For performance changes, capture baseline and candidate measurements and explain the environment.
4. Review `git diff --check`, the full diff, and staged file list.
5. Describe user-visible behavior, tests, coverage evidence, docs, performance results, known limitations, and migration steps.

CI and Pages run asynchronously after push. Check their status separately; a local pass or successful push does not mean CI or publication completed.
