// First-person gameplay scripts (included by `scripts.rs`): walking, jumping, picking things up,
// throwing them, pressing buttons, a flashlight and a PSX-look toggle.

fn look_direction(yaw: f32, pitch: f32) -> Vec3 {
    let (y, p) = (yaw.to_radians(), pitch.to_radians());
    Vec3::new(-y.sin() * p.cos(), p.sin(), -y.cos() * p.cos())
}

/// Nearest entity at or above `e` (the hit collider's owner or its parents) that has component `T`.
fn owner_with<T: Component>(world: &World, mut e: Entity) -> Option<Entity> {
    loop {
        if world.has::<T>(e) {
            return Some(e);
        }
        e = hierarchy::parent_of(world, e)?;
    }
}

/// Walk (WASD), sprint (Shift), jump (Space), look with the mouse, carry things (E), throw them
/// (left click) and use buttons (E). Put it on a capsule entity with a RigidBody that has
/// `lock_rotation`; the camera goes on a child called "Eyes".
pub struct FpsPlayer {
    walk_speed: f32,
    sprint_speed: f32,
    jump_speed: f32,
    sensitivity: f32,
    reach: f32,
    max_carry_mass: f32,
    throw_speed: f32,
    objective: String,
    yaw: f32,
    pitch: f32,
    ready: bool,
    wish: Vec3,
    sprinting: bool,
    jump_buffer: f32,
    grounded: bool,
    held: Option<Entity>,
    hold_distance: f32,
    eyes: Option<Entity>,
    flashlight: Option<(Entity, f32)>,
    flashlight_on: bool,
    start: Vec3,
}

impl Default for FpsPlayer {
    fn default() -> Self {
        Self {
            walk_speed: 4.6,
            sprint_speed: 7.4,
            jump_speed: 5.4,
            sensitivity: 0.11,
            reach: 3.3,
            max_carry_mass: 60.0,
            throw_speed: 13.0,
            objective: String::new(),
            yaw: 0.0,
            pitch: 0.0,
            ready: false,
            wish: Vec3::ZERO,
            sprinting: false,
            jump_buffer: 0.0,
            grounded: false,
            held: None,
            hold_distance: 2.0,
            eyes: None,
            flashlight: None,
            flashlight_on: false,
            start: Vec3::ZERO,
        }
    }
}

impl FpsPlayer {
    fn eye_position(&self, ctx: &Ctx) -> Vec3 {
        self.eyes.map_or(ctx.world_position(), |e| hierarchy::global_matrix(&ctx.scene.world, e).w_axis.truncate())
    }

    fn release(&mut self) {
        self.held = None;
    }

    /// Looks for something to interact with; returns the text to show.
    fn interact(&mut self, ctx: &mut Ctx, eye: Vec3, dir: Vec3) {
        let hit = ctx.raycast(eye, dir, self.reach);
        let Some(hit) = hit else { return };
        let use_pressed = ctx.action_pressed("Interact");
        let target = owner_with::<Switch>(&ctx.scene.world, hit.entity);
        if let Some(s) = target {
            let (label, interactive) = {
                let sw = ctx.get_of::<Switch>(s).unwrap();
                (sw.prompt.clone(), sw.kind != SwitchKind::Plate && !sw.prompt.is_empty())
            };
            if interactive {
                ctx.hud_prompt(&format!("[E]  {label}"));
                if use_pressed {
                    press(ctx.world(), s);
                    ctx.hud_message("Click!", 1.2);
                }
            }
            return;
        }
        if let Some(body) = owner_with::<RigidBody>(&ctx.scene.world, hit.entity) {
            let Some(rb) = ctx.get_of::<RigidBody>(body) else { return };
            if rb.kind != BodyKind::Dynamic || body == ctx.entity {
                return;
            }
            if rb.mass > self.max_carry_mass {
                ctx.hud_prompt("Too heavy");
                return;
            }
            ctx.hud_prompt("[E]  Pick up");
            if use_pressed {
                self.held = Some(body);
                self.hold_distance = hit.distance.clamp(1.4, 2.4);
            }
        }
    }
}

impl Script for FpsPlayer {
    fn start(&mut self, ctx: &mut Ctx) {
        self.eyes = ctx.find("Eyes");
        self.start = ctx.world_position();
        if let Some(e) = ctx.find("Flashlight") {
            let base = ctx.get_of::<Light>(e).map_or(1.0, |l| l.intensity);
            self.flashlight = Some((e, base));
        }
        ctx.lock_cursor(true);
    }

    fn update(&mut self, ctx: &mut Ctx) {
        if !self.ready {
            self.ready = true;
            if let Some(eyes) = self.eyes {
                if let Some(t) = ctx.get_of::<Transform>(eyes) {
                    self.yaw = t.rotation.y;
                    self.pitch = t.rotation.x;
                }
            }
        }
        // Mouse capture: Esc frees the cursor, a click takes it back.
        let locked = ctx.input.cursor_locked();
        if locked && ctx.key_pressed(Key::Escape) {
            ctx.lock_cursor(false);
        } else if !locked && ctx.input.mouse_pressed(MouseButton::Left) {
            ctx.lock_cursor(true);
        }
        let look = ctx.input.cursor_locked();
        if look {
            self.yaw -= ctx.input.mouse_delta.x * self.sensitivity;
            self.pitch = (self.pitch - ctx.input.mouse_delta.y * self.sensitivity).clamp(-88.0, 88.0);
        }
        if let Some(eyes) = self.eyes {
            if let Some(t) = ctx.get_mut_of::<Transform>(eyes) {
                t.rotation = Vec3::new(self.pitch, self.yaw, 0.0);
            }
        }

        // Movement wish (applied in fixed_update).
        let (x, z) = (ctx.action_axis("MoveLeft", "MoveRight"), ctx.action_axis("MoveBack", "MoveForward"));
        let yaw = self.yaw.to_radians();
        let forward = Vec3::new(-yaw.sin(), 0.0, -yaw.cos());
        let right = Vec3::new(yaw.cos(), 0.0, -yaw.sin());
        self.wish = (forward * z + right * x).normalize_or_zero();
        self.sprinting = ctx.action_down("Sprint");
        self.jump_buffer = if ctx.action_pressed("Jump") { 0.18 } else { (self.jump_buffer - ctx.dt()).max(0.0) };

        // Look at things.
        ctx.hud_crosshair(true);
        ctx.hud_objective(&self.objective);
        let eye = self.eye_position(ctx);
        let dir = look_direction(self.yaw, self.pitch);
        if let Some(h) = self.held {
            let alive = ctx.scene.world.is_alive(h) && ctx.get_of::<RigidBody>(h).is_some();
            if !alive {
                self.release();
            } else {
                ctx.hud_prompt("[E] Drop     [Click] Throw");
                if ctx.action_pressed("Interact") || ctx.input.mouse_pressed(MouseButton::Right) {
                    self.release();
                } else if ctx.input.mouse_pressed(MouseButton::Left) && look {
                    let v = dir * self.throw_speed + ctx.velocity() * 0.5;
                    ctx.set_velocity_of(h, v);
                    self.release();
                }
            }
        } else if look {
            self.interact(ctx, eye, dir);
        }

        // Flashlight and the PSX look.
        if ctx.key_pressed(Key::F) {
            self.flashlight_on = !self.flashlight_on;
        }
        if let Some((e, base)) = self.flashlight {
            let on = self.flashlight_on;
            if let Some(l) = ctx.get_mut_of::<Light>(e) {
                l.intensity = if on { base } else { 0.0 };
            }
        }
        if ctx.key_pressed(Key::P) {
            let s = &mut ctx.scene.settings;
            s.psx = !s.psx;
            s.render_scale = if s.psx { 0.4 } else { 1.0 };
            let text = if s.psx { "PSX look: on" } else { "PSX look: off" };
            ctx.hud_message(text, 2.0);
        }
        // Fell off the map.
        if ctx.world_position().y < -30.0 {
            let start = self.start;
            ctx.transform_mut().position = start;
            ctx.set_velocity(Vec3::ZERO);
        }
    }

    fn fixed_update(&mut self, ctx: &mut Ctx) {
        // Ground check: a short ray from the capsule centre down to just below the feet.
        let origin = ctx.world_position();
        let ground = ctx.raycast(origin, Vec3::NEG_Y, 0.9 + 0.12);
        self.grounded = ground.is_some();
        if let (Some(g), Some(h)) = (&ground, self.held) {
            if g.entity == h {
                self.release(); // standing on the carried object
            }
        }

        let v = ctx.velocity();
        let speed = if self.sprinting { self.sprint_speed } else { self.walk_speed };
        let target = self.wish * speed;
        let k = if self.grounded { 0.35 } else { 0.06 };
        let mut horizontal = Vec3::new(v.x, 0.0, v.z);
        horizontal += (target - horizontal) * k;
        let mut vy = v.y;
        if self.jump_buffer > 0.0 && self.grounded && vy < 1.0 {
            vy = self.jump_speed;
            self.jump_buffer = 0.0;
        }
        ctx.set_velocity(Vec3::new(horizontal.x, vy, horizontal.z));

        // Carry: pull the held body towards a point in front of the eyes.
        if let Some(h) = self.held {
            let eye = self.eye_position(ctx);
            let dir = look_direction(self.yaw, self.pitch.max(-35.0));
            let goal = eye + dir * self.hold_distance;
            let pos = hierarchy::global_matrix(&ctx.scene.world, h).w_axis.truncate();
            let mut pull = (goal - pos) * 14.0;
            if pull.length() > 14.0 {
                pull = pull.normalize() * 14.0;
            }
            ctx.set_velocity_of(h, pull);
            ctx.set_angular_velocity_of(h, Vec3::ZERO);
        }
    }

    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![
            Field::f32("walk_speed", &mut self.walk_speed),
            Field::f32("sprint_speed", &mut self.sprint_speed),
            Field::f32("jump_speed", &mut self.jump_speed),
            Field::f32("sensitivity", &mut self.sensitivity),
            Field::f32("reach", &mut self.reach),
            Field::f32("max_carry_mass", &mut self.max_carry_mass),
            Field::f32("throw_speed", &mut self.throw_speed),
            Field::text("objective", &mut self.objective),
        ]
    }
}
