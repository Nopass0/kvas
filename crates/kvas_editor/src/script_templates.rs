//! Starter code for "New script". Every template is plain, commented Rust that already uses the
//! engine's prelude, so a new file compiles and can be edited straight away.

use crate::i18n::t;

/// One selectable template: label, description, source with `NAME` placeholders.
pub struct Template {
    pub label: &'static str,
    pub hint: &'static str,
    pub body: &'static str,
}

pub const TEMPLATES: &[Template] = &[
    Template {
        label: "Empty script",
        hint: "A script with update() and one editable number.",
        body: "
script_params! {
    /// NAME: describe what this script does.
    pub struct NAME {
        /// Shown in the Inspector as a number field.
        speed: f32 = 1.0,
        enabled: bool = true,
    }
}

impl Script for NAME {
    /// Called once when the game starts.
    fn start(&mut self, ctx: &mut Ctx) {
        ctx.log(\"NAME started\");
    }

    /// Called every frame. `ctx.dt()` is the frame time in seconds.
    fn update(&mut self, ctx: &mut Ctx) {
        if !self.enabled {
            return;
        }
        let _ = (self.speed, ctx.dt());
    }

    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
",
    },
    Template {
        label: "Spinner",
        hint: "Rotates the object continuously.",
        body: "
script_params! {
    /// NAME: rotates the object, degrees per second around each axis.
    pub struct NAME {
        degrees_per_second: Vec3 = Vec3::new(0.0, 90.0, 0.0),
    }
}

impl Script for NAME {
    fn update(&mut self, ctx: &mut Ctx) {
        let step = self.degrees_per_second * ctx.dt();
        ctx.transform_mut().rotation += step;
    }

    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
",
    },
    Template {
        label: "Player movement",
        hint: "WASD movement relative to the object, Shift to sprint.",
        body: "
script_params! {
    /// NAME: moves the object with the MoveForward/MoveBack/MoveLeft/MoveRight actions.
    pub struct NAME {
        speed: f32 = 4.0,
        sprint_multiplier: f32 = 1.8,
    }
}

impl Script for NAME {
    fn update(&mut self, ctx: &mut Ctx) {
        // x = right/left, y = forward/back, each in -1..1 (actions are set in Project Settings > Input).
        let input = ctx.action_vector(\"MoveLeft\", \"MoveRight\", \"MoveBack\", \"MoveForward\");
        let speed = if ctx.action_down(\"Sprint\") { self.speed * self.sprint_multiplier } else { self.speed };
        let step = (Vec3::new(input.x, 0.0, -input.y)) * speed * ctx.dt();
        ctx.transform_mut().position += step;
    }

    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
",
    },
    Template {
        label: "First-person player",
        hint: "WASD, mouse look, jump, sprint, crouch on a capsule body.",
        body: "
script_params! {
    /// First-person player on a capsule body: WASD, mouse look, Space to jump, Shift to sprint,
    /// Ctrl to crouch, Esc to free or recapture the mouse. Put it on the body (capsule collider +
    /// rigid body with \"lock rotation\") and make the camera a child object named \"Camera\".
    pub struct NAME {
        walk_speed: f32 = 4.2,
        sprint_speed: f32 = 7.5,
        crouch_speed: f32 = 2.2,
        jump_speed: f32 = 5.2,
        /// How quickly the player reaches the wanted speed (higher = snappier).
        acceleration: f32 = 14.0,
        look_sensitivity: f32 = 0.11,
        /// Camera height above the body centre in metres.
        eye_height: f32 = 0.7,
        camera: EntityRef = EntityRef(\"Camera\".into()),
        yaw: Local<f32> = Local(0.0),
        pitch: Local<f32> = Local(0.0),
        crouch: Local<f32> = Local(0.0),
        bob: Local<f32> = Local(0.0),
        captured: Local<bool> = Local(true),
    }
}

impl Script for NAME {
    fn start(&mut self, ctx: &mut Ctx) {
        // Hide the cursor and use the mouse for looking as soon as the game starts.
        ctx.lock_cursor(true);
        ctx.hud_crosshair(true);
    }

    fn update(&mut self, ctx: &mut Ctx) {
        let dt = ctx.dt();
        if ctx.key_pressed(Key::Escape) {
            self.captured.0 = !self.captured.0;
            ctx.lock_cursor(self.captured.0);
        }
        if self.captured.0 {
            let look = ctx.input.mouse_delta;
            self.yaw.0 -= look.x * self.look_sensitivity;
            self.pitch.0 = (self.pitch.0 - look.y * self.look_sensitivity).clamp(-85.0, 85.0);
        }

        // Movement relative to where the player looks.
        let input = ctx.action_vector(\"MoveLeft\", \"MoveRight\", \"MoveBack\", \"MoveForward\");
        let crouching = ctx.key_down(Key::Ctrl);
        let speed = if crouching { self.crouch_speed } else if ctx.action_down(\"Sprint\") { self.sprint_speed } else { self.walk_speed };
        let yaw = self.yaw.0.to_radians();
        let forward = Vec3::new(-yaw.sin(), 0.0, -yaw.cos());
        let right = Vec3::new(yaw.cos(), 0.0, -yaw.sin());
        let wish = (forward * input.y + right * input.x).clamp_length_max(1.0) * speed;
        let velocity = ctx.velocity();
        let k = 1.0 - (-self.acceleration * dt).exp();
        let horizontal = Vec3::new(velocity.x, 0.0, velocity.z).lerp(wish, k);

        // Standing on something? A short ray just below the capsule tells.
        let feet = ctx.world_position() - Vec3::Y * 0.95;
        let grounded = ctx.raycast(feet, Vec3::NEG_Y, 0.25).is_some();
        let mut vertical = velocity.y;
        if grounded && ctx.action_pressed(\"Jump\") {
            vertical = self.jump_speed;
        }
        ctx.set_velocity(Vec3::new(horizontal.x, vertical, horizontal.z));

        // Camera: look angles, smooth crouch and a little head bob while walking.
        self.crouch.0 += ((crouching as i32 as f32) - self.crouch.0) * (1.0 - (-10.0 * dt).exp());
        let moving = (horizontal.length() / self.walk_speed).min(1.5) * grounded as i32 as f32;
        self.bob.0 += horizontal.length() * dt * 2.2;
        let height = self.eye_height - 0.45 * self.crouch.0 + self.bob.0.sin() * 0.035 * moving;
        let (pitch, yaw_deg) = (self.pitch.0, self.yaw.0);
        if let Some(cam) = self.camera.resolve(ctx) {
            if let Some(t) = ctx.get_mut_of::<Transform>(cam) {
                t.position = Vec3::new(0.0, height, 0.0);
                t.rotation = Vec3::new(pitch, yaw_deg, 0.0);
            }
        }
    }

    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
",
    },
    Template {
        label: "Sound on key",
        hint: "Plays a sound when a key is pressed, with 3D position and pitch.",
        body: "
script_params! {
    /// NAME: plays a clip from assets/ when the Fire action is pressed.
    pub struct NAME {
        clip: SoundRef = SoundRef(String::new()),
        volume: f32 = 1.0,
        /// 1 = original pitch, 2 = an octave higher.
        pitch: f32 = 1.0,
        /// Play at the object's position (quieter when far away, panned left/right).
        positional: bool = true,
    }
}

impl Script for NAME {
    fn update(&mut self, ctx: &mut Ctx) {
        if self.clip.0.is_empty() || !ctx.action_pressed(\"Fire\") {
            return;
        }
        if self.positional {
            let at = ctx.world_position();
            ctx.play_sound_at(&self.clip.0, self.volume, at);
        } else {
            ctx.play_sound_pitched(&self.clip.0, self.volume, self.pitch);
        }
    }

    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
",
    },
    Template {
        label: "Day-night cycle",
        hint: "Runs the sky clock; clouds follow the hour.",
        body: "
script_params! {
    /// NAME: advances the time of day and thickens clouds towards evening.
    pub struct NAME {
        /// Game hours per real second (1.0 = a full day in 24 seconds).
        hours_per_second: f32 = 0.25,
        start_hour: f32 = 9.0,
    }
}

impl Script for NAME {
    fn start(&mut self, ctx: &mut Ctx) {
        ctx.set_time_of_day(self.start_hour);
        ctx.set_day_speed(self.hours_per_second);
    }

    fn update(&mut self, ctx: &mut Ctx) {
        let hour = ctx.time_of_day();
        let evening = ((hour - 15.0) / 5.0).clamp(0.0, 1.0);
        ctx.set_clouds(0.3 + 0.4 * evening);
    }

    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}
",
    },
];

/// Imports at the top of a script module.
pub const MODULE_HEADER: &str = "//! NAME script. Everything a script needs comes from the engine prelude:\n//! Ctx (time, input, sound, sky, physics), Vec3, Transform, components and the script macros.\n\nuse kvas_engine::prelude::*;\n";

pub fn label(i: usize) -> &'static str {
    t(TEMPLATES[i].label)
}

pub fn hint(i: usize) -> &'static str {
    t(TEMPLATES[i].hint)
}

/// Converts `DoorOpener` to `door_opener` for a module file name.
pub fn module_name(name: &str) -> String {
    let mut out = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_ascii_uppercase() && i > 0 {
            out.push('_');
        }
        out.push(c.to_ascii_lowercase());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn module_names_are_snake_case() {
        assert_eq!(module_name("DoorOpener"), "door_opener");
        assert_eq!(module_name("Spin"), "spin");
    }
}

/// Starter camera shader (see "Camera shaders" in the scene settings).
pub const POST_EFFECT_TEMPLATE: &str = "// Camera shader. Runs on the finished picture of the camera.
//
// You write one function; it is called for every pixel:
//   uv     0..1 across the screen (0,0 = top left)
//   color  the picture at this pixel (display colours)
//   params the four numbers from the scene settings, editable live (and from scripts:
//          ctx.set_post_effect_params(index, [a, b, c, d]))
//   time   seconds since the game started
//
// Helpers you can call: scene_color(uv) samples the picture anywhere, screen_aspect(), pixel_size().
// Shared function libraries are pulled in with #include (the whole file is searched for lib_ functions):
//   builtin:lib/color  lib_luma, lib_hue_shift, lib_saturation, lib_contrast, lib_vignette ...
//   builtin:lib/noise  lib_hash21, lib_noise, lib_fbm
//   builtin:lib/math   lib_remap, lib_pulse, lib_rot2
// You can also include your own file: #include \"shaders/lib/my_functions.wgsl\"
#include \"builtin:lib/color\"
#include \"builtin:lib/noise\"

fn effect(uv: vec2<f32>, color: vec3<f32>, params: vec4<f32>, time: f32) -> vec3<f32> {
    // params.x = strength of the warm grade, params.y = vignette
    var c = lib_saturation(color, 1.0 + 0.25 * params.x);
    c = lib_contrast(c, 1.0 + 0.1 * params.x);
    c *= vec3<f32>(1.04, 1.0, 0.94 + 0.0 * params.x);
    c *= lib_vignette(uv, params.y);
    return c;
}
";
