# Kvas Engine Roadmap

This is a community planning document, not a delivery promise. It separates implemented capabilities from partial work and proposals. Priorities indicate order of attention: **P0** blocks reliable everyday development or first-game workflows; **P1** is a core production capability; **P2** expands quality and creative workflows; **P3** is exploratory. Priorities can change after design discussion and contributor capacity.

Status: `[x]` implemented and exercised; `[~]` partial, experimental, or awaiting a named validation step; `[ ]` not complete. A checkbox only changes to `[x]` when implementation, tests, docs, and platform limitations are reviewed together.

## Current capabilities and limits

The editor has scene hierarchy and inspector, transform tools, project/asset browsing, node-based materials, texture painting, panels for animation and sound, undo/redo, and English/Russian UI. The engine has a sparse-set ECS, scene serialization, shared runtime and renderer, wgpu rendering, Rapier physics, Rust scripts with hot reload, asset packaging, particles, post-processing, and a custom spatial upscaler inspired by FSR.

The following are not equivalent to full vendor/product integrations: the upscaler is not AMD's FSR SDK; DLSS selection currently falls back to the custom spatial upscaler; the frame-generation UI does not mean a production frame-generation backend is active. Hardware ray tracing/path tracing, complete skeletal animation, and broad non-Windows validation are not complete. Check [the NVIDIA notes](docs/NVIDIA.md) and the relevant code before relying on a capability.

## 0. Reliability, performance and release engineering

### 0.1 Test and benchmark foundation (P0)

- [x] Workspace formatting, check, unit/integration tests, and doctests are available through the local verification workflow.
- [ ] Establish reproducible Criterion or equivalent benchmark suites for ECS queries, scene load/save, asset decode/pack, render submission, material compilation, and representative editor workloads.
- [ ] Publish a baseline by OS, CPU, GPU/backend, build profile, and workload; record variance and define non-flaky regression policy.
- [ ] Add memory/allocation and load-time benchmarks, with realistic showcase scenes and assets that can legally be distributed.
- [ ] Add fuzz/property tests for scene, asset archive, import metadata, and project configuration parsers; define corpus and crash triage.
- [ ] Add GPU correctness/performance smoke on an owned runner or a clearly documented manual process; do not treat CPU CI as GPU validation.
- [ ] Add a CI coverage artifact and, after a stable baseline exists, propose a justified threshold by crate and changed code. Do not set an arbitrary global floor.
- [ ] Track test duration and flaky failures; split fast PR checks from scheduled hardware/performance suites without weakening required correctness checks.

### 0.2 Rendering throughput and memory (P0–P1)

- [ ] GPU instancing for repeated meshes, with stable object IDs, per-instance transforms/material data, and a compatibility plan for user WGSL.
- [ ] Static batching where it improves measured workloads, with correct bounds, visibility, and invalidation when source objects change.
- [ ] GPU frustum/occlusion culling and indirect draws, gated by backend support and validated against CPU visibility results.
- [ ] LOD generation and screen-space selection, including mesh simplification, transition handling, and foliage impostors.
- [ ] Sort transparent geometry correctly; add depth-aware soft particles and avoid unnecessary depth copies.
- [ ] Streaming texture mip levels and meshes with bounded memory, predictable cancellation, and visible progress.
- [ ] CPU/GPU profiler with render-pass timings, draw/instance counts, memory estimates, and exportable captures.
- [ ] Documented frame-time and memory budgets for representative scenes; regression report on benchmark changes.

### 0.3 Asset loading and packaging (P0–P1)

- [ ] Async model/texture decode and upload with cancellation, progress, useful errors, and responsive first frame.
- [ ] Cache imported/decoded intermediates by source content and importer settings; invalidate safely when either changes.
- [ ] Optional KTX2/Basis Universal or platform-native texture compression, mesh optimization, and compressed package archives; keep licensing and target support explicit.
- [ ] Release build pipeline that builds player and scripts with the chosen profile and reports missing assets/dependencies before packaging.
- [ ] Deterministic asset manifest, hashes, diagnostics, package validation, and reproducible-build metadata.
- [ ] Identify unused assets and support safe stripping without deleting source project content.

### 0.4 Upscaling and frame generation (P1–P2)

- [x] Custom spatial upscaler in the EASU/RCAS family of approaches; not certified or branded as official AMD FSR.
- [ ] Temporal anti-aliasing/upscaling foundation: motion vectors, jitter, history buffers, disocclusion handling, ghosting tests, and per-camera controls.
- [ ] Add an open, vendor-neutral temporal upscaler only after quality, licensing, device support, and GPU cost have been measured.
- [ ] Official AMD FSR integration as an optional backend after SDK version, license, API requirements, and supported wgpu/native backends are reviewed.
- [ ] NVIDIA DLSS Super Resolution integration through a supported native graphics backend/Streamline path, with SDK redistribution and NVIDIA hardware requirements documented. UI selection must report unavailable or fallback state accurately.
- [ ] Frame generation research and prototype: motion data/optical flow, latency controls, HUD handling, artifact evaluation, and explicit hardware support. Keep it opt-in and never present interpolated frames as simulation frames.
- [ ] Remove or relabel settings that imply unavailable backends until the end-to-end implementation and hardware test pass.

### 0.5 Lighting and image quality (P1–P2)

- [ ] Cascaded directional shadows, point/spot shadow options, contact shadows, and stable bias controls.
- [ ] SSAO/GTAO, screen-space reflections, volumetric fog/light shafts, cloud shadows, and image-quality/performance controls.
- [ ] Lightmap baking, probes, reflection captures, and a documented bake/update workflow.
- [ ] HDR display output, tone mapping and color transforms, LUT workflows, lens effects, and color-managed screenshots.
- [ ] Water depth/refraction, shoreline foam, caustics, and spline-based rivers with performance bounds.
- [ ] Hardware ray tracing and path tracing investigation: first compare hardware/API requirements across D3D12 and Vulkan backends, wgpu exposure, fallback paths, shader architecture, licensing, deterministic reference scenes, and performance cost. Then implement an optional ray-query/RT backend only where device support and validation are real. Provide raster fallbacks and never advertise ray tracing before adapter-backed tests and user documentation exist.

## 1. Editor usability and content tools

### 1.1 Workspace and navigation (P1)

- [x] Dockable editor panels, saved preferences/layout, themes, language switching, hierarchy, inspector, project browser, and transform gizmos.
- [ ] Command palette, global search, recent actions, contextual shortcuts, searchable settings, and shortcut customization.
- [ ] Multi-select, grouping, prefabs/instances with overrides, scene-to-scene copy, and safe reference repair.
- [ ] View modes for wireframe, overdraw, colliders, lighting, LOD, UVs, and texture mip diagnostics.
- [ ] Better window/layout presets, floating panels, multi-viewport and multi-monitor recovery, keyboard-only navigation, and accessibility audit.
- [ ] Clearer first-run project manager, templates, recoverable autosave, crash recovery, and actionable import/build errors.
- [ ] Consistent menu organization, discoverable tooltips, inline help, and localization coverage for all visible errors and status messages.

### 1.2 Asset browser and import (P1)

- [x] Project files, basic previews, context actions, script/shader/material creation, and drag-and-drop workflows.
- [ ] Rich model/animation/audio/texture previews and import settings for scale, axis, color space, compression, material mapping, and collision.
- [ ] Rename/move with scene reference updates, undo, trash/restore, duplicate detection, tags, filters, and dependency graph.
- [ ] Import glTF scenes, skins, skeletal clips, morph targets, cameras, lights, and animation events with validation reports.
- [ ] Asset metadata sidecars and stable importer versioning for reproducible reimport.

### 1.3 Layered image and texture editor (P1–P2)

- [x] Texture paint with masks and a small fixed layer set, brush tools, fill, and project asset output.
- [ ] Scalable non-destructive layer stack with groups, visibility, opacity, blend modes, masks, clipping, locks, reorder, merge, and undo history.
- [ ] Paint color, roughness, metallic, normal, height, and emission channels; preview channel packing and export texture sets.
- [ ] Brush library, pressure/tilt/rotation support, stabilizer, symmetry, selection tools, gradient/fill, clone/heal, and tablet calibration.
- [ ] Large-canvas tiling, mip preview, color management, autosave/recovery, and memory-budget controls.
- [ ] Bake layers and masks into optimized texture atlases with explicit source preservation and export settings.
- [ ] Consider PSD import/export only after format/library license and round-trip capability are evaluated; preserve interoperability with open formats.
- [ ] Keep 3D texture painting, UV-aware painting, and image editing in a coherent workspace while allowing a standalone 2D canvas workflow.

### 1.4 Audio workstation (P1–P2)

- [x] Audio source component, spatial playback, basic waveform panel, and script API.
- [ ] Timeline with non-destructive clip trim, split, fades, gain, normalization preview, markers, loop regions, and undo/redo.
- [ ] Mixer with buses, sends, meters, automation, mute/solo, voice limits, and reusable snapshots.
- [ ] Effects chain (EQ, compressor, reverb, low-pass/underwater), spatial zones, snapshots, and preview without destructive source edits.
- [ ] Stream long music from disk, improve codec/error reporting, and support loop points and metadata.
- [ ] Sound cues with random selection, pitch/volume variation, concurrency limits, and deterministic preview.
- [ ] Record/import workflow, waveform zoom, keyboard controls, accessibility, and a documented supported-format matrix.

### 1.5 Animation workspace (P1)

- [x] Initial animation panel and basic property/tween mechanisms; coverage and format limits must be verified per feature.
- [ ] Animation timeline with keyframes, channels, curves, snapping, markers, events, copy/paste, loop modes, and undo/redo.
- [ ] Curve editor with tangents, interpolation types, presets, and multi-channel selection.
- [ ] Skeletal animation and skinning from glTF, clip import/export, blend spaces, retargeting, and root motion.
- [ ] State-machine animator with transitions, parameters, preview, live debugging, and reproducible runtime/editor behavior.
- [ ] IK solvers, constraints, additive layers, animation events, and property animation for components/materials/cameras.
- [ ] Stylus pressure/tilt support for animation drawing workflows where relevant; validate input device APIs and accessibility.

## 2. Scripting and integrated code editor

### 2.1 Current foundation and authoring (P1)

- [x] Rust project scripts, hot reload, templates, tabs, syntax highlighting, search/replace, line navigation, basic API hints, and error isolation.
- [ ] Full language-server integration (rust-analyzer or a safe project-aware service) with diagnostics, code completion, hover docs, go-to definition, references, imports, rename, and formatting.
- [ ] Import/dependency management with explicit workspace context, cache/progress, offline behavior, license visibility, and diagnostics for incompatible crates.
- [ ] Symbol/outline panel, code folding, multi-cursor, snippets, refactor assists, bracket matching, diff/merge, and configurable keymaps.
- [ ] Debugger with breakpoints, call stack, locals, watches, conditional breakpoints, and safe interaction with hot reload.
- [ ] Compiler diagnostics embedded at exact lines, with structured error codes and links to relevant engine docs.
- [ ] Autocomplete for engine API, project scripts, components, assets, and material parameters from the same metadata used by runtime/editor.
- [ ] Securely constrain project builds and plugin/script execution; document trust boundaries and subprocess behavior.
- [ ] Evaluate optional Lua/WASM scripting and visual scripting only after runtime safety, performance, packaging, and maintenance tradeoffs are reviewed.

### 2.2 Engine APIs (P1–P2)

- [x] Core lifecycle, input helpers, hierarchy queries, scene save/load, events/timers, and physics API coverage in progress.
- [ ] Typed signals/events, material/shader parameter APIs, stable entity references, query filters, and clear borrow/lifecycle semantics.
- [ ] Versioned script API, deprecation policy, migration helpers, and compatibility tests for saved scenes and scripts.
- [ ] Expand doctests and cookbook examples across camera, audio, particles, materials, mesh editing, and UI.

## 3. Materials, animation, simulation and content

### 3.1 Node materials (P1)

- [x] Node graph materials, generated WGSL, mesh preview, reusable nodes, and material library.
- [ ] Graph UX: zoom/minimap, groups/frames, comments, multi-select, copy/paste, undo/redo, search, and accessible keyboard operations.
- [ ] Parameters exposed to inspector and scripts, material instances, more texture channels, and shader compile diagnostics tied to nodes.
- [ ] More complete Principled material model and import mappings for glTF; define semantics and fallbacks before adding nodes.
- [ ] Partial WGSL-to-graph inspection where it can be represented without losing source; retain a safe editable-code path for unsupported constructs.
- [ ] Validate Blender export against supported Blender versions and document unsupported node types.

### 3.2 Physics, particles and world (P1–P2)

- [x] Rapier integration, common rigid bodies/colliders/rays/triggers and CPU particles with multiple presets.
- [ ] Character controller for slopes, steps, moving platforms, and predictable fixed-step behavior; add tests for each.
- [ ] GPU particles, trails/ribbons, flipbooks, sub-emitters, collision and soft-particle depth, with measured memory limits.
- [ ] Local volumetric fog/light zones, improved voxel smoke, particle lighting, and wind/vortex fields.
- [ ] Terrain heightfield editor, sculpt/paint layers, foliage, spline roads/rivers, and streaming.
- [ ] Procedural city/house/tree/dungeon tools with seeds, deterministic regeneration, LOD, and license-clear example outputs.
- [ ] Cloth, soft-body, rope, vehicle and destruction prototypes only with stable solver/performance goals and representative regression scenes.

## 4. Build, distribution and supported platforms

- [x] Editor build command, player, project metadata, icon/basic window settings, and asset archives.
- [ ] Build profiles/platform matrix, release automation, dependency report, reproducible artifact metadata, and actionable failure logs.
- [ ] Installer/archive formats, application version resources, code signing guidance, splash screen, and in-game graphics settings.
- [ ] Static or bundled script strategy that does not require shipping an unexplained local DLL; document platform constraints.
- [ ] Linux and macOS validation, packaging, graphics backend coverage, input/audio support, and release artifacts.
- [ ] WebAssembly/WebGPU feasibility study and minimal sample only after unsupported engine features and resource constraints are documented.
- [ ] CI smoke test for a built game: launch where possible, load a deterministic scene, capture a frame, and compare robust metrics/images with a reviewed tolerance.
- [ ] User-facing crash diagnostics and opt-in privacy-preserving reports; never collect project content or identifiers without explicit consent.

## 5. Documentation, onboarding and community

- [x] Bilingual VitePress site, getting-started guides, node references, scripting cookbook, and Rust API publication pipeline.
- [ ] Audit every public subsystem against the project map and produce English and Russian guides for editor, assets/import, materials, animation, audio, physics, rendering, scripting, profiling, and packaging.
- [ ] Keep API reference generation, examples, version compatibility, screenshots, and navigation checked in CI.
- [ ] Tutorials built from clean clones and validated commands; provide a first playable project in under a documented onboarding target.
- [ ] Add architecture decision records for renderer/backends, script API compatibility, asset formats, and serialization evolution.
- [ ] Add benchmark methodology and reproducible sample-project documentation.
- [ ] Enable and moderate GitHub Discussions; publish contribution labels, triage process, release notes, support matrix, and maintainer contact path.
- [ ] Define release/versioning policy and migration notes before promising stable public APIs.

## How to propose or complete a roadmap item

Open an issue with the user problem, proposed scope, dependencies, acceptance criteria, expected platform support, licensing questions, tests, documentation, and measurable performance goal where applicable. Link the issue in the pull request. Break large editor or renderer initiatives into reviewable milestones. Update status only after behavior, tests, documentation, and limitations are verified.
