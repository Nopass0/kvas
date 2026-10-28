# Quick start

Kvas is a Rust workspace: the editor (`kvas_editor`), the player (`kvas_player`), the engine and the renderer.
You need [Rust](https://rustup.rs) and a GPU with Vulkan or DirectX 12. Tested on Windows.

## Run the editor

```bash
cargo run -p kvas_editor                      # project hub: recent, open, create 3D / 2D
cargo run -p kvas_editor -- games/demo        # open a project directly
cargo run -p kvas_editor -- games/demo --play # open it and press Play
cargo build -p kvas_player                    # once: the editor copies it into a built game
```

Add `--release` for speed. The editor builds your scripts with the same profile it was built with.

![Demo scene](/images/demo.png)

## Controls in the scene

| Action | Input |
|---|---|
| Fly the camera | hold RMB, then W A S D Q E (Shift = faster) |
| Pan / zoom / orbit | MMB drag / wheel / Alt + LMB drag |
| Focus the selection | F |
| Move / Rotate / Scale | W / E / R (hold Ctrl while dragging to snap) |
| Play / Stop | Ctrl+P (changes made while playing are discarded) |
| Save, Undo, Redo, Duplicate, Delete | Ctrl+S, Ctrl+Z, Ctrl+Y, Ctrl+D, Del |
| Place a model | drag it from the Project panel into the scene |

## A project

A project is a folder: `project.json`, `assets/` (scenes, models, textures, shaders, materials) and `scripts/` (a `cdylib` crate).
Every new project already contains:

- `assets/materials/library/` - 80 procedural materials,
- `assets/shaders/library/` - ready-made object and camera shaders,
- `scripts/src/<folder>/` - starter scripts (movement, camera, gameplay, audio, world, UI).

Create your first script with **+ New script** in the Project panel, pick a template and save.
The editor builds the library and reloads the scripts without closing the scene.

## Build the game

`File → Build Game`, or without a window:

```bash
cargo run -p kvas_editor -- --build games/demo
```

The result is in `<project>/build/`: `game.exe`, `game.pak` and the scripts library.

## Next

- [Scripting](/en/scripting/) - the first script, the `Ctx` cheat sheet.
- [Writing scripts the easy way](/en/scripting/easy) - the `script!` macro, Inspector variables, debugging.
- [Cookbook](/en/scripting/cookbook) - ready scripts: chaser, patrol, spawner, camera, coins, jelly.
- [Vertices, UVs and smoothing](/en/scripting/mesh)
- [Node editor](/en/nodes/) and the [material catalogue](/en/nodes/materials).
