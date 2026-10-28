# Writing scripts the easy way

How to write a script with a minimum of code, expose variables to the Inspector neatly and debug without pain.

## One block, one script: `script!`

The `script!` macro declares **variables with defaults** and **callbacks** in one place. It generates `Default`, the Inspector
`fields()` and the `Script` implementation for you.

```rust
use kvas_engine::prelude::*;

script! {
    /// Spins the object and counts frames.
    pub struct Spinner {
        speed: f32 = 90.0,             // Inspector: a number
        axis: Vec3 = Vec3::Y,          // Inspector: three X Y Z fields
        enabled: bool = true,          // a checkbox
        label: String = "spin".into(), // a text field
        frames: Local<u32> = Local(0), // Local = script state, hidden and not saved
    }

    start(self, ctx) {
        ctx.log(format!("{} started", self.label));
    }

    update(self, ctx) {
        if self.enabled {
            ctx.rotate(self.axis * self.speed * ctx.dt());
        }
        *self.frames += 1;
    }
}

register_scripts!(Spinner);
```

Callbacks are `start`, `update` and `fixed_update` (any of them, in any order). The first word in the parentheses is the name of `self`, the second is the context.

### Types you can expose

| Type | Widget |
|---|---|
| `f32`, `i32`, `u32` | number field |
| `bool` | checkbox |
| `Vec3` | three fields X Y Z |
| `String` | text field |
| `Color` | colour picker |
| `EntityRef` | pick an object by name |
| `MeshRef`, `TextureRef`, `SoundRef` | file picker of the right kind |
| any `enum` implementing `Choice` | drop-down |
| `Local<T>` | hidden |

## Already have a struct? `fields!`

```rust
#[derive(Default)]
struct Door { speed: f32, open: bool, name: String }

impl Script for Door {
    fn update(&mut self, ctx: &mut Ctx) { /* ... */ }
    fn fields(&mut self) -> Vec<Field<'_>> { fields!(self; speed, open, name) }
}
```

One list instead of `Field::f32("speed", &mut self.speed)` per field. The Inspector and scene key is the variable name.

## Debugging

Everything lands in the editor Console: warnings in yellow, errors in red.

| Method | What it does |
|---|---|
| `ctx.log("text")` | a plain line |
| `ctx.warn("text")` | a yellow warning |
| `ctx.error("text")` | a red error; the script keeps running |
| `ctx.watch("speed", self.speed)` | prints `speed = 4.0` (via `Debug`) |
| `ctx.watch_every(1.0, "pos", ctx.position())` | same, at most once per second - safe inside `update` |

```rust
update(self, ctx) {
    ctx.watch_every(0.5, "position", ctx.position());
    if ctx.distance_to(self.target).is_none() {
        ctx.warn("the target is gone");
    }
}
```

::: tip A panic is not fatal
A `panic!` in a script does not bring the editor down: the instance is disabled and the Console shows the script and object name.
Fix the code and save - the script reloads.
:::

## Tests instead of manual runs

Scripts run without a window, so logic is easy to test (see [the cookbook](./cookbook#how-to-test-a-script-without-the-editor)).

## Smoothing and easing

Everything is in the `prelude`:

| Function | Use |
|---|---|
| `lerp(a, b, t)`, `inverse_lerp`, `remap`, `remap_clamped` | interpolation and range mapping |
| `smoothstep(e0, e1, x)`, `ease_smooth(t)`, `ease_smoother(t)` | S curves |
| `approach(cur, target, max_step)` | move towards a target without overshoot |
| `damp(cur, target, lambda, dt)` | exponential smoothing, frame-rate independent |
| `smooth_damp(cur, target, velocity, time, dt)` | a spring without overshoot: returns `(value, velocity)` |
| `approach_degrees`, `delta_degrees`, `wrap_degrees` | angles along the short way |

```rust
// A camera eases towards a target; the velocity is kept between frames in a Local field.
update(self, ctx) {
    let (x, v) = smooth_damp(ctx.position().x, self.target_x, *self.vx, 0.3, ctx.dt());
    *self.vx = v;
    ctx.translate(Vec3::new(x - ctx.position().x, 0.0, 0.0));
}
```
