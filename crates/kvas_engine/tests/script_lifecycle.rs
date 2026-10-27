//! Host-linked scripts exercise lifecycle failure paths without a native DLL build.
use kvas_engine::{
    prelude::*,
    runtime::Runtime,
    scene::Scene,
    script::{ScriptDef, ScriptHost},
};

#[derive(Default)]
struct Healthy;
impl Script for Healthy {
    fn start(&mut self, ctx: &mut Ctx) {
        ctx.log("healthy started");
    }
    fn update(&mut self, ctx: &mut Ctx) {
        ctx.transform_mut().position.x += 1.0;
    }
}
#[derive(Default)]
struct BrokenCallback;
impl Script for BrokenCallback {
    fn update(&mut self, _: &mut Ctx) {
        panic!("callback failure");
    }
}
struct BrokenFactory;
impl Default for BrokenFactory {
    fn default() -> Self {
        panic!("factory failure");
    }
}
impl Script for BrokenFactory {}
#[derive(Default)]
struct BrokenFields;
impl Script for BrokenFields {
    fn fields(&mut self) -> Vec<Field<'_>> {
        panic!("fields failure");
    }
}
#[derive(Default)]
struct SelfDespawn;
impl Script for SelfDespawn {
    fn start(&mut self, ctx: &mut Ctx) {
        ctx.despawn(ctx.entity);
    }
    fn update(&mut self, _: &mut Ctx) {
        panic!("removed scripts must not update");
    }
}

fn attach(scene: &mut Scene, name: &str, scripts: &[&str]) -> Entity {
    let entity = scene.spawn(name);
    scene.world.insert(entity, Scripts(scripts.iter().map(|name| ScriptSlot { name: name.to_string(), ..Default::default() }).collect()));
    entity
}

#[test]
fn failures_are_isolated_and_initialization_errors_do_not_spam_each_frame() {
    let host = ScriptHost::from_defs(vec![
        ScriptDef::of::<Healthy>("Healthy"),
        ScriptDef::of::<BrokenCallback>("Callback"),
        ScriptDef::of::<BrokenFactory>("Factory"),
        ScriptDef::of::<BrokenFields>("Fields"),
    ])
    .unwrap();
    let mut scene = Scene::default();
    let entity = attach(&mut scene, "entity", &["Factory", "Fields", "Callback", "Missing", "Healthy"]);
    let mut runtime = Runtime::new(scene, Some(host));
    for _ in 0..5 {
        runtime.tick(1.0 / 60.0);
    }
    assert_eq!(runtime.scene.world.get::<Transform>(entity).unwrap().position.x, 5.0);
    assert_eq!(runtime.scripts.instance_count(), 2);
    assert_eq!(runtime.scripts.failed_count(), 4);
    assert_eq!(runtime.log.iter().filter(|s| s.contains("initialization panicked")).count(), 2);
    assert_eq!(runtime.log.iter().filter(|s| s.contains("not found")).count(), 1);
    assert_eq!(runtime.log.iter().filter(|s| s.contains("callback failure")).count(), 1);
    assert_eq!(runtime.log.iter().filter(|s| s.as_str() == "healthy started").count(), 1);
    runtime.scripts.restart_instances();
    runtime.tick(1.0 / 60.0);
    assert_eq!(runtime.log.iter().filter(|s| s.as_str() == "healthy started").count(), 2);
}

#[test]
fn self_despawn_during_start_suppresses_update_and_removed_slots_are_recreated() {
    let host = ScriptHost::from_defs(vec![ScriptDef::of::<Healthy>("Healthy"), ScriptDef::of::<SelfDespawn>("SelfDespawn")]).unwrap();
    let mut scene = Scene::default();
    let removed = attach(&mut scene, "removed", &["SelfDespawn"]);
    let healthy = attach(&mut scene, "healthy", &["Healthy"]);
    let mut runtime = Runtime::new(scene, Some(host));
    runtime.tick(0.01);
    assert!(!runtime.scene.world.is_alive(removed));
    assert_eq!(runtime.scripts.failed_count(), 0);
    assert!(!runtime.log.iter().any(|s| s.contains("panicked")));
    runtime.scene.world.get_mut::<Scripts>(healthy).unwrap().0[0].name = "SelfDespawn".into();
    runtime.tick(0.01);
    assert!(!runtime.scene.world.is_alive(healthy));
}

#[test]
fn host_rejects_duplicate_and_empty_names() {
    assert!(ScriptHost::from_defs(vec![ScriptDef::of::<Healthy>("")]).is_err());
    assert!(ScriptHost::from_defs(vec![ScriptDef::of::<Healthy>("same"), ScriptDef::of::<Healthy>("same")]).is_err());
    assert!(ScriptHost::from_defs(Vec::new()).unwrap().names().is_empty());
}

#[derive(Default)]
struct ApiExercise;
impl Script for ApiExercise {
    fn start(&mut self, ctx: &mut Ctx) {
        assert!(ctx.add_tag(ctx.entity, "actor"));
        assert!(ctx.has_tag(ctx.entity, "actor"));
        assert_eq!(ctx.tagged("actor"), vec![ctx.entity]);
        let child = ctx.spawn_primitive("animated", "cube", Vec3::ZERO);
        assert!(ctx.set_parent_local(child, Some(ctx.entity)));
        assert!(ctx.move_to(child, Vec3::Y, 1.0, Ease::Linear));
        assert!(ctx.set_visible(ctx.entity, false));
        assert!(!ctx.is_visible(child));
        let temporary = ctx.spawn("temporary");
        assert!(ctx.despawn_after(temporary, 0.02));
        assert!(!ctx.despawn_after(temporary, f32::NAN));
        assert!(ctx.teleport(Vec3::X * 10.0, true));
    }
}

#[test]
fn ctx_gameplay_helpers_work_in_the_actual_runtime() {
    let host = ScriptHost::from_defs(vec![ScriptDef::of::<ApiExercise>("ApiExercise")]).unwrap();
    let mut scene = Scene::default();
    let owner = attach(&mut scene, "owner", &["ApiExercise"]);
    scene.world.insert(owner, RigidBody { gravity_scale: 0.0, ..Default::default() });
    let mut runtime = Runtime::new(scene, Some(host));
    runtime.tick(0.03);
    assert!(runtime.log.iter().all(|s| !s.contains("panicked")), "{:?}", runtime.log);
    assert_eq!(runtime.scene.world.get::<Transform>(owner).unwrap().position.x, 10.0);
    assert!(runtime.scene.find("temporary").is_none());
    let animated = runtime.scene.find("animated").unwrap();
    assert!((runtime.scene.world.get::<Transform>(animated).unwrap().position.y - 0.03).abs() < 1e-6);
}

#[derive(Default)]
struct TeleportSecondFrame;
impl Script for TeleportSecondFrame {
    fn update(&mut self, ctx: &mut Ctx) {
        if ctx.time.frame == 2 {
            assert!(ctx.teleport(Vec3::new(12.0, 3.0, 0.0), true));
        }
    }
}

#[test]
fn teleport_moves_an_existing_dynamic_body_and_resets_momentum() {
    let host = ScriptHost::from_defs(vec![ScriptDef::of::<TeleportSecondFrame>("Teleport")]).unwrap();
    let mut scene = Scene::default();
    let entity = attach(&mut scene, "body", &["Teleport"]);
    scene.world.insert(entity, RigidBody { gravity_scale: 0.0, set_velocity: Some(Vec3::X * 30.0), ..Default::default() });
    scene.world.insert(entity, Collider::default());
    let mut runtime = Runtime::new(scene, Some(host));
    runtime.tick(1.0 / 60.0);
    runtime.tick(1.0 / 60.0);
    let position = runtime.scene.world.get::<Transform>(entity).unwrap().position;
    assert!((position - Vec3::new(12.0, 3.0, 0.0)).length() < 1e-5, "{position}");
    assert_eq!(runtime.scene.world.get::<RigidBody>(entity).unwrap().velocity, Vec3::ZERO);
}

#[derive(Default)]
struct Sender;
impl Script for Sender {
    fn update(&mut self, ctx: &mut Ctx) {
        if ctx.time.frame == 1 {
            ctx.emit("score", serde_json::json!(1)).unwrap();
            let recipient = ctx.find("receiver-a").unwrap();
            ctx.send_to(recipient, "score", serde_json::json!(2)).unwrap();
        }
    }
}
#[derive(Default)]
struct Receiver;
impl Script for Receiver {
    fn update(&mut self, ctx: &mut Ctx) {
        for signal in ctx.signals("score") {
            ctx.transform_mut().position.x += signal.payload.as_u64().unwrap() as f32;
        }
    }
}

#[test]
fn signals_deliver_next_frame_independently_of_callback_order_and_expire() {
    for sender_first in [true, false] {
        let host = ScriptHost::from_defs(vec![ScriptDef::of::<Sender>("Sender"), ScriptDef::of::<Receiver>("Receiver")]).unwrap();
        let mut scene = Scene::default();
        if sender_first {
            attach(&mut scene, "sender", &["Sender"]);
        }
        let a = attach(&mut scene, "receiver-a", &["Receiver"]);
        let b = attach(&mut scene, "receiver-b", &["Receiver"]);
        if !sender_first {
            attach(&mut scene, "sender", &["Sender"]);
        }
        let mut runtime = Runtime::new(scene, Some(host));
        runtime.tick(0.01);
        assert_eq!(runtime.scene.world.get::<Transform>(a).unwrap().position.x, 0.0);
        runtime.set_paused(true);
        runtime.tick(0.1);
        assert_eq!(runtime.scene.signals.pending_len(), 2);
        runtime.set_paused(false);
        runtime.tick(0.01);
        assert_eq!(runtime.scene.world.get::<Transform>(a).unwrap().position.x, 3.0);
        assert_eq!(runtime.scene.world.get::<Transform>(b).unwrap().position.x, 1.0);
        runtime.tick(0.01);
        assert_eq!(runtime.scene.world.get::<Transform>(a).unwrap().position.x, 3.0);
        assert!(runtime.scene.signals.incoming().is_empty());
        let snapshot = runtime.scene.snapshot(&kvas_engine::registry::Registry::default()).unwrap().0;
        assert!(snapshot.signals.incoming().is_empty());
        assert_eq!(snapshot.signals.pending_len(), 0);
    }
}
