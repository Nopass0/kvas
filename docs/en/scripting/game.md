---
outline: [2, 3]
---

# Game systems of the SDK

Everything a complete game needs: a screen interface with your own fonts, menus and options, saves, files, scenes, cameras and their shaders, sound with room echo, particles, decals, trees and grass. All examples are script code (`impl Script` or `script!`). A whole game built on these systems is `games/last_service` (see [LAST SERVICE](../guide/last-service)).

## Screen interface: `ctx.ui()`

The interface is described **every frame** (immediate mode): a script says what is on screen in `update` and learns what the player clicked on the previous frame. Coordinates are in a virtual screen **720** units high; its width follows the window (`ctx.ui_width()`, 1280 at 16:9). Rectangles are `[x, y, width, height]` from the top-left corner.

| Method | Draws |
|---|---|
| `panel(rect, colour, radius)` | a filled panel |
| `frame(rect, colour, outline, radius)` | a panel with an outline |
| `text(text, [x, y], size, colour, font, Align::Left/Center/Right)` | one line |
| `paragraph(text, [x, y], width, size, colour, font)` | wrapped text |
| `image(path, rect, tint)` | an image from assets (crisp up to 256 px) |
| `button(id, rect, text) -> bool` | true on the frame after a click |
| `hovered(id)` | pointer over the button |
| `slider(id, rect, value) -> f32` | returns the new 0..1 value |
| `toggle(id, rect, label, on) -> bool` | ON/OFF switch |
| `set_style(ButtonStyle)` | font, size, colours, radius and alignment of the following buttons |

While buttons or sliders are on screen the cursor is freed; lock it again with `ctx.lock_cursor(true)`.

**Fonts:** give a `.ttf`/`.otf` asset path as `font` (or `ButtonStyle::font`); it loads on first use. Characters the font lacks fall back to the default font.

```rust
use kvas_engine::prelude::*;
const FONT: &str = "ui/fonts/terminal.ttf";

script! {
    /// Main menu with a volume slider.
    pub struct MainMenu { volume: f32 = 0.8, open: Local<bool> = Local(true) }
    update(self, ctx) {
        if !*self.open { return; }
        let w = ctx.ui_width();
        let ui = ctx.ui();
        ui.panel([0.0, 0.0, w, 720.0], [0.0, 0.0, 0.0, 0.6], 0.0);
        ui.set_style(ButtonStyle { font: FONT.into(), size: 28.0, ..Default::default() });
        if ui.button("play", [w / 2.0 - 150.0, 320.0, 300.0, 48.0], "PLAY") {
            *self.open = false;
            ctx.lock_cursor(true);
        }
        let v = ui.slider("volume", [w / 2.0 - 150.0, 400.0, 300.0, 18.0], self.volume);
        if ui.button("quit", [w / 2.0 - 150.0, 450.0, 300.0, 48.0], "QUIT") { ctx.quit_game(); }
        self.volume = v;
        ctx.set_volumes(v, v, v);
    }
}
```

## Saves and settings

Game data lives in its own folder: `%APPDATA%/Kvas/<game>` for a built game, `<project>/.kvas/data` in the editor.

| Method | |
|---|---|
| `ctx.save_game(slot, title, &json)` | write a slot (atomically) |
| `ctx.load_game(slot)` | read it |
| `ctx.save_slots()` | all slots, newest first (`slot`, `title`, `time`, `scene`) |
| `ctx.has_save(slot)`, `ctx.storage().delete_save(slot)` | |
| `ctx.storage().save_as(slot, title, &T)` / `load_as::<T>(slot)` | typed, any `Serialize` |
| `ctx.setting_f32 / setting_bool / setting_text(key, default)`, `ctx.set_setting(key, json)` | options kept across runs |

## Files

`ctx.storage()`: `read_text`, `write_text`, `read_bytes`, `write_bytes`, `append_line`, `list`, `exists`, `delete` — relative paths inside the game folder only. `kvas_engine::storage::external`: the same for any path on the computer (mods, import/export), plus `documents_dir()`.

## Scenes

`ctx.load_scene(path)`, `ctx.add_scene(path)`, `ctx.reload_scene()`, `ctx.scene_path()`. Time, bindings, cursor and the save folder carry on; physics and scripts start over.

## Cameras

`ctx.active_camera()`, `set_active_camera(e)`, `set_active_camera_named(name)`, `place_camera(e, position, target)`, `set_camera_fov(e, degrees)`, `camera_position()`, `camera_forward()`, `add_camera_effect(e, shader, params)`, `set_camera_effect(e, index, on, params)`.

Each camera has its own shader list (`Camera.effects`, Inspector "Camera shaders"), used while it is active, after the scene's effects. **"Look through this camera"** in the Inspector shows the scene from it in the editor. Camera shader thumbnails show a sample scene through the shader.

## Sound mix and room echo

`play_sound`, `play_sound_at`, `play_music` (own volume in the mix), `stop_sound`, `set_sound_volume`, `set_volumes(master, music, effects)`, `set_reverb(amount 0..1, room_ms)` — echo for new sounds (20 ms closet, 40 room, 120 hall). `AudioSource` gives positional stereo, distance falloff and a directional cone; the listener is the active camera.

## Particles

Presets: `fire`, `smoke`, `sparks`, `snow`, `rain`, `heavy_rain`, `drizzle`, `magic`, `dust`, `fog`, `mist`, `steam`, `embers`, `fireflies`, `leaves`, `bubbles`, `campfire_smoke`, `dust_storm`, `explosion`. New fields: `stretch` (streaks along motion), `collide_floor` + `floor_height`, `splash` (droplets at impact), `follow_camera` (rain around the player), `soft` (fade where they cut into surfaces).

## Decals

`Decal` projects onto everything inside its box along local −Y (+Y points out of the surface); X/Z scale is the size, Y the reach. Any geometry, no UVs needed. Fields: `texture`, `normal_map`, `orm_map`, `shape` (Blob, Scorch, BulletHole, Cracks, Puddle, Grunge, PaintSplat, Moss, Rust, Leaks, Ring), `blend` (Normal, Multiply, Screen, Lighten, Darken, Overlay, Add, SoftLight, ReliefOnly, ColorDodge, ColorBurn, Tint), `color`, `opacity`, `roughness`, `metallic`, `emission`, `normal_strength`, `angle_fade`, `order`, `max_distance`. Puddles reflect what is on screen (screen-space reflections) and ripple with rain.

Placing: Create → Decal; **Ctrl+click** a surface with a decal selected; drag an image with **Alt** to create one. Each decal is one box drawn only where visible; decals of one image are batched; far ones are skipped by `max_distance`. Presets: `blood`, `scorch`, `bullet_hole`, `cracks`, `puddle`, `dirt`, `paint`, `moss`, `rust`, `leaks`, `graffiti_glow`, `light_spot`.

## Trees and grass

`Tree`: `style` (Broadleaf, Conifer, Palm, Bush), `height`, `trunk_radius`, `crookedness`, `levels`, `branches`, `branch_length`, `branch_angle`, `leaf_density`, `leaf_size`, bark and two leaf colours, `wind`, `seed`. `Grass`: `round` (soft-edged ellipse), `flowers`.

## Shaders that see behind transparent surfaces

Transparent objects (alpha < 1, or a shader with `water` / `glass` in its name) can call `scene_depth_behind(in.clip, in.world_pos)`, `scene_point_behind(...)` and `scene_color_behind(in.clip)` — for foam, soft edges, caustics, refraction and blend modes. A `// kvas:alpha_to_coverage` shader gets smooth cut-out edges; `// kvas:decal` draws as a decal.

## Antialiasing

`settings.post.antialias` (0..1, default 1): FXAA on top of 4× MSAA. Textures have mipmaps and 8× anisotropic filtering.

## Colliders shaped like the model

`Collider` has shapes taken from the object's own model (`MeshRenderer.mesh`):

* `ShapeKind::Mesh`: the model's exact triangles. For static level geometry: walls, floors, counters, cupboards.
* `ShapeKind::Hull`: the model's convex hull. For things that move (`RigidBody`): chairs, crates, barrels.

For them `size` multiplies the model (usually `1, 1, 1`); the object's scale is applied. The model is read once and cached.

```rust
// A wall from a model collides exactly with its shape.
scene.world.insert(wall, Collider { shape: ShapeKind::Mesh, ..Default::default() });
// A chair that can be pushed.
scene.world.insert(chair, Collider { shape: ShapeKind::Hull, ..Default::default() });
scene.world.insert(chair, RigidBody { mass: 6.0, ..Default::default() });
```

A ray that only finds trigger areas (a thing behind a thin counter rim, say): `QueryOptions { only_triggers: true, .. }`; a second, solid-only ray then tells whether it is blocked.
