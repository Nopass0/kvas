#!/usr/bin/env python3
"""Generates the starter script library of the editor (crates/kvas_editor/library/scripts/generated/**).

Every script is a plain Rust file in the style of the hand-written ones (script_params! + impl Script), so the
editor's auto-registration finds it. Variants of one template differ in their defaults and, where it makes sense,
in behaviour. Run from the repository root:  python tools/gen_scripts.py
"""
import os
import shutil

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "crates", "kvas_editor", "library", "scripts", "generated")
FILES = {}  # relative path -> text
NAMES = set()


def snake(name):
    out = ""
    for i, c in enumerate(name):
        if c.isupper() and i and not name[i - 1].isupper():
            out += "_"
        out += c.lower()
    return out


def add(folder, name, doc, fields, body, extra_use=""):
    """fields: list of (name, type, default). body: the update() body."""
    assert name not in NAMES, name
    NAMES.add(name)
    decl = "".join(f"        {n}: {t} = {d},\n" for n, t, d in fields)
    text = f"use kvas_engine::prelude::*;\n{extra_use}\nscript_params! {{\n    /// {doc}\n    pub struct {name} {{\n{decl}    }}\n}}\n\n"
    text += f"impl Script for {name} {{\n    fn update(&mut self, ctx: &mut Ctx) {{\n{body}    }}\n    fn fields(&mut self) -> Vec<Field<'_>> {{\n        self.params()\n    }}\n}}\n"
    FILES[f"{folder}/{snake(name)}.rs"] = text


def f(x):
    return repr(float(x)) if "." in repr(float(x)) else f"{float(x)}"


AXES = {"X": "Vec3::X", "Y": "Vec3::Y", "Z": "Vec3::Z"}

# ------------------------------------------------------------------ transform animators
for axis, vec in AXES.items():
    for label, speed in [("Slow", 20), ("", 90), ("Fast", 360)]:
        add("animate", f"Spin{axis}{label}", f"Rotates around the {axis} axis ({speed} degrees per second by default).", [("degrees_per_second", "f32", f(speed))],
            f"        let step = {vec} * self.degrees_per_second * ctx.dt();\n        ctx.rotate(step);\n")

for axis, vec in AXES.items():
    for label, amp, spd in [("Small", 0.1, 2.0), ("", 0.3, 2.0), ("Big", 1.0, 1.0)]:
        add("animate", f"Bob{axis}{label}", f"Moves up and down along {axis} like a buoy.", [("amplitude", "f32", f(amp)), ("speed", "f32", f(spd)), ("origin", "Local<Option<Vec3>>", "Local(None)")],
            f"        let origin = *self.origin.0.get_or_insert(ctx.transform().position);\n        let wave = (ctx.elapsed() * self.speed).sin() * self.amplitude;\n        ctx.transform_mut().position = origin + {vec} * wave;\n")

for label, lo, hi, spd in [("Gentle", 0.9, 1.1, 2.0), ("", 0.8, 1.2, 3.0), ("Strong", 0.6, 1.5, 3.0), ("Heartbeat", 0.9, 1.25, 7.0), ("Breathing", 0.95, 1.05, 1.0), ("Pop", 0.5, 2.0, 1.5)]:
    add("animate", f"PulseScale{label}", "Scales the object smoothly between a minimum and a maximum.", [("min_scale", "f32", f(lo)), ("max_scale", "f32", f(hi)), ("speed", "f32", f(spd)), ("base", "Local<Option<Vec3>>", "Local(None)")],
        "        let base = *self.base.0.get_or_insert(ctx.transform().scale);\n        let k = 0.5 + 0.5 * (ctx.elapsed() * self.speed).sin();\n        ctx.transform_mut().scale = base * (self.min_scale + (self.max_scale - self.min_scale) * k);\n")

for axis, vec in AXES.items():
    for label, deg, spd in [("Rock", 10, 2.0), ("Sway", 25, 1.0), ("Shake", 4, 14.0)]:
        add("animate", f"Wobble{axis}{label}", f"Swings around {axis} by +-{deg} degrees.", [("degrees", "f32", f(deg)), ("speed", "f32", f(spd)), ("base", "Local<Option<Vec3>>", "Local(None)")],
            f"        let base = *self.base.0.get_or_insert(ctx.transform().rotation);\n        ctx.transform_mut().rotation = base + {vec} * (ctx.elapsed() * self.speed).sin() * self.degrees;\n")

for axis, vec in AXES.items():
    for label, dist, spd in [("Short", 1.0, 1.0), ("", 3.0, 1.5), ("Long", 8.0, 2.5)]:
        add("animate", f"Slide{axis}{label}", f"Slides back and forth along {axis}.", [("distance", "f32", f(dist)), ("speed", "f32", f(spd)), ("origin", "Local<Option<Vec3>>", "Local(None)")],
            f"        let origin = *self.origin.0.get_or_insert(ctx.transform().position);\n        let k = 0.5 + 0.5 * (ctx.elapsed() * self.speed * 0.5).sin();\n        ctx.transform_mut().position = origin + {vec} * self.distance * k;\n")

for label, spd in [("Slow", 1.0), ("", 3.0), ("Fast", 8.0)]:
    add("animate", f"Drive{label}", "Moves forward (local -Z) at a constant speed in metres per second.", [("speed", "f32", f(spd))], "        ctx.translate_local(Vec3::NEG_Z * self.speed * ctx.dt());\n")

for label, amp, spd, deg in [("Float", 0.15, 1.5, 20), ("Hover", 0.4, 1.0, 40), ("Levitate", 0.8, 0.6, 10), ("Idle", 0.05, 2.5, 60)]:
    add("animate", f"Hover{label}", "Bobs up and down while slowly turning: pickups, crystals, floating signs.",
        [("amplitude", "f32", f(amp)), ("speed", "f32", f(spd)), ("degrees_per_second", "f32", f(deg)), ("origin_y", "Local<Option<f32>>", "Local(None)")],
        "        let y0 = *self.origin_y.0.get_or_insert(ctx.transform().position.y);\n        let t = ctx.elapsed();\n        let dt = ctx.dt();\n        let tr = ctx.transform_mut();\n        tr.position.y = y0 + (t * self.speed).sin() * self.amplitude;\n        tr.rotation.y += self.degrees_per_second * dt;\n")

for label, r, spd in [("Close", 2.0, 60), ("", 4.0, 40), ("Wide", 10.0, 20), ("Fast", 3.0, 120), ("Slow", 6.0, 10), ("Tiny", 0.8, 90)]:
    add("animate", f"OrbitCentre{label}", "Circles around the point where the object started (or an object named in `center`).",
        [("radius", "f32", f(r)), ("degrees_per_second", "f32", f(spd)), ("center", "String", '"".into()'), ("start", "Local<Option<Vec3>>", "Local(None)")],
        "        let start = *self.start.0.get_or_insert(ctx.position());\n        let center = ctx.find(&self.center).and_then(|e| ctx.position_of(e)).unwrap_or(start);\n        let angle = (ctx.elapsed() * self.degrees_per_second).to_radians();\n        let at = center + Vec3::new(angle.cos(), 0.0, angle.sin()) * self.radius + Vec3::Y * (start.y - center.y);\n        ctx.set_position(at);\n")

for label, hue in [("Slow", 0.05), ("", 0.15), ("Fast", 0.6), ("Disco", 2.0)]:
    add("animate", f"ColorCycle{label}", "Cycles the object colour through the rainbow.", [("cycles_per_second", "f32", f(hue)), ("saturation", "f32", "0.8"), ("brightness", "f32", "1.0")],
        "        let me = ctx.entity;\n        let h = (ctx.elapsed() * self.cycles_per_second).fract();\n        let c = hsv_to_rgb(h, self.saturation, self.brightness);\n        ctx.set_color(me, [c[0], c[1], c[2], 1.0]);\n")

for label, colour, hz in [("Red", (1.0, 0.1, 0.1), 2.0), ("Warning", (1.0, 0.6, 0.0), 4.0), ("Alarm", (1.0, 0.0, 0.0), 8.0), ("Cyan", (0.1, 0.9, 1.0), 1.0), ("Green", (0.2, 1.0, 0.3), 1.5)]:
    add("animate", f"Blink{label}", "Blinks between the original colour and a flash colour.", [("flash_r", "f32", f(colour[0])), ("flash_g", "f32", f(colour[1])), ("flash_b", "f32", f(colour[2])), ("hertz", "f32", f(hz)), ("original", "Local<Option<[f32; 4]>>", "Local(None)")],
        "        let me = ctx.entity;\n        let base = *self.original.0.get_or_insert_with(|| ctx.get::<MeshRenderer>().map_or([1.0; 4], |m| m.color));\n        let on = (ctx.elapsed() * self.hertz).fract() < 0.5;\n        let c = if on { [self.flash_r, self.flash_g, self.flash_b, 1.0] } else { base };\n        ctx.set_color(me, c);\n")

for label, vis in [("Slow", 1.0), ("", 0.5), ("Fast", 0.15)]:
    add("animate", f"Flicker{label}", "Switches the object on and off irregularly like a broken lamp.", [("interval", "f32", f(vis)), ("chance_on", "f32", "0.6")],
        "        let me = ctx.entity;\n        if ctx.every(self.interval) {\n            let on = ctx.chance(self.chance_on);\n            ctx.set_visible(me, on);\n        }\n")

# ------------------------------------------------------------------ player and key control
KEYSETS = {"Wasd": ("W", "S", "A", "D"), "Arrows": ("ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight"), "Ijkl": ("I", "K", "J", "L")}
for kname, (up, down, left, right) in KEYSETS.items():
    for label, spd in [("Walk", 3.0), ("Run", 7.0), ("Sprint", 12.0)]:
        add("control", f"Move{kname}{label}", f"Moves the object on the ground with the {kname.upper()} keys ({spd} m/s). Not physics based.", [("speed", "f32", f(spd))],
            f"        let mut dir = Vec3::ZERO;\n        if ctx.key_down(Key::{up}) {{ dir.z -= 1.0; }}\n        if ctx.key_down(Key::{down}) {{ dir.z += 1.0; }}\n        if ctx.key_down(Key::{left}) {{ dir.x -= 1.0; }}\n        if ctx.key_down(Key::{right}) {{ dir.x += 1.0; }}\n        let step = dir.normalize_or_zero() * self.speed * ctx.dt();\n        ctx.translate(step);\n")
    add("control", f"Turn{kname}", f"Turns left and right with the {kname.upper()} keys.", [("degrees_per_second", "f32", "120.0")],
        f"        let turn = (ctx.key_down(Key::{left}) as i32 - ctx.key_down(Key::{right}) as i32) as f32;\n        ctx.rotate(Vec3::new(0.0, turn * self.degrees_per_second * ctx.dt(), 0.0));\n")
    add("control", f"Drive{kname}", f"Car-style control with the {kname.upper()} keys: forward / back and steering.", [("speed", "f32", "8.0"), ("steer", "f32", "90.0")],
        f"        let gas = (ctx.key_down(Key::{up}) as i32 - ctx.key_down(Key::{down}) as i32) as f32;\n        let turn = (ctx.key_down(Key::{left}) as i32 - ctx.key_down(Key::{right}) as i32) as f32;\n        ctx.rotate(Vec3::new(0.0, turn * self.steer * ctx.dt() * gas.signum().max(0.3), 0.0));\n        ctx.translate_local(Vec3::NEG_Z * gas * self.speed * ctx.dt());\n")

for label, force in [("Low", 4.0), ("", 6.5), ("High", 10.0), ("Moon", 3.0)]:
    add("control", f"Jump{label}", "Jumps with Space when the body is close to the ground (a physics RigidBody is required).", [("jump_speed", "f32", f(force)), ("ground_distance", "f32", "1.1")],
        "        if !ctx.key_pressed(Key::Space) {\n            return;\n        }\n        let from = ctx.position();\n        let grounded = ctx.raycast(from, Vec3::NEG_Y, self.ground_distance).is_some();\n        if grounded {\n            let mut v = ctx.velocity();\n            v.y = self.jump_speed;\n            ctx.set_velocity(v);\n        }\n")

for label, key in [("E", "E"), ("F", "F"), ("Space", "Space"), ("Enter", "Enter")]:
    add("control", f"Toggle{label}", f"Shows or hides the object when you press {key}.", [("visible", "Local<bool>", "Local(true)")],
        f"        if ctx.key_pressed(Key::{key}) {{\n            *self.visible = !*self.visible;\n            let me = ctx.entity;\n            let on = *self.visible;\n            ctx.set_visible(me, on);\n        }}\n")

for label, key in [("E", "E"), ("F", "F"), ("Space", "Space"), ("Num1", "Num1"), ("Num2", "Num2")]:
    add("control", f"SoundOn{label}", f"Plays a sound when {key} is pressed.", [("sound", "SoundRef", "SoundRef(String::new())"), ("volume", "f32", "1.0")],
        f"        if ctx.key_pressed(Key::{key}) && !self.sound.0.is_empty() {{\n            let at = ctx.position();\n            ctx.play_sound_at(&self.sound.0, self.volume, at);\n        }}\n")

for label, key in [("E", "E"), ("F", "F"), ("Mouse", "")]:
    cond = f"ctx.key_pressed(Key::{key})" if key else "ctx.mouse_pressed(MouseButton::Left)"
    add("control", f"ShootBall{label}", f"Throws a physics ball forward with {key or 'the left mouse button'}.", [("speed", "f32", "18.0"), ("lifetime", "f32", "6.0")],
        f"        if !{cond} {{\n            return;\n        }}\n        let dir = ctx.forward();\n        let at = ctx.position() + dir * 1.2;\n        let ball = ctx.spawn_primitive(\"Ball\", \"sphere\", at);\n        ctx.scene.world.insert(ball, RigidBody::default());\n        ctx.scene.world.insert(ball, Collider::default());\n        ctx.set_velocity_of(ball, dir * self.speed);\n        ctx.despawn_after(ball, self.lifetime);\n")

# ------------------------------------------------------------------ gameplay
for label, secs in [("1s", 1), ("2s", 2), ("3s", 3), ("5s", 5), ("10s", 10), ("30s", 30)]:
    add("gameplay", f"DespawnAfter{label}", f"Removes the object {secs} seconds after the game starts.", [("seconds", "f32", f(secs)), ("armed", "Local<bool>", "Local(false)")],
        "        if !*self.armed {\n            *self.armed = true;\n            let me = ctx.entity;\n            ctx.despawn_after(me, self.seconds);\n        }\n")

SHAPES = ["cube", "sphere", "cylinder", "capsule", "cone"]
for shape in SHAPES:
    for label, interval, count in [("Slow", 2.0, 30), ("Fast", 0.4, 60)]:
        add("gameplay", f"Spawn{shape.capitalize()}{label}", f"Drops a physics {shape} every {interval} s above the object (at most `max_alive` at a time).",
            [("interval", "f32", f(interval)), ("max_alive", "i32", str(count)), ("height", "f32", "4.0"), ("spread", "f32", "2.0"), ("alive", "Local<Vec<Entity>>", "Local(Vec::new())")],
            f"        self.alive.retain(|e| ctx.is_alive(*e));\n        if !ctx.every(self.interval) || self.alive.len() as i32 >= self.max_alive {{\n            return;\n        }}\n        let at = ctx.position() + Vec3::new(ctx.random_range(-self.spread, self.spread), self.height, ctx.random_range(-self.spread, self.spread));\n        let e = ctx.spawn_primitive(\"{shape}\", \"{shape}\", at);\n        ctx.scene.world.insert(e, RigidBody::default());\n        ctx.scene.world.insert(e, Collider::default());\n        self.alive.push(e);\n")

for label, points, radius in [("Coin", 1, 1.2), ("Gem", 10, 1.0), ("Star", 5, 1.5), ("Crystal", 25, 1.0), ("Orb", 2, 1.4)]:
    add("gameplay", f"Pickup{label}", f"A pickup worth {points} point(s): spins, and is collected by the object named Player. Adds to the shared variable `score`.",
        [("points", "f32", f(points)), ("radius", "f32", f(radius)), ("spin", "f32", "120.0"), ("sound", "SoundRef", "SoundRef(String::new())")],
        "        ctx.rotate(Vec3::new(0.0, self.spin * ctx.dt(), 0.0));\n        let Some(player) = ctx.find(\"Player\") else { return };\n        if ctx.distance_to(player).is_some_and(|d| d < self.radius) {\n            ctx.add_global(\"score\", self.points);\n            if !self.sound.0.is_empty() {\n                let at = ctx.position();\n                ctx.play_sound_at(&self.sound.0, 1.0, at);\n            }\n            ctx.despawn_self();\n        }\n")

for label, key, amount in [("Health", "health", 25), ("Ammo", "ammo", 10), ("Coins", "coins", 5), ("Lives", "lives", 1), ("Energy", "energy", 50)]:
    add("gameplay", f"Pickup{label}", f"Adds {amount} to the shared variable `{key}` when the Player touches it.", [("amount", "f32", f(amount)), ("radius", "f32", "1.3")],
        f"        let Some(player) = ctx.find(\"Player\") else {{ return }};\n        if ctx.distance_to(player).is_some_and(|d| d < self.radius) {{\n            ctx.add_global(\"{key}\", self.amount);\n            ctx.hud_message(&format!(\"+{{}} {key}\", self.amount), 1.0);\n            ctx.despawn_self();\n        }}\n")

for label, radius in [("Near", 2.0), ("", 4.0), ("Far", 8.0), ("Wide", 15.0)]:
    add("gameplay", f"ProximitySignal{label}", "Sends a signal once when the Player comes close.", [("channel", "String", '"proximity".into()'), ("radius", "f32", f(radius)), ("fired", "Local<bool>", "Local(false)")],
        "        if *self.fired {\n            return;\n        }\n        let Some(player) = ctx.find(\"Player\") else { return };\n        if ctx.distance_to(player).is_some_and(|d| d < self.radius) {\n            *self.fired = true;\n            let _ = ctx.emit(&self.channel, json!({ \"from\": \"proximity\" }));\n        }\n")

for axis, vec in AXES.items():
    for label, dist in [("Short", 2.0), ("", 3.5)]:
        add("gameplay", f"SlideDoor{axis}{label}", f"Slides {dist} m along {axis} when it receives the signal `open` and back on `close`.", [("channel_open", "String", '"open".into()'), ("channel_close", "String", '"close".into()'), ("distance", "f32", f(dist)), ("speed", "f32", "3.0"), ("origin", "Local<Option<Vec3>>", "Local(None)"), ("open", "Local<bool>", "Local(false)")],
            f"        let origin = *self.origin.0.get_or_insert(ctx.transform().position);\n        if !ctx.signals(&self.channel_open).is_empty() {{\n            *self.open = true;\n        }}\n        if !ctx.signals(&self.channel_close).is_empty() {{\n            *self.open = false;\n        }}\n        let goal = if *self.open {{ origin + {vec} * self.distance }} else {{ origin }};\n        let here = ctx.transform().position;\n        ctx.transform_mut().position = here + (goal - here).clamp_length_max(self.speed * ctx.dt());\n")

for label, key in [("Click", "E"), ("Enter", "Enter")]:
    add("gameplay", f"Button{label}", f"Sends the signal `open` when the Player is near and presses {key}.", [("channel", "String", '"open".into()'), ("radius", "f32", "2.0")],
        f"        let Some(player) = ctx.find(\"Player\") else {{ return }};\n        if ctx.key_pressed(Key::{key}) && ctx.distance_to(player).is_some_and(|d| d < self.radius) {{\n            let _ = ctx.emit(&self.channel, json!({{}}));\n        }}\n")

for label, below in [("Floor", -20.0), ("Deep", -100.0), ("Shallow", -5.0)]:
    add("gameplay", f"KillZone{label}", f"Teleports the object back to where it started when it falls below y = {below}.", [("min_y", "f32", f(below)), ("start", "Local<Option<Vec3>>", "Local(None)")],
        "        let start = *self.start.0.get_or_insert(ctx.position());\n        if ctx.position().y < self.min_y {\n            ctx.teleport(start, true);\n        }\n")

for label, key in [("Score", "score"), ("Clicks", "clicks"), ("Kills", "kills"), ("Deaths", "deaths")]:
    add("gameplay", f"Count{label}", f"Increments the shared variable `{key}` each time Space is pressed (a test counter).", [("step", "f32", "1.0")],
        f"        if ctx.key_pressed(Key::Space) {{\n            let value = ctx.add_global(\"{key}\", self.step);\n            ctx.hud_message(&format!(\"{key}: {{value}}\"), 1.0);\n        }}\n")

for label, text, secs in [("Welcome", "Welcome!", 3.0), ("Objective", "Find the exit", 5.0), ("Hint", "Press E to interact", 4.0), ("Warning", "Careful!", 2.0), ("Win", "You win!", 6.0)]:
    add("gameplay", f"Message{label}", f"Shows the message \"{text}\" once when the game starts.", [("text", "String", f'"{text}".into()'), ("seconds", "f32", f(secs)), ("shown", "Local<bool>", "Local(false)")],
        "        if !*self.shown {\n            *self.shown = true;\n            ctx.hud_message(&self.text, self.seconds);\n        }\n")

for label, secs in [("Second", 1.0), ("Five", 5.0), ("Ten", 10.0), ("Minute", 60.0)]:
    add("gameplay", f"Heartbeat{label}", f"Broadcasts the signal `tick_{int(secs)}` every {secs} s.", [("interval", "f32", f(secs))],
        f"        if ctx.every(self.interval) {{\n            let _ = ctx.emit(\"tick_{int(secs)}\", json!({{ \"elapsed\": ctx.elapsed() }}));\n        }}\n")

for label, target in [("Player", "Player"), ("Target", "Target"), ("Camera", "Main Camera")]:
    add("gameplay", f"Teleport{label}", f"Moves the object to the object named {target} when T is pressed.", [("target", "String", f'"{target}".into()')],
        "        if ctx.key_pressed(Key::T) {\n            if let Some(p) = ctx.find(&self.target).and_then(|e| ctx.position_of(e)) {\n                ctx.teleport(p, true);\n            }\n        }\n")

for label, secs in [("Fast", 1.0), ("", 3.0), ("Slow", 8.0)]:
    add("gameplay", f"Respawn{label}", f"Puts the object back at its start after {secs} s without moving (stuck objects).", [("seconds", "f32", f(secs)), ("start", "Local<Option<Vec3>>", "Local(None)"), ("last", "Local<Option<Vec3>>", "Local(None)"), ("still", "Local<f32>", "Local(0.0)")],
        "        let start = *self.start.0.get_or_insert(ctx.position());\n        let here = ctx.position();\n        let moved = self.last.0.is_none_or(|p| p.distance(here) > 0.01);\n        *self.last.0.get_or_insert(here) = here;\n        *self.still = if moved { 0.0 } else { *self.still + ctx.dt() };\n        if *self.still > self.seconds {\n            ctx.teleport(start, true);\n            *self.still = 0.0;\n        }\n")

# ------------------------------------------------------------------ camera
for label, off in [("Behind", (0.0, 3.0, 7.0)), ("High", (0.0, 9.0, 6.0)), ("TopDown", (0.0, 14.0, 0.5)), ("Side", (9.0, 2.5, 0.0)), ("Close", (0.0, 1.8, 3.5)), ("Shoulder", (1.2, 1.9, 3.0))]:
    for sm_label, k in [("Smooth", 5.0), ("Tight", 14.0)]:
        add("camera", f"Follow{label}{sm_label}", f"Follows the object named `target` with an offset {off}, smoothing {k}.",
            [("target", "String", '"Player".into()'), ("offset", "Vec3", f"Vec3::new({f(off[0])}, {f(off[1])}, {f(off[2])})"), ("smoothing", "f32", f(k))],
            "        let Some(player) = ctx.find(&self.target).and_then(|e| ctx.position_of(e)) else { return };\n        let wanted = player + self.offset;\n        let here = ctx.position();\n        let next = here.lerp(wanted, 1.0 - (-self.smoothing * ctx.dt()).exp());\n        ctx.set_position(next);\n        ctx.look_at(player);\n")

for label, power in [("Soft", 0.05), ("", 0.15), ("Hard", 0.4), ("Quake", 1.0)]:
    add("camera", f"ShakeOnKey{label}", "Shakes the camera for a moment when you press C (impact feeling).", [("strength", "f32", f(power)), ("duration", "f32", "0.4"), ("time_left", "Local<f32>", "Local(0.0)"), ("origin", "Local<Option<Vec3>>", "Local(None)")],
        "        let origin = *self.origin.0.get_or_insert(ctx.transform().position);\n        if ctx.key_pressed(Key::C) {\n            *self.time_left = self.duration;\n        }\n        if *self.time_left > 0.0 {\n            *self.time_left -= ctx.dt();\n            let k = (*self.time_left / self.duration).clamp(0.0, 1.0) * self.strength;\n            let jitter = Vec3::new(ctx.random_range(-k, k), ctx.random_range(-k, k), 0.0);\n            ctx.transform_mut().position = origin + jitter;\n        } else {\n            ctx.transform_mut().position = origin;\n        }\n")

for label, fov_lo, fov_hi in [("Sprint", 60.0, 75.0), ("Cinematic", 35.0, 50.0), ("Wide", 70.0, 100.0)]:
    add("camera", f"FovBreath{label}", "Slowly changes the field of view between two values.", [("min_fov", "f32", f(fov_lo)), ("max_fov", "f32", f(fov_hi)), ("speed", "f32", "0.5")],
        "        let k = 0.5 + 0.5 * (ctx.elapsed() * self.speed).sin();\n        let fov = self.min_fov + (self.max_fov - self.min_fov) * k;\n        if let Some(c) = ctx.get_mut::<Camera>() {\n            c.fov = fov;\n        }\n")

for label, deg in [("Slow", 6), ("", 15), ("Fast", 45)]:
    add("camera", f"CameraPan{label}", "Turns the camera around the vertical axis for trailers and menus.", [("degrees_per_second", "f32", f(deg))], "        ctx.rotate(Vec3::new(0.0, self.degrees_per_second * ctx.dt(), 0.0));\n")

# ------------------------------------------------------------------ world
for label, speed in [("Slow", 0.05), ("", 0.2), ("Fast", 1.0), ("Timelapse", 4.0), ("Frozen", 0.0)]:
    add("world", f"DayNight{label}", f"Sets the day length: {speed} game hours per second.", [("hours_per_second", "f32", f(speed)), ("start_hour", "f32", "9.0"), ("done", "Local<bool>", "Local(false)")],
        "        if !*self.done {\n            *self.done = true;\n            ctx.set_time_of_day(self.start_hour);\n            ctx.set_day_speed(self.hours_per_second);\n        }\n")

for label, hour in [("Dawn", 6.0), ("Noon", 12.0), ("Dusk", 18.5), ("Midnight", 0.0), ("Morning", 9.0), ("Evening", 20.0)]:
    add("world", f"SetTime{label}", f"Sets the time of day to {hour} h and freezes it.", [("hour", "f32", f(hour)), ("done", "Local<bool>", "Local(false)")],
        "        if !*self.done {\n            *self.done = true;\n            ctx.set_time_of_day(self.hour);\n            ctx.set_day_speed(0.0);\n        }\n")

for label, amount in [("Clear", 0.0), ("Few", 0.25), ("Cloudy", 0.6), ("Overcast", 1.0)]:
    add("world", f"Clouds{label}", f"Sets the cloud cover to {amount}.", [("amount", "f32", f(amount)), ("done", "Local<bool>", "Local(false)")],
        "        if !*self.done {\n            *self.done = true;\n            ctx.set_clouds(self.amount);\n        }\n")

for label, preset in [("Cinematic", "cinematic"), ("Dreamy", "dreamy"), ("Crisp", "crisp"), ("Retro", "retro"), ("Plain", "none")]:
    add("world", f"Look{label}", f"Applies the `{preset}` post-processing look when the game starts.", [("done", "Local<bool>", "Local(false)")],
        f"        if !*self.done {{\n            *self.done = true;\n            ctx.post_preset(\"{preset}\");\n        }}\n")

for label, ex in [("Dark", 0.5), ("Bright", 1.5), ("Blinding", 3.0)]:
    add("world", f"Exposure{label}", f"Sets the exposure to {ex}.", [("exposure", "f32", f(ex)), ("done", "Local<bool>", "Local(false)")],
        "        if !*self.done {\n            *self.done = true;\n            ctx.set_exposure(self.exposure);\n        }\n")

for label, vol in [("Quiet", 0.2), ("Half", 0.5), ("Full", 1.0)]:
    add("audio", f"MasterVolume{label}", f"Sets the master volume to {vol}.", [("volume", "f32", f(vol)), ("done", "Local<bool>", "Local(false)")],
        "        if !*self.done {\n            *self.done = true;\n            ctx.set_master_volume(self.volume);\n        }\n")

for label, p in [("Left", -1.0), ("Centre", 0.0), ("Right", 1.0)]:
    add("audio", f"StereoLoop{label}", f"Plays a looping 2D sound panned to the {label.lower()}.", [("sound", "SoundRef", "SoundRef(String::new())"), ("volume", "f32", "0.6"), ("started", "Local<bool>", "Local(false)")],
        f"        if !*self.started && !self.sound.0.is_empty() {{\n            *self.started = true;\n            ctx.play_sound_panned(&self.sound.0, self.volume, {f(p)});\n        }}\n")

# ------------------------------------------------------------------ debug
for label, secs in [("Second", 1.0), ("Five", 5.0), ("Tenth", 0.1)]:
    add("debug", f"LogPosition{label}", f"Prints the object position every {secs} s.", [("interval", "f32", f(secs))], "        ctx.watch_every(self.interval, \"position\", ctx.position());\n")
for label, secs in [("Second", 1.0), ("Five", 5.0)]:
    add("debug", f"LogFps{label}", f"Prints the frame rate every {secs} s.", [("interval", "f32", f(secs))], "        let fps = ctx.fps();\n        ctx.watch_every(self.interval, \"fps\", fps);\n")
add("debug", "LogTags", "Prints the tags of the object once at start.", [("done", "Local<bool>", "Local(false)")],
    "        if !*self.done {\n            *self.done = true;\n            let tags = ctx.tags_of(ctx.entity);\n            ctx.log(format!(\"tags: {tags:?}\"));\n        }\n")
add("debug", "LogGlobals", "Prints the shared variable `score` every 2 s.", [("key", "String", '"score".into()')],
    "        let value = ctx.global_f32(&self.key, 0.0);\n        ctx.watch_every(2.0, &self.key, value);\n")


def main():
    if os.path.isdir(ROOT):
        shutil.rmtree(ROOT)
    for rel, text in FILES.items():
        path = os.path.join(ROOT, rel)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w", encoding="utf-8", newline="\n") as fh:
            fh.write(text)
    print(f"{len(FILES)} scripts -> {os.path.normpath(ROOT)}")


main()
