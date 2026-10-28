# Script cookbook

Every recipe is a finished script. The code on this page is **pulled straight from the tests**
(`crates/kvas_engine/tests/cookbook.rs`): it is compiled and run in CI, so it cannot go stale.

To use a recipe: copy the struct into `<project>/scripts/src/`, add the type to `register_scripts!(...)` and drag the script onto an object.
Number fields that default to `0` are set in the Inspector (`speed` and `stop_distance` for `Chaser`).

## Spinning with an Inspector setting

A field declared in `fields()` shows in the Inspector and is saved in the scene.

<<< ../../../crates/kvas_engine/tests/cookbook.rs#spinner{rust}

## A chaser

Walks to the object named `Player`, faces it and stops at a distance. `look_at_flat` changes only the heading;
`move_towards` returns `true` when the point is reached.

<<< ../../../crates/kvas_engine/tests/cookbook.rs#chaser{rust}

## Patrol between points

Keeps the route and the index of the next point. State (`next`) is reset by a hot reload - that is fine.

<<< ../../../crates/kvas_engine/tests/cookbook.rs#patrol{rust}

## A ball spawner

`Timer::repeating` counts the interval, `Rng` gives repeatable randomness, `despawn_after` cleans up.

<<< ../../../crates/kvas_engine/tests/cookbook.rs#spawner{rust}

## A third-person camera

Exponential smoothing `1 - exp(-k * dt)` feels the same at 30 and 144 FPS.

<<< ../../../crates/kvas_engine/tests/cookbook.rs#camera_follow{rust}

## A coin

Checks what is nearby by the `player` tag (add the tag in the Inspector or with `ctx.add_tag`).

<<< ../../../crates/kvas_engine/tests/cookbook.rs#pickup{rust}

## Shooting with a ray

`raycast` returns a `RayHit`. Push the hit body through its `RigidBody` component.

<<< ../../../crates/kvas_engine/tests/cookbook.rs#shooter{rust}

## Messages: a button and a door

Scripts do not know each other: the button broadcasts a signal, the door listens. Signals are delivered next frame.

<<< ../../../crates/kvas_engine/tests/cookbook.rs#signals{rust}

## Vertex deformation and shared variables

`make_editable` turns a built-in primitive into an editable mesh, `editable(e)` gives access to the vertices:
`wave`, `twist`, `taper`, `bend`, `noise_displace`, `smooth`, `inflate`. `set_global` / `global_f32` / `add_global` are variables shared by all scripts.

<<< ../../../crates/kvas_engine/tests/cookbook.rs#jelly{rust}

## How to test a script without the editor

Scripts are ordinary types, so they run inside a test - exactly what this cookbook does:

```rust
let host = ScriptHost::from_defs(vec![ScriptDef::of::<Chaser>("Chaser")]).unwrap();
let mut runtime = Runtime::new(scene, Some(host));
for _ in 0..240 {
    runtime.tick(1.0 / 60.0);
}
assert!(runtime.scene.world.get::<Transform>(chaser).unwrap().position.x > 5.0);
```
