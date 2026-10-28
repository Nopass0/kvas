# Scripting in Rust

A script is an ordinary Rust struct. It is compiled into a dynamic library next to the project and
**hot-reloaded when you save the file**, without closing the editor.

## Your first script

The **+ New script** button of the Project panel creates a file from a template. By hand
(`<project>/scripts/src/spinner.rs`):

<<< ../../../crates/kvas_engine/tests/cookbook.rs#spinner{rust}

Register it once in `lib.rs`:

```rust
use kvas_engine::prelude::*;

mod spinner;
use spinner::Spinner;

register_scripts!(Spinner);
```

Then drag the script from the Project panel onto an object. Fields returned from `fields()` appear in the Inspector and are saved in the scene.

::: tip Even shorter
The [`script!` macro](./easy) declares variables and callbacks in one block, with no `Default` and no `fields()`.
:::

## Lifecycle

| Method | Called | Use it for |
|---|---|---|
| `start(&mut self, ctx)` | once, before the first frame of the instance | setup, finding objects, default values |
| `update(&mut self, ctx)` | every frame | movement, input, logic |
| `fixed_update(&mut self, ctx)` | every physics step (60 Hz) | forces and impulses |
| `fields(&mut self)` | when the Inspector or the scene needs the fields | Inspector and saving |

A hot reload creates a new instance and calls `start` again: **script state is reset**, field values from `fields()` are kept.

## What `Ctx` gives you

`Ctx` is the only door into the engine.

### Time

| Method | Description |
|---|---|
| `dt()` | seconds since the previous call (the physics step in `fixed_update`) |
| `elapsed()`, `frame()`, `fps()` | game time, frame number, FPS |
| `every(seconds)` | `true` once per N seconds |
| `time_of_day()`, `set_time_of_day(h)`, `is_night()` | day / night |

### Position and movement

| Method | Description |
|---|---|
| `position()` / `position_of(e)` | world position of this / another object |
| `set_position(p)` | place at a world point |
| `translate(d)` / `translate_local(d)` | move along the parent's / own axes |
| `rotate(deg)` | add Euler degrees |
| `forward()`, `right()`, `up()` | the object's directions |
| `look_at(p)` / `look_at_flat(p)` | face a point (with pitch / yaw only) |
| `move_towards(p, speed)` | walk to a point, `true` on arrival |
| `distance_to(e)`, `direction_to(p)` | distance and direction |
| `move_to(e, p, seconds, ease)` | smooth tween of any object |
| `teleport(p, reset_velocity)` | physics-aware teleport |

### Finding, spawning, removing

| Method | Description |
|---|---|
| `find(name)`, `find_all(name)`, `find_with::<T>()` | by name or by component |
| `spawn(name)`, `spawn_primitive(name, "sphere", pos)` | create an object |
| `despawn(e)`, `despawn_self()`, `despawn_after(e, seconds)` | remove |
| `set_color(e, rgba)`, `set_visible(e, bool)`, `set_texture(e, path)` | appearance |
| `attach_script(e, "Name")` | add a script to an object |

### Hierarchy, tags and shared variables

| Method | Description |
|---|---|
| `parent()`, `parent_of(e)`, `root()` | parent, another object's parent, hierarchy root |
| `children(e)`, `descendants(e)`, `find_child(name)` | children, all descendants, descendant by name |
| `tagged(tag)`, `tagged_within(tag, center, radius)`, `nearest_tagged(tag)`, `tags_of(e)` | tags |
| `set_global(key, json)`, `global_f32(key, default)`, `global_bool(..)`, `add_global(key, delta)` | variables shared by all scripts (score, difficulty) |
| `var(e, "Script", "field")`, `set_var(e, "Script", "field", json)` | a field of a script on another object |

### Physics

| Method | Description |
|---|---|
| `add_impulse(v)`, `set_velocity(v)`, `velocity()` | drive this object's body |
| `raycast(origin, dir, max)` | returns `RayHit { entity, point, normal, distance }` |
| `overlap_sphere(center, r)`, `touching(e)` | who is near / touching |

### Input and UI

| Method | Description |
|---|---|
| `key_down(Key::W)`, `key_pressed(..)`, `mouse_pressed(MouseButton::Left)` | raw input |
| `action_down("Jump")`, `action_axis("Left", "Right")`, `action_vector(..)` | named actions (File → Project Settings → Input) |
| `hud_message(text, seconds)`, `hud_prompt(text)`, `hud_objective(text)` | on-screen text |
| `lock_cursor(true)` | capture the cursor for a shooter |

### Sound, sky, effects

`play_sound("sfx/hit.wav", 1.0)`, `play_music(..)`, `play_sound_at(clip, volume, pos)`, `sky()`, `set_clouds(0.5)`,
`post_preset("cinematic")`, `set_exposure(1.2)`.

The `AudioSource` component supports **directional sound**: `cone_angle` (degrees, 360 = every direction) sets a beam along the
object's -Z axis; outside the cone the volume falls to 10 %. Stereo position follows the listener (the first camera).
Selecting a source in the editor draws its audible radii and the cone.

### Messages between scripts

`ctx.emit("open_door", json!({ "who": "player" }))` broadcasts, `ctx.send_to(entity, "damage", json!(10))` targets one object,
`ctx.signals("open_door")` reads what arrived. Messages are delivered next frame.

### Debugging

`ctx.log`, `ctx.warn`, `ctx.error`, `ctx.watch("name", value)`, `ctx.watch_every(1.0, "name", value)` - see [the easy way](./easy#debugging).

## Rules that keep hot reload sound

- The library and the editor must be built by the **same** compiler from the **same** engine sources. On a mismatch you get a "rebuild" message instead of a crash.
- Statics are duplicated across the library boundary: talk to the engine only through `Ctx` (log with `ctx.log`).
- A `panic!` in a script does not bring the editor down: the instance is disabled and the Console shows why.
- Scripts cannot define new component types - use the built-in ones and script fields.

## Next

- [Writing scripts the easy way](./easy)
- [Cookbook](./cookbook)
- [Vertices, UVs and smoothing](./mesh)
- [Rust API reference (rustdoc)](https://nopass0.github.io/kvas/api/kvas_engine/index.html)
