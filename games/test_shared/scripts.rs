// Scripts shared by the test projects (each project's scripts crate `include!`s this file).

use kvas_engine::prelude::*;

/// Free-fly camera: hold the right mouse button to look, WASD / arrows to move (Shift = faster),
/// Space / Ctrl move up and down. Uses the action bindings from Project Settings.
pub struct FirstPerson {
    speed: f32,
    sensitivity: f32,
    yaw: f32,
    pitch: f32,
    ready: bool,
}

impl Default for FirstPerson {
    fn default() -> Self {
        Self { speed: 5.0, sensitivity: 0.18, yaw: 0.0, pitch: 0.0, ready: false }
    }
}

impl Script for FirstPerson {
    fn update(&mut self, ctx: &mut Ctx) {
        if !self.ready {
            let r = ctx.transform().rotation;
            (self.pitch, self.yaw, self.ready) = (r.x, r.y, true);
        }
        if ctx.mouse_down(MouseButton::Right) {
            self.yaw -= ctx.input.mouse_delta.x * self.sensitivity;
            self.pitch = (self.pitch - ctx.input.mouse_delta.y * self.sensitivity).clamp(-85.0, 85.0);
        }
        let boost = if ctx.action_down("Sprint") { 3.0 } else { 1.0 };
        let (x, z) = (ctx.action_axis("MoveLeft", "MoveRight"), ctx.action_axis("MoveBack", "MoveForward"));
        let lift = ctx.key_down(Key::Space) as i32 as f32 - ctx.key_down(Key::Ctrl) as i32 as f32;
        let dt = ctx.dt();
        let speed = self.speed * boost;
        let t = ctx.transform_mut();
        t.rotation = Vec3::new(self.pitch, self.yaw, 0.0);
        let (forward, right) = (t.forward(), t.right());
        t.position += (forward * z + right * x + Vec3::Y * lift) * speed * dt;
    }

    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![Field::f32("speed", &mut self.speed), Field::f32("sensitivity", &mut self.sensitivity)]
    }
}

/// Circles a point and keeps looking at it. Good for unattended demos and screenshots.
pub struct OrbitCamera {
    center: Vec3,
    radius: f32,
    height: f32,
    speed: f32,
    angle: f32,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self { center: Vec3::new(0.0, 2.0, 0.0), radius: 14.0, height: 5.0, speed: 8.0, angle: 0.0 }
    }
}

impl Script for OrbitCamera {
    fn update(&mut self, ctx: &mut Ctx) {
        self.angle += self.speed.to_radians() * ctx.dt();
        let pos = self.center + Vec3::new(self.angle.cos() * self.radius, self.height, self.angle.sin() * self.radius);
        let d = self.center - pos;
        let t = ctx.transform_mut();
        t.position = pos;
        t.rotation = Vec3::new((d.y / d.length()).asin().to_degrees(), (-d.x).atan2(-d.z).to_degrees(), 0.0);
    }

    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![Field::vec3("center", &mut self.center), Field::f32("radius", &mut self.radius), Field::f32("height", &mut self.height), Field::f32("speed", &mut self.speed)]
    }
}

/// Varies the wind of every cloth in the scene over time (slow swell plus quick flutter).
pub struct GustController {
    base: Vec3,
    swell: f32,
}

impl Default for GustController {
    fn default() -> Self {
        Self { base: Vec3::new(3.0, 0.0, 1.5), swell: 0.7 }
    }
}

impl Script for GustController {
    fn update(&mut self, ctx: &mut Ctx) {
        let t = ctx.elapsed();
        let k = 1.0 + self.swell * (t * 0.45).sin() + 0.25 * (t * 2.7).sin();
        let wind = self.base * k.max(0.0);
        for (_, cloth) in ctx.world().iter_mut::<Cloth>() {
            cloth.wind = wind;
        }
    }

    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![Field::vec3("base", &mut self.base), Field::f32("swell", &mut self.swell)]
    }
}

/// Press E to drop a physics ball from above this object (it pushes cloth around as it falls).
#[derive(Default)]
pub struct BallDropper {
    count: i32,
}

impl Script for BallDropper {
    fn update(&mut self, ctx: &mut Ctx) {
        if !ctx.action_pressed("Interact") {
            return;
        }
        self.count += 1;
        let at = ctx.world_position() + Vec3::new(((self.count * 37) % 11) as f32 * 0.12 - 0.6, 0.0, ((self.count * 53) % 7) as f32 * 0.1 - 0.3);
        let name = format!("Ball {}", self.count);
        let e = ctx.spawn_primitive(&name, "sphere", at);
        let world = ctx.world();
        world.insert(e, RigidBody { mass: 2.0, ..Default::default() });
        world.insert(e, Collider { shape: ShapeKind::Sphere, size: Vec3::ONE, restitution: 0.3, ..Default::default() });
        if let Some(t) = world.get_mut::<Transform>(e) {
            t.scale = Vec3::splat(0.5);
        }
    }
}

/// Flickers a point light like a flame.
#[derive(Default)]
pub struct Flicker {
    base: f32,
}

impl Script for Flicker {
    fn start(&mut self, ctx: &mut Ctx) {
        self.base = ctx.get::<Light>().map_or(1.0, |l| l.intensity);
    }
    fn update(&mut self, ctx: &mut Ctx) {
        let t = ctx.elapsed();
        let noise = Noise::new(NoiseKind::Perlin).with_octaves(2).sample2(t * 3.0, 4.0);
        let base = self.base;
        if let Some(l) = ctx.get_mut::<Light>() {
            l.intensity = base * (1.0 + noise * 0.35);
        }
    }
}

include!("fps.rs");

register_scripts!(FirstPerson, OrbitCamera, GustController, BallDropper, Flicker, FpsPlayer);
