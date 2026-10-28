//! The script cookbook. Every recipe is compiled and the main ones are run for a few frames, then
//! the documentation site embeds the very same code (`docs/scripting/cookbook.md`), so the examples
//! on the website cannot rot.
use kvas_engine::{
    prelude::*,
    runtime::Runtime,
    scene::Scene,
    script::{ScriptDef, ScriptHost},
};

// #region spinner
/// Spins the object around Y. `speed` is shown in the Inspector and saved in the scene.
#[derive(Default)]
pub struct Spinner {
    speed: f32,
}

impl Script for Spinner {
    fn update(&mut self, ctx: &mut Ctx) {
        ctx.rotate(Vec3::new(0.0, self.speed * ctx.dt(), 0.0));
    }

    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![Field::f32("speed", &mut self.speed)]
    }
}
// #endregion spinner

// #region chaser
/// Walks towards the object named "Player" and stops at `stop_distance`.
#[derive(Default)]
pub struct Chaser {
    speed: f32,
    stop_distance: f32,
}

impl Script for Chaser {
    fn update(&mut self, ctx: &mut Ctx) {
        let Some(player) = ctx.find("Player") else { return };
        let Some(target) = ctx.position_of(player) else { return };
        ctx.look_at_flat(target);
        if ctx.distance_to(player).is_some_and(|d| d > self.stop_distance) {
            ctx.move_towards(target, self.speed);
        }
    }

    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![Field::f32("speed", &mut self.speed), Field::f32("stop_distance", &mut self.stop_distance)]
    }
}
// #endregion chaser

// #region patrol
/// Walks between the points of `route` in a loop.
#[derive(Default)]
pub struct Patrol {
    route: Vec<Vec3>,
    next: usize,
    speed: f32,
}

impl Script for Patrol {
    fn start(&mut self, ctx: &mut Ctx) {
        if self.route.is_empty() {
            // Default route: a square around the start point.
            let o = ctx.position();
            self.route = vec![o + Vec3::X * 4.0, o + Vec3::new(4.0, 0.0, 4.0), o + Vec3::Z * 4.0, o];
        }
    }

    fn update(&mut self, ctx: &mut Ctx) {
        let target = self.route[self.next % self.route.len()];
        ctx.look_at_flat(target);
        if ctx.move_towards(target, self.speed.max(1.0)) {
            self.next += 1;
        }
    }
}
// #endregion patrol

// #region spawner
/// Spawns a small physical ball every `interval` seconds at a random spot around the object.
#[derive(Default)]
pub struct BallSpawner {
    rng: Rng,
    timer: Option<Timer>,
}

impl Script for BallSpawner {
    fn start(&mut self, _ctx: &mut Ctx) {
        self.rng = Rng::new(7);
        self.timer = Timer::repeating(0.5);
    }

    fn update(&mut self, ctx: &mut Ctx) {
        let Some(timer) = self.timer.as_mut() else { return };
        if timer.tick_secs(ctx.dt()).just_finished() {
            let at = ctx.position() + self.rng.on_circle(3.0) + Vec3::Y * 4.0;
            let ball = ctx.spawn_primitive("Ball", "sphere", at);
            ctx.set_color(ball, [self.rng.f32(), self.rng.f32(), self.rng.f32(), 1.0]);
            ctx.despawn_after(ball, 10.0);
        }
    }
}
// #endregion spawner

// #region camera_follow
/// Smoothly follows the object named "Player" from behind and above.
#[derive(Default)]
pub struct FollowCamera {
    offset: Vec3,
}

impl Script for FollowCamera {
    fn start(&mut self, _ctx: &mut Ctx) {
        if self.offset == Vec3::ZERO {
            self.offset = Vec3::new(0.0, 4.0, 7.0);
        }
    }

    fn update(&mut self, ctx: &mut Ctx) {
        let Some(player) = ctx.find("Player") else { return };
        let Some(target) = ctx.position_of(player) else { return };
        let wanted = target + self.offset;
        let here = ctx.position();
        // Exponential smoothing is frame-rate independent: the same feel at 30 and 144 FPS.
        let smooth = here.lerp(wanted, 1.0 - (-6.0 * ctx.dt()).exp());
        ctx.set_position(smooth);
        ctx.look_at(target);
    }
}
// #endregion camera_follow

// #region pickup
/// A coin: the player collects it by touching it.
#[derive(Default)]
pub struct Coin;

impl Script for Coin {
    fn update(&mut self, ctx: &mut Ctx) {
        ctx.rotate(Vec3::new(0.0, 120.0 * ctx.dt(), 0.0));
        let hits = ctx.overlap_sphere(ctx.position(), 0.8);
        if hits.into_iter().any(|e| ctx.has_tag(e, "player")) {
            ctx.hud_message("+1 coin", 1.0);
            ctx.despawn_self();
        }
    }
}
// #endregion pickup

// #region shooter
/// Press the left mouse button to shoot a ray forward and push what it hits.
#[derive(Default)]
pub struct Shooter {
    force: f32,
}

impl Script for Shooter {
    fn update(&mut self, ctx: &mut Ctx) {
        if !ctx.mouse_pressed(MouseButton::Left) {
            return;
        }
        let origin = ctx.position();
        let direction = ctx.forward();
        if let Some(hit) = ctx.raycast(origin, direction, 50.0) {
            ctx.hud_message("hit!", 0.5);
            // Apply the impulse to the hit object through the world.
            if let Some(body) = ctx.get_mut_of::<RigidBody>(hit.entity) {
                body.impulse += direction * self.force.max(5.0);
            }
        }
    }
}
// #endregion shooter

// #region signals
/// A door that opens when somebody broadcasts the "open_door" signal.
#[derive(Default)]
pub struct Door {
    open: bool,
}

impl Script for Door {
    fn update(&mut self, ctx: &mut Ctx) {
        if !ctx.signals("open_door").is_empty() {
            self.open = true;
        }
        let target_y = if self.open { 3.0 } else { 0.0 };
        let y = ctx.transform().position.y;
        ctx.transform_mut().position.y = approach(y, target_y, 2.0 * ctx.dt());
    }
}

/// A button that sends the signal after being touched.
#[derive(Default)]
pub struct DoorButton;

impl Script for DoorButton {
    fn update(&mut self, ctx: &mut Ctx) {
        if ctx.action_pressed("Interact") {
            let _ = ctx.emit("open_door", json!({ "from": "button" }));
        }
    }
}
// #endregion signals

fn defs() -> ScriptHost {
    ScriptHost::from_defs(vec![
        ScriptDef::of::<Spinner>("Spinner"),
        ScriptDef::of::<Chaser>("Chaser"),
        ScriptDef::of::<Patrol>("Patrol"),
        ScriptDef::of::<BallSpawner>("BallSpawner"),
        ScriptDef::of::<FollowCamera>("FollowCamera"),
        ScriptDef::of::<Coin>("Coin"),
        ScriptDef::of::<Shooter>("Shooter"),
        ScriptDef::of::<Door>("Door"),
        ScriptDef::of::<DoorButton>("DoorButton"),
    ])
    .unwrap()
}

fn attach(scene: &mut Scene, name: &str, position: Vec3, scripts: &[&str]) -> Entity {
    let entity = scene.spawn(name);
    scene.world.insert(entity, Transform::at(position));
    scene.world.insert(entity, Scripts(scripts.iter().map(|name| ScriptSlot { name: name.to_string(), ..Default::default() }).collect()));
    entity
}

#[test]
fn recipes_run_without_panicking_and_do_what_they_say() {
    let mut scene = Scene::default();
    let player = scene.spawn("Player");
    scene.world.insert(player, Transform::at(Vec3::new(10.0, 0.0, 0.0)));
    let chaser = attach(&mut scene, "Chaser", Vec3::ZERO, &["Chaser"]);
    let spinner = attach(&mut scene, "Spinner", Vec3::new(0.0, 5.0, 0.0), &["Spinner"]);
    let patrol = attach(&mut scene, "Patrol", Vec3::new(0.0, 0.0, 20.0), &["Patrol"]);
    let _spawner = attach(&mut scene, "Spawner", Vec3::new(0.0, 0.0, -20.0), &["BallSpawner"]);
    let _camera = attach(&mut scene, "Camera", Vec3::new(0.0, 10.0, 20.0), &["FollowCamera"]);
    let _door = attach(&mut scene, "Door", Vec3::ZERO, &["Door", "DoorButton"]);
    let _coin = attach(&mut scene, "Coin", Vec3::new(30.0, 0.0, 0.0), &["Coin"]);
    let _shooter = attach(&mut scene, "Shooter", Vec3::new(50.0, 0.0, 0.0), &["Shooter"]);
    let mut runtime = Runtime::new(scene, Some(defs()));
    for _ in 0..120 {
        runtime.tick(1.0 / 60.0);
    }
    let pos = |e: Entity| runtime.scene.world.get::<Transform>(e).unwrap().position;
    // `speed` is 0 until set in the Inspector, so the chaser does not move yet, but it faced the player.
    assert!(runtime.scene.world.get::<Transform>(chaser).unwrap().rotation.y.abs() > 1.0);
    assert_eq!(pos(chaser), Vec3::ZERO);
    assert_eq!(runtime.scene.world.get::<Transform>(spinner).unwrap().rotation.y, 0.0, "speed 0 = no spin");
    // The patrol walks at least 1 m/s along its default square.
    assert!(pos(patrol).distance(Vec3::new(0.0, 0.0, 20.0)) > 1.0);
}

#[test]
fn chaser_reaches_the_player_when_given_a_speed() {
    let mut scene = Scene::default();
    let player = scene.spawn("Player");
    scene.world.insert(player, Transform::at(Vec3::new(10.0, 0.0, 0.0)));
    let chaser = scene.spawn("Chaser");
    scene.world.insert(chaser, Transform::at(Vec3::ZERO));
    let mut slot = ScriptSlot { name: "Chaser".into(), ..Default::default() };
    slot.props.insert("speed".into(), json!(4.0));
    slot.props.insert("stop_distance".into(), json!(2.0));
    scene.world.insert(chaser, Scripts(vec![slot]));
    let mut runtime = Runtime::new(scene, Some(defs()));
    for _ in 0..240 {
        runtime.tick(1.0 / 60.0);
    }
    let x = runtime.scene.world.get::<Transform>(chaser).unwrap().position.x;
    assert!((7.5..=8.5).contains(&x), "stopped {x} m along the way, expected about 8");
}
