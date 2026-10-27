# Kvas Engine

3D game engine in Rust: ECS core, wgpu renderer, Rapier physics, Rust scripts with hot reload,
and a Unity-style editor. Games are built into a single folder: `game.exe` + `game.pak` + scripts library.

## Quick start

```bash
cargo run -p kvas_editor                           # project hub: recent projects, open, create 3D / 2D
cargo run -p kvas_editor -- games/demo             # open a project directly (folder or its project.json)
cargo run -p kvas_editor -- games/demo --play      # open and start Play immediately
cargo run -p kvas_editor -- --build games/demo     # build the game without a window
cargo build -p kvas_player          # needed once, the editor copies it when you build a game
```

Use `--release` for both when you want speed. The editor builds scripts with the same profile it was built with.

Editor controls:

| Action | Input |
|---|---|
| Fly camera | hold RMB, then W A S D Q E (Shift = faster) |
| Pan / zoom / orbit | MMB drag / scroll / Alt + LMB drag |
| Focus selection | F (or double-click in Hierarchy) |
| Move / Rotate / Scale | W / E / R (Ctrl while dragging snaps) |
| Play / Stop | toolbar or Ctrl+P (changes made while playing are discarded) |
| Save, Undo, Redo, Duplicate, Delete | Ctrl+S, Ctrl+Z, Ctrl+Y, Ctrl+D, Del |
| Import models / textures / shaders | Project panel > Import, or drop files on the window (into the open folder) |
| Project panel | folder tree + thumbnail grid, GPU 3D previews of models and shaders, drag files onto folders |
| Input bindings | File > Project Settings > Input (named actions, rebindable from scripts) |
| Export PNG | File > Export View PNG (clean Scene/Game camera image) |
| Render statistics | hover the FPS/object counter in the viewport |
| Place a model | drag it from the Project panel into the Scene view |
| Reparent | drag a row onto another row in the Hierarchy |

Build a game: `File > Build Game`, or without a window: `kvas_editor --build path/to/project`.
The result is in `<project>/build/`.

## Layout

```
crates/kvas_ecs      sparse-set ECS (entities, components, queries). No dependencies.
crates/kvas_engine   components, scene (JSON), hierarchy, assets (+ .pak), meshes (primitives, OBJ, glTF),
                     Rapier physics, script host, game loop (Runtime), input, reflection
crates/kvas_render   wgpu renderer, WGSL shader prelude, debug lines, egui glue
crates/kvas_editor   the editor (panels in separate files; actions in editor.rs)
crates/kvas_player   runs a packed game or a project folder
games/demo           sample project (scene, shaders, model, Rust scripts)
third_party/egui_dock  egui_dock 0.21.1 retargeted to egui 0.37 (MIT)
```

A project is a folder: `project.json`, `assets/` (scenes, models, textures, shaders) and `scripts/` (a cdylib crate).

## Design notes (DRY / KISS)

* **One reflection mechanism.** A component or script lists its editable fields as `Field`s
  (`reflect.rs`). The Inspector, scene saving of script properties and undo all use it, so a new
  component is: a struct, `impl Component` with `fields()`, one line in `registry.rs`, one line in
  `components.rs::name_components!`.
* **One asset path.** Everything loads through `Assets` (folder in the editor, `.pak` in a shipped game).
* **One game loop.** `Runtime` is used by the editor's Play mode and by the player.
* **One renderer path.** The scene is rendered to a texture shown by egui, in the editor and in the player.
* **Play mode runs a copy** of the scene, so Stop restores it exactly. Undo is scene snapshots,
  committed once per interaction.

## Scripts (Rust)

Scripts live in `<project>/scripts/src/lib.rs`, are compiled by cargo into a dynamic library and
hot-reloaded when you save a file (the editor rebuilds automatically; "Build Scripts" forces it).

```rust
use kvas_engine::prelude::*;

pub struct Spinner { speed: Vec3 }
impl Default for Spinner { fn default() -> Self { Self { speed: Vec3::new(0.0, 90.0, 0.0) } } }

impl Script for Spinner {
    fn update(&mut self, ctx: &mut Ctx) {            // also: start, fixed_update (60 Hz, physics)
        let dt = ctx.dt();
        ctx.transform_mut().rotation += self.speed * dt;
    }
    fn fields(&mut self) -> Vec<Field<'_>> {          // shown in the Inspector, saved in the scene
        vec![Field::vec3("speed", &mut self.speed)]
    }
}
register_scripts!(Spinner);
```

Input actions: `ctx.action_down("Jump")`, `action_pressed`, `action_axis(neg, pos)`, `rebind(name, &["Space"])`.

`Ctx` gives: `transform()`, `get::<T>()`, `find(name)`, `spawn_primitive`, `despawn`, `add_impulse`,
`set_velocity`, `raycast`, `key_down` / `key_pressed`, `dt()`, `elapsed()`, `log()`, and `scene.world`
for anything else. Scripts can only use the built-in component types.

Rules that keep hot reload sound:
* The library and the host must come from the same compiler and engine sources. This is checked on
  load (`AbiInfo`); on mismatch you get "rebuild them" instead of a crash. New projects copy the
  engine's `Cargo.lock` so dependency layouts match.
* Statics are duplicated across the library boundary: talk to the engine through `Ctx` only (log with `ctx.log`).
* A panic in a script is caught; that script instance is disabled and the error shows in the Console.
* Hot reload recreates script instances, so script state is reset (exposed fields are kept).

## Rust API and documentation

The complete [Rust API guide](docs/ENGINE_API.md) is also compiled into
`kvas_engine::guide`, with executable examples. Public ECS, engine and renderer
items have rustdoc, including units, lifecycle, failure cases and native-DLL limits.

```powershell
cargo doc -p kvas_ecs -p kvas_engine -p kvas_render --no-deps --open
cargo test -p kvas_ecs -p kvas_engine -p kvas_render --doc
./tools/verify.ps1          # workspace check, tests, doctests and strict rustdoc
./tools/verify.ps1 -Gpu     # additionally run opt-in native GPU/readback tests
```

Available engine services include:

| Area | API |
|---|---|
| ECS | allocation-free entity iteration, joins of 2/3 types, disjoint mutable access, lazy/checked insertion, retention |
| Runtime | validated fixed-step policy, pause/single-step, time scale, bounded catch-up and frame diagnostics |
| Gameplay | Tags, inherited Visibility, subtree Lifetime, MoveTween with 14 easing curves |
| Messaging | bounded next-frame SignalBus with broadcast/directed JSON payloads; local typed EventQueue |
| Time | explicit one-shot/repeating Timer with pause, catch-up count and exact Duration accounting |
| Physics | CCD, live collision masks, filtered ray/sphere queries, solid/sensor contacts, dynamic-body teleport |
| Geometry | checked mesh topology, smooth normals, affine transforms, mirror winding, mesh append and area |
| Spatial | validated Ray/Aabb/Frustum, viewport picking and world-to-screen conversion |
| Persistence | validated scene save/snapshot, semantic project validation, transactional file replacement |
| Assets | zero-copy packed reads, checked paths/archives, deterministic repacking, dependency-aware GPU hot reload |
| Rendering | native windowless PNG capture, mesh/shadow submission statistics and extra debug shapes |

Native script instances retain their library for their entire lifetime. A fixed-layout
C compatibility probe is checked before exchanging Rust containers. Rebuild hosts and
all script libraries after an engine update. Full component literals should use
`..Default::default()` when they do not need to specify new physics fields.

Windowless export uses the same renderer as the editor/player:

```powershell
cargo run -p kvas_render --example capture_scene -- output.png
cargo run -p kvas_render --example capture_scene -- demo.png games/demo
```

## Shaders

A mesh shader is a WGSL file in `assets/shaders`. The engine prelude (`kvas_render/src/shaders/prelude.wgsl`)
is prepended: `globals` (view_proj, camera, lights, time), `object` (model, color, roughness...), the
albedo texture, `standard_vertex()`, `lighting()` and `to_display()`. You write `vs_main` and `fs_main`.
Assign it in a MeshRenderer. Saving the file reloads it live; compile errors appear in the Console
and the object falls back to the built-in shader. See `games/demo/assets/shaders/`.

## Physics

* `RigidBody` (Dynamic / Fixed / Kinematic) + `Collider` (box, sphere, capsule, cylinder, trigger).
* A `Collider` without its own body attaches to the nearest ancestor with a `RigidBody`: build
  compound structures from a hierarchy and they move as one body.
* `Joint`: Fixed (weld), Ball, Hinge (with limits), Spring. Joints attach where the objects currently
  are, to a target entity by name or to the world. Moving a kinematic body from a script moves what rests on it.

## Effects and test projects

* **Grass** (`Grass`): tens of thousands of blades merged into one mesh; wind is animated on the GPU by `builtin:grass`.
* **Cloth** (`Cloth`): Verlet cloth with wind on every triangle, tearing (`tear`), ground / sphere / box collisions and friction. Pin along the top edge, corners, left edge or none. Wind and other tunables can be changed from scripts every frame without restarting the simulation.
* **Voxel smoke** (`VoxelSmoke`): one mesh, hidden faces culled.
* **Volumetric clouds** (`VoxelClouds`): a 3D voxel density texture generated from fractal noise and ray-marched in `builtin:clouds` (sun scattering with a two-lobe phase function, self-shadowing, sky ambient, soft box edges, wind scrolling, and rays bent along the density gradient as an artistic stand-in for refraction). One box, one draw call.
* **Grass** is split into chunks that are frustum-culled; far blades thin out on the GPU. Chunks are `Transient` entities: hidden from the hierarchy and never saved. The renderer frustum-culls every object.
* **glTF/GLB models** import their base colour texture, including files that reference `Textures/*.png` next to the model. Cut-out textures (leaves) are alpha-tested.
* **Procedural geometry** is a `DynamicMesh` component written by systems (`kvas_engine::procedural`); the editor shows the initial state outside Play.
* Shaders get 8 free floats (`MeshRenderer.user`, `object.user0` / `object.user1`) and can use embedded `builtin:` shaders.
* Scene view modes (floating toolbar): Lit, Albedo, Normals, Depth (with 4 m contour rings), UV.

Test projects (regenerate with `cargo run -p kvas_engine --example make_tests` and `make_assets`): `games/test_meadow`, `games/test_voxels`, `games/test_cloth`, `games/test_assets` (a castle built from the Kenney Retro Medieval Kit GLBs in `.test_assets`). After changing the engine run `tools/refresh_tests.sh` to rebuild their script libraries. They share `games/test_shared/scripts.rs` (free-fly camera, orbit camera, wind controller, ball dropper).

## Known limits

* Lighting: one sun with a 2048 shadow map (40 m around the camera, PCF), up to 8 point/spot lights (no shadows for those), procedural sky with time of day, stars, moon, clouds (Scene Settings > Sky), glass and metal via `reflectivity` / `metallic` (Fresnel reflections of the sky; no refraction or screen-space reflections).
* No audio, no 2D UI or skeletal animation. Static OBJ, FBX (ufbx), glTF and GLB meshes are supported. Third-party asset packs are not bundled (itch.io packs must be downloaded by the user).
* Not implemented yet: ray tracing, DLSS / FSR, virtualized geometry, node-based materials, audio, particles, fluids, spot/point shadows, refraction and skeletal animation.
* glTF: static meshes with baked node transforms and one merged material/base-colour texture; external `.bin` buffers and textures are read through Assets. Multiple material draw sections, skinning and animation are not implemented.
* Scripts cannot define new component types; Play mode input is read from the editor window.
* Verified on Windows with the Rust 1.99 toolchain. Other platforms were not tested.
