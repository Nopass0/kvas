# Script architecture: a script on its own object

In Kvas, as in Unity, behaviour lives **on objects**: a door knows how to open, the player how to walk, the camera how to look. One object can carry **several scripts** (the player: movement, interaction, inventory). Scripts talk through **signals** and **shared state**, not one huge file.

## How the build works

Every file in `scripts/src/` (except `lib.rs`) can hold scripts: a type with `impl Script for Name` registers itself. The generated `scripts/build.rs` drives the build:

* **only used scripts are compiled**: those on objects of scenes and prefabs (`assets/**/*.scene.json`), on objects of the scene open in the editor (`.kvas/used_scripts.txt`, written by the editor), or mentioned in the code of used scripts (`attach_script("Name")`, shared types). An unfinished script that is not on any object **cannot break the build** and costs nothing;
* files **without** scripts (shared code, constants, helpers) are always compiled;
* the build records whether each script has `update` and `fixed_update`. A script without them is **never called per frame**: no call, no variable exchange;
* script variables (its `fields()`) reach the Inspector and other scripts only when they **change**.

To check every script of a project at once (before publishing a library, for example):

```bash
KVAS_ALL_SCRIPTS=1 cargo build --manifest-path scripts/Cargo.toml
```

Attaching a script that the built library does not have yet starts a build by itself.

## One script, one job

```rust
// scripts/src/world/door.rs — on the door's hinge
use kvas_engine::prelude::*;

pub struct Door {
    pub open: bool,
    pub locked: bool,
    pub angle: f32,
    current: f32,
}

impl Default for Door {
    fn default() -> Self {
        Self { open: false, locked: false, angle: 95.0, current: 0.0 }
    }
}

impl Script for Door {
    fn update(&mut self, ctx: &mut Ctx) {
        // The player pressed E looking at the door: Interactor sent "use".
        if !ctx.signals("use").is_empty() && !self.locked {
            self.open = !self.open;
        }
        let target = if self.open { self.angle } else { 0.0 };
        self.current += (target - self.current).clamp(-160.0 * ctx.dt(), 160.0 * ctx.dt());
        ctx.transform_mut().rotation.y = self.current;
    }

    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![Field::bool("open", &mut self.open), Field::bool("locked", &mut self.locked), Field::f32("angle", &mut self.angle)]
    }
}
```

Make the leaf (model and collider) **children** of the hinge: turning, the hinge carries them along.

## How scripts talk

| Way | When | Example |
|---|---|---|
| **Signal to one** `ctx.send_to(e, "channel", data)` | an event for one object | player → door: `use` |
| **Signal to all** `ctx.emit("channel", data)` | an event several listen to | `ls.say` (subtitles), `ls.caught` |
| **Receiving** `ctx.signals("channel")` | in the receiver's `update` | `for s in ctx.signals("use") { ... }` |
| **Shared state** `ctx.set_global` / `ctx.global` | a value with one owner and many readers | game mode, story, settings |
| **Another script's variables** `ctx.var` / `ctx.set_var` | configure someone else's script | the director locks a door: `ctx.set_var(door, "Door", "locked", true.into())` |

A signal arrives **on the next frame** and is visible in that frame's `update`. Data is JSON (`serde_json::json!`).

Rule for shared state: **each key has one writer**. Only the director writes the story, only the UI the settings, only `Interactor` "what the player looks at". Everyone else reads.

## Example: LAST SERVICE

| Object | Scripts | What they do |
|---|---|---|
| Lida (player) | `PlayerController`, `Interactor`, `Inventory`, `RoomAcoustics` | walk/run/crouch and steps; the look ray and E; what is in hand; room echo |
| Eyes (player camera) | `MouseLook`, `CameraFx` | mouse and bob; fear, blood, VHS shaders |
| Flashlight | `Flashlight` | F, battery waver |
| every door | `Door` | hinge, lock, animation, sound |
| lamps | `FlickerLight` | flicker, power from the switch panel |
| kitchen fan | `Spinner` | turning blades |
| Counter (monster) | `MonsterAi` | patrol, sight, hearing, chase, animation clips |
| Anna, Pavel | `Npc` | work, turning to the player, talking (`Animator` clips) |
| Game | `GameDirector` | story, door locks, saves, music |
| UI | `GameUI` | menus, options, pause, journal, panel, HUD |

Shared types and helpers are in `scripts/src/shared.rs` (a file without scripts, always compiled).

## Engine documentation in the editor

**Help → API reference (Shift+F1)**: every public struct, field, enum, function and method of the engine with its full description from the sources (rustdoc). The code editor's hints come from the same reference (after `ctx.`, the methods of `Ctx`); the 📖 button of a hint or **F1** on the word under the cursor opens its full page.
