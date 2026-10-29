# Character animation (skeleton, clips, Animator)

A model with a skeleton (FBX or glTF/GLB with bones and weights) comes alive with the **Animator** component: it plays clips, blends between them and deforms the model every frame. Textures, shaders and shadows work as for any model.

## Quick start

1. Put a character model in the scene (`.fbx`, `.glb`).
2. **Add component → Animator.** The clip list shows every clip of the model. The animation plays in the Scene view, without starting the game.
3. In a script:

```rust
let monster = ctx.find("Counter").unwrap();
ctx.play_animation(monster, "Walk_Nervous");   // cross-fades from the current clip
ctx.set_animation_speed(monster, 1.4);         // steps in time with the real speed
let clips = ctx.animation_clips(monster);      // what the model offers
```

Full access goes through the component: `ctx.get_mut_of::<Animator>(e)` (`clip`, `speed`, `looping`, `playing`, `fade`; read-only `time` and `clips`).

## Where clips come from

* **The file.** Every FBX animation (`Rig|Walk` becomes `Walk`) and glTF animation. Walking clips are made "in place": the hips' straight forward drift is removed (the sway stays), because a script moves the object.
* **Procedural clips for humanoids.** When bones have familiar names (`hips`/`pelvis`, `spine`, `chest`, `neck`, `head`, `upper_arm`, `forearm`, `hand`, `thigh`, `shin`, `foot`, sides as `.L/.R`, `_left/_right`, `Left…`; Mixamo works), the model gets the clips its file lacks:

| Clip | What it does |
|---|---|
| `Idle` | breathing, weight shift, glances |
| `Walk`, `Run` | stride with arm and torso motion |
| `Talk` | nods and hand gestures |
| `Wave` | waves |
| `Sit` | sits (lower the object to the seat height) |
| `Wipe` | wipes a counter |
| `Look` | looks around |

The character's axes (up, right, forward) come from the rest pose, and arms in a T- or A-pose are brought down first, so the clips fit any model with such bones.

## Blending and accuracy

* Changing `clip` blends over `fade` seconds (0 is instant).
* `looping = false` holds the last frame (a strike, a fall).
* Skinning uses up to four bones per vertex, on the CPU: a few thousand vertices take microseconds.
* The rest pose matches the unanimated model exactly (checked by `cargo test -p kvas_engine --test skeletal`).

## Example: LAST SERVICE

* `MonsterAi` picks the clip from the monster's state: `Idle_Watchful`, `Walk_Nervous`, `Run_Frantic`, `Scream` at the first sighting, `Attack_Lunge` when it catches you; the clip speed follows the movement speed.
* The `Npc` script on Anna and Pavel: work (`Wipe`, Anna wipes the counter; `Look`, Pavel looks around), turning to the player with `Idle` when close, `Talk` while one of that character's lines plays, `Sit` at the end.
